//! OPDS type → Atom XML serialization.
//!
//! Uses `quick_xml::Writer` to emit well-formed OPDS 1.x Atom feeds and
//! entries. Element order follows Atom RFC 4287: `id`, `title`, `updated`,
//! links, then entries.

use std::io::Cursor;

use itertools::Itertools;
use livtet_opds_types::{Acquisition, Author, Feed, Link, OpdsError, Price, Publication};
use quick_xml::{
    events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event},
    writer::Writer,
};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

use crate::rel::rel_to_atom;

/// Whether a feed is a navigation or acquisition catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedKind {
    Navigation,
    Acquisition,
    SearchResult,
}

/// Render a [`Feed`] as an OPDS 1.x Atom XML document.
pub fn render_feed(feed: &Feed, kind: FeedKind) -> Result<String, OpdsError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(xml_err)?;

    let mut root = BytesStart::new("feed");
    root.push_attribute(("xmlns", "http://www.w3.org/2005/Atom"));
    root.push_attribute(("xmlns:dc", "http://purl.org/dc/terms/"));
    root.push_attribute(("xmlns:opds", "http://opds-spec.org/2010/catalog"));
    root.push_attribute(("xmlns:opensearch", "http://a9.com/-/spec/opensearch/1.1/"));
    writer.write_event(Event::Start(root)).map_err(xml_err)?;

    // Required ordered feed metadata.
    write_text_element(&mut writer, "id", &feed_id(feed))?;
    write_text_element(&mut writer, "title", &feed.metadata.title)?;
    write_text_element(
        &mut writer,
        "updated",
        &format_date(
            feed.metadata
                .modified
                .unwrap_or_else(OffsetDateTime::now_utc),
        ),
    )?;

    // Feed-level links.
    for link in &feed.links {
        write_feed_link(&mut writer, link)?;
    }

    // Facets are serialized as direct `<link>` children of the feed.
    for facet in &feed.facets {
        write_facet_group(&mut writer, facet, kind)?;
    }

    // Search metadata.
    if let Some(n) = feed.metadata.number_of_items {
        let elem = BytesStart::new("opensearch:totalResults");
        let text = n.to_string();
        writer.write_event(Event::Start(elem)).map_err(xml_err)?;
        writer
            .write_event(Event::Text(BytesText::new(&text)))
            .map_err(xml_err)?;
        writer
            .write_event(Event::End(BytesEnd::new("opensearch:totalResults")))
            .map_err(xml_err)?;
    }
    if let Some(n) = feed.metadata.items_per_page {
        let elem = BytesStart::new("opensearch:itemsPerPage");
        let text = n.to_string();
        writer.write_event(Event::Start(elem)).map_err(xml_err)?;
        writer
            .write_event(Event::Text(BytesText::new(&text)))
            .map_err(xml_err)?;
        writer
            .write_event(Event::End(BytesEnd::new("opensearch:itemsPerPage")))
            .map_err(xml_err)?;
    }
    if let Some(n) = feed.metadata.current_page {
        let elem = BytesStart::new("opensearch:startIndex");
        let text = ((n.max(1) - 1)
            .checked_mul(feed.metadata.items_per_page.unwrap_or(1))
            .unwrap_or(0))
        .to_string();
        writer.write_event(Event::Start(elem)).map_err(xml_err)?;
        writer
            .write_event(Event::Text(BytesText::new(&text)))
            .map_err(xml_err)?;
        writer
            .write_event(Event::End(BytesEnd::new("opensearch:startIndex")))
            .map_err(xml_err)?;
    }

    // Entries.
    match kind {
        FeedKind::Navigation => {
            for collection in &feed.navigation {
                write_navigation_entry(&mut writer, collection)?;
            }
            for collection in &feed.groups {
                write_navigation_entry(&mut writer, collection)?;
            }
        }
        FeedKind::Acquisition => {
            for publication in &feed.publications {
                write_publication_entry(&mut writer, publication)?;
            }
        }
        FeedKind::SearchResult => {
            for publication in &feed.publications {
                write_publication_entry(&mut writer, publication)?;
            }
        }
    }

    writer
        .write_event(Event::End(BytesEnd::new("feed")))
        .map_err(xml_err)?;

    String::from_utf8(writer.into_inner().into_inner()).map_err(|e| OpdsError::Xml(e.to_string()))
}

/// Render a single [`Publication`] as an Atom `<entry>` document.
pub fn render_entry(publication: &Publication) -> Result<String, OpdsError> {
    let mut writer = Writer::new(Cursor::new(Vec::new()));
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
        .map_err(xml_err)?;
    writer
        .write_event(Event::Start(BytesStart::new("entry")))
        .map_err(xml_err)?;

    write_publication_entry_body(&mut writer, publication)?;

    writer
        .write_event(Event::End(BytesEnd::new("entry")))
        .map_err(xml_err)?;

    String::from_utf8(writer.into_inner().into_inner()).map_err(|e| OpdsError::Xml(e.to_string()))
}

fn feed_id(feed: &Feed) -> String {
    feed.metadata
        .identifier
        .clone()
        .unwrap_or_else(|| "tag:livtet.opds.feed".to_string())
}

fn write_text_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: &str,
) -> Result<(), OpdsError> {
    writer
        .write_event(Event::Start(BytesStart::new(name)))
        .map_err(xml_err)?;
    writer
        .write_event(Event::Text(BytesText::new(value)))
        .map_err(xml_err)?;
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(xml_err)
}

fn write_link_inner<W: std::io::Write>(
    writer: &mut Writer<W>,
    link: &Link,
    rel_fn: &dyn Fn(&[String]) -> String,
    extra_attrs: &[(&str, &str)],
) -> Result<(), OpdsError> {
    let mut elem = BytesStart::new("link");
    elem.push_attribute(("href", link.href.as_str()));
    elem.push_attribute(("type", link.type_.as_str()));
    let rel = rel_fn(&link.rel);
    if !rel.is_empty() {
        elem.push_attribute(("rel", rel.as_str()));
    }
    if let Some(title) = &link.title {
        elem.push_attribute(("title", title.as_str()));
    }
    for (k, v) in extra_attrs {
        elem.push_attribute((*k, *v));
    }

    let mut has_children = false;
    if let Some(props) = &link.properties
        && (props.indirect_acquisition.is_some()
            || props.price.is_some()
            || props.availability.is_some())
    {
        has_children = true;
    }

    if has_children {
        writer.write_event(Event::Start(elem)).map_err(xml_err)?;
        if let Some(props) = &link.properties {
            if let Some(children) = &props.indirect_acquisition {
                for child in children {
                    write_indirect_acquisition(writer, child)?;
                }
            }
            if let Some(price) = &props.price {
                write_price(writer, price)?;
            }
            if let Some(availability) = &props.availability {
                write_availability(writer, availability)?;
            }
        }
        writer
            .write_event(Event::End(BytesEnd::new("link")))
            .map_err(xml_err)?;
    } else {
        writer.write_event(Event::Empty(elem)).map_err(xml_err)?;
    }

    Ok(())
}

fn write_feed_link<W: std::io::Write>(
    writer: &mut Writer<W>,
    link: &Link,
) -> Result<(), OpdsError> {
    write_link_inner(writer, link, &|rels| rels.join(" "), &[])
}

fn write_facet_group<W: std::io::Write>(
    writer: &mut Writer<W>,
    facet: &livtet_opds_types::Collection,
    _kind: FeedKind,
) -> Result<(), OpdsError> {
    for link in &facet.links {
        let mut elem = BytesStart::new("link");
        elem.push_attribute(("href", link.href.as_str()));
        elem.push_attribute(("type", link.type_.as_str()));
        let active = link.rel.iter().any(|r| r.eq_ignore_ascii_case("self"));
        let rel = if active {
            "self"
        } else {
            "http://opds-spec.org/shelf"
        };
        elem.push_attribute(("rel", rel));
        if let Some(title) = &link.title {
            elem.push_attribute(("title", title.as_str()));
        }
        elem.push_attribute(("opds:facetGroup", facet.metadata.title.as_str()));
        elem.push_attribute(("opds:activeFacet", active.to_string().as_str()));
        if let Some(props) = &link.properties
            && let Some(n) = props.number_of_items
        {
            elem.push_attribute(("opds:count", n.to_string().as_str()));
        }
        writer.write_event(Event::Empty(elem)).map_err(xml_err)?;
    }
    Ok(())
}

fn write_navigation_entry<W: std::io::Write>(
    writer: &mut Writer<W>,
    collection: &livtet_opds_types::Collection,
) -> Result<(), OpdsError> {
    writer
        .write_event(Event::Start(BytesStart::new("entry")))
        .map_err(xml_err)?;

    write_text_element(
        writer,
        "id",
        collection
            .metadata
            .identifier
            .as_deref()
            .unwrap_or(&collection.metadata.title),
    )?;
    write_text_element(writer, "title", &collection.metadata.title)?;
    write_text_element(
        writer,
        "updated",
        &format_date(
            collection
                .metadata
                .modified
                .unwrap_or_else(OffsetDateTime::now_utc),
        ),
    )?;

    for link in &collection.links {
        write_feed_link(writer, link)?;
    }

    writer
        .write_event(Event::End(BytesEnd::new("entry")))
        .map_err(xml_err)?;
    Ok(())
}

fn write_publication_entry<W: std::io::Write>(
    writer: &mut Writer<W>,
    publication: &Publication,
) -> Result<(), OpdsError> {
    writer
        .write_event(Event::Start(BytesStart::new("entry")))
        .map_err(xml_err)?;
    write_publication_entry_body(writer, publication)?;
    writer
        .write_event(Event::End(BytesEnd::new("entry")))
        .map_err(xml_err)?;
    Ok(())
}

fn write_publication_entry_body<W: std::io::Write>(
    writer: &mut Writer<W>,
    publication: &Publication,
) -> Result<(), OpdsError> {
    let meta = &publication.metadata;

    write_text_element(
        writer,
        "id",
        meta.identifier.as_deref().unwrap_or("urn:uuid:missing"),
    )?;
    write_text_element(writer, "title", &meta.title)?;
    write_text_element(writer, "updated", &format_date(meta.modified))?;

    if let Some(dt) = meta.published {
        write_text_element(writer, "published", &format_date(dt))?;
    }

    for author in &meta.authors {
        write_author(writer, author)?;
    }

    if let Some(lang) = &meta.language {
        write_text_element(writer, "dc:language", lang)?;
    }

    if let Some(description) = &meta.description {
        write_text_element(writer, "summary", description.trim())?;
    }

    // Acquisition links first, then images, then remaining links.
    let (acquisition, rest): (Vec<_>, Vec<_>) = publication
        .links
        .iter()
        .partition(|l| l.rel.iter().any(|r| is_acquisition_rel(r)));

    for link in acquisition {
        write_publication_link(writer, link)?;
    }
    for image in &publication.images {
        write_publication_link(writer, image)?;
    }
    for link in rest {
        write_publication_link(writer, link)?;
    }

    Ok(())
}

fn write_author<W: std::io::Write>(
    writer: &mut Writer<W>,
    author: &Author,
) -> Result<(), OpdsError> {
    writer
        .write_event(Event::Start(BytesStart::new("author")))
        .map_err(xml_err)?;
    write_text_element(writer, "name", &author.name)?;
    writer
        .write_event(Event::End(BytesEnd::new("author")))
        .map_err(xml_err)?;
    Ok(())
}

fn write_publication_link<W: std::io::Write>(
    writer: &mut Writer<W>,
    link: &Link,
) -> Result<(), OpdsError> {
    let length_str = link.length.map(|n| n.to_string());
    let mut extra = Vec::new();
    if let Some(lang) = &link.hreflang {
        extra.push(("hreflang", lang.as_str()));
    }
    if let Some(s) = length_str.as_deref() {
        extra.push(("length", s));
    }
    write_link_inner(writer, link, &|rels| map_rels_for_atom(rels), &extra)
}

fn write_indirect_acquisition<W: std::io::Write>(
    writer: &mut Writer<W>,
    acq: &Acquisition,
) -> Result<(), OpdsError> {
    if acq.child.is_empty() {
        let mut elem = BytesStart::new("opds:indirectAcquisition");
        elem.push_attribute(("type", acq.type_.as_str()));
        writer.write_event(Event::Empty(elem)).map_err(xml_err)
    } else {
        let mut elem = BytesStart::new("opds:indirectAcquisition");
        elem.push_attribute(("type", acq.type_.as_str()));
        writer.write_event(Event::Start(elem)).map_err(xml_err)?;
        for child in &acq.child {
            write_indirect_acquisition(writer, child)?;
        }
        writer
            .write_event(Event::End(BytesEnd::new("opds:indirectAcquisition")))
            .map_err(xml_err)
    }
}

fn write_price<W: std::io::Write>(writer: &mut Writer<W>, price: &Price) -> Result<(), OpdsError> {
    let mut elem = BytesStart::new("opds:price");
    elem.push_attribute(("currencycode", price.currency.as_str()));
    writer.write_event(Event::Start(elem)).map_err(xml_err)?;
    writer
        .write_event(Event::Text(BytesText::new(&format!("{:.2}", price.value))))
        .map_err(xml_err)?;
    writer
        .write_event(Event::End(BytesEnd::new("opds:price")))
        .map_err(xml_err)
}

fn write_availability<W: std::io::Write>(
    writer: &mut Writer<W>,
    availability: &livtet_opds_types::Availability,
) -> Result<(), OpdsError> {
    let state = availability.state.to_lowercase();
    let (tag, mut attr) = match state.as_str() {
        "available" | "ready" => ("opds:available", None),
        "unavailable" => ("opds:unavailable", availability.until),
        _ => ("opds:available", availability.until),
    };
    if let Some(since) = availability.since {
        attr = Some(since);
    }
    let mut elem = BytesStart::new(tag);
    if let Some(dt) = attr {
        elem.push_attribute(("date", format_date(dt).as_str()));
    }
    writer.write_event(Event::Empty(elem)).map_err(xml_err)
}

fn map_rels_for_atom(rels: &[String]) -> String {
    let mapped = rels.iter().map(|r| rel_to_atom(r).into_owned());
    Itertools::intersperse_with(mapped, || " ".to_string()).collect()
}

fn is_acquisition_rel(rel: &str) -> bool {
    rel == "acquisition"
        || rel.starts_with("http://opds-spec.org/acquisition")
        || rel == "open-access"
        || rel == "borrow"
        || rel == "buy"
        || rel == "sample"
        || rel == "subscribe"
}

fn format_date(dt: OffsetDateTime) -> String {
    dt.format(&Rfc3339).unwrap_or_else(|_| {
        OffsetDateTime::now_utc()
            .format(&Rfc3339)
            .unwrap_or_default()
    })
}

fn xml_err<E: std::fmt::Display>(e: E) -> OpdsError {
    OpdsError::Xml(e.to_string())
}
