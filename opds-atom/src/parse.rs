//! Atom XML → OPDS type parsing.
//!
//! Uses a streaming `quick_xml::Reader` with a small manual state machine.
//! The parser is intentionally lenient: it recognizes elements no matter
//! whether they are prefixed (`atom:entry`) or in the default Atom
//! namespace (`<entry>`).

use std::collections::HashMap;

use livtet_opds_types::{
    Acquisition, Author, Availability, Collection, Feed, Link, Metadata, OpdsError, Price,
    Properties, Publication, PublicationMetadata, Series,
};
use quick_xml::{
    Reader, XmlVersion,
    events::{Event, attributes::Attribute},
};
use time::{Date, OffsetDateTime, Time};

/// Dispatch a text element's local name to the correct field on
/// `entry` (when parsing inside `<entry>`) or `metadata` (feed-level).
/// Shared between the `Event::Text` and `Event::End` handlers so the
/// 12-arm match block is written once.
fn handle_text_element(
    local: &str,
    text: &str,
    current_entry: &mut Option<PartialPublication>,
    metadata: &mut PartialMetadata,
    current_author: &mut Option<AuthorBuilder>,
) {
    match local {
        "id" => {
            if let Some(entry) = current_entry.as_mut() {
                entry.identifier = Some(text.to_string());
            } else {
                metadata.identifier = Some(text.to_string());
            }
        }
        "title" => {
            if let Some(entry) = current_entry.as_mut() {
                entry.title = text.to_string();
            } else {
                metadata.title = text.to_string();
            }
        }
        "updated" => {
            if let Some(dt) = parse_date(text) {
                if let Some(entry) = current_entry.as_mut() {
                    entry.modified = Some(dt);
                } else {
                    metadata.modified = Some(dt);
                }
            }
        }
        "published" => {
            if let Some(dt) = parse_date(text)
                && let Some(entry) = current_entry.as_mut()
            {
                entry.published = Some(dt);
            }
        }
        "summary" | "content" => {
            if let Some(entry) = current_entry.as_mut() {
                entry.description = Some(text.to_string());
            }
        }
        "rights" => {
            if let Some(entry) = current_entry.as_mut() {
                entry.description = Some(
                    entry
                        .description
                        .as_ref()
                        .map(|d| format!("{}\n{}", d, text))
                        .unwrap_or_else(|| text.to_string()),
                );
            }
        }
        "name" => {
            if let Some(author) = current_author.as_mut() {
                author.name = text.to_string();
            }
        }
        "language" => {
            if let Some(entry) = current_entry.as_mut() {
                entry.language = Some(text.to_string());
            }
        }
        "issued" => {
            if let Some(dt) = parse_date(text)
                && let Some(entry) = current_entry.as_mut()
            {
                entry.published = entry.published.or(Some(dt));
            }
        }
        "totalresults" => {
            if let Ok(n) = text.parse::<u64>() {
                metadata.number_of_items = Some(n);
            }
        }
        "itemsperpage" => {
            if let Ok(n) = text.parse::<u64>() {
                metadata.items_per_page = Some(n);
            }
        }
        "startindex" => {
            if let Ok(n) = text.parse::<u64>() {
                metadata.current_page = Some(n);
            }
        }
        _ => {}
    }
}

fn attr_value<R>(attr: &Attribute<'_>, reader: &Reader<R>) -> Result<String, OpdsError> {
    attr.decoded_and_normalized_value(XmlVersion::Implicit1_0, reader.decoder())
        .map_err(|err| OpdsError::Xml(format!("{err}")))
        .map(|cow| cow.into_owned())
}

use crate::rel::rel_from_atom;

/// Parse an Atom `<feed>` document into a [`Feed`].
#[cfg_attr(test, mutants::skip)]
pub fn parse_feed(body: &str) -> Result<Feed, OpdsError> {
    let mut parser = Parser::new(body)?;
    parser.parse_feed()
}

/// Parse an Atom `<entry>` document into a [`Publication`].
pub fn parse_entry(body: &str) -> Result<Publication, OpdsError> {
    let mut parser = Parser::new(body)?;
    parser.parse_entry_document()
}

struct Parser<'a> {
    reader: Reader<&'a [u8]>,
    buf: Vec<u8>,
    text_buf: Vec<u8>,
}

impl<'a> Parser<'a> {
    fn new(body: &'a str) -> Result<Self, OpdsError> {
        let reader = Reader::from_str(body);
        Ok(Self {
            reader,
            buf: Vec::new(),
            text_buf: Vec::new(),
        })
    }

    fn parse_feed(&mut self) -> Result<Feed, OpdsError> {
        let mut feed = Feed::new("");
        let mut metadata = PartialMetadata::default();
        let mut current_entry: Option<PartialPublication> = None;
        let mut current_link: Option<PartialLink> = None;
        let mut current_acquisition: Vec<AcquisitionBuilder> = Vec::new();
        let mut element_stack: Vec<String> = Vec::new();
        let mut current_element: Option<String> = None;
        let mut current_author: Option<AuthorBuilder> = None;
        let mut text_buffer: Vec<String> = Vec::new();

        loop {
            self.buf.clear();
            match self.reader.read_event_into(&mut self.buf) {
                Ok(Event::Start(ref e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref()).to_lowercase();
                    element_stack.push(local.clone());
                    current_element = Some(local.clone());

                    // For simple text-bearing elements, read the whole text content
                    // in one go; this preserves entity-escaped text correctly.
                    if matches!(
                        local.as_str(),
                        "id" | "title"
                            | "updated"
                            | "published"
                            | "summary"
                            | "content"
                            | "rights"
                            | "name"
                            | "language"
                            | "issued"
                            | "extent"
                            | "totalresults"
                            | "itemsperpage"
                            | "startindex"
                            | "price"
                    ) {
                        let text = self
                            .reader
                            .read_text_into(e.to_end().name(), &mut self.text_buf)
                            .map_err(|err| OpdsError::Xml(format!("{err}")))?;
                        let decoded = text
                            .xml10_content()
                            .map_err(|err| OpdsError::Xml(format!("{err}")))?;
                        let text = quick_xml::escape::unescape(&decoded)
                            .map_err(|err| OpdsError::Xml(format!("{err}")))?
                            .into_owned();
                        let text = text.trim();
                        if !text.is_empty() {
                            handle_text_element(
                                local.as_str(),
                                text,
                                &mut current_entry,
                                &mut metadata,
                                &mut current_author,
                            );
                        }
                        element_stack.pop();
                        current_element = element_stack.last().cloned();
                        continue;
                    }

                    match local.as_str() {
                        "entry" => {
                            current_entry = Some(PartialPublication::default());
                        }
                        "link" => {
                            let link = parse_link_start(e, &self.reader)?;
                            current_link = Some(link);
                        }
                        "indirectacquisition" => {
                            let mut acq_type = String::new();
                            for attr in e.attributes() {
                                let attr = attr.map_err(|err| OpdsError::Xml(format!("{err}")))?;
                                if local_name(attr.key.as_ref()).eq_ignore_ascii_case("type") {
                                    acq_type = attr_value(&attr, &self.reader)?;
                                }
                            }
                            let builder = AcquisitionBuilder {
                                type_: acq_type,
                                children: Vec::new(),
                            };
                            current_acquisition.push(builder);
                        }
                        "author" => {
                            current_author = Some(AuthorBuilder::default());
                        }
                        "contributor" => {
                            current_author = Some(AuthorBuilder {
                                role: Some("contributor".to_string()),
                                ..Default::default()
                            });
                        }
                        _ => {}
                    }
                }
                Ok(Event::Empty(e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref()).to_lowercase();
                    match local.as_str() {
                        "link" => {
                            let link = parse_link_start(&e, &self.reader)?;
                            flush_link(
                                &mut current_entry,
                                &mut feed,
                                link,
                                "feed",
                                element_stack.last().map(String::as_str),
                            );
                        }
                        "category" => {
                            if let Some(ref mut entry) = current_entry {
                                for attr in e.attributes() {
                                    let attr =
                                        attr.map_err(|err| OpdsError::Xml(format!("{err}")))?;
                                    let name = local_name(attr.key.as_ref());
                                    // OPDS 1.x uses `term` for the subject
                                    // identifier; OPDS 2 drafts and some
                                    // catalogs use `label` for the same
                                    // field. Treat them identically.
                                    if name.eq_ignore_ascii_case("term")
                                        || name.eq_ignore_ascii_case("label")
                                    {
                                        let value = attr_value(&attr, &self.reader)?;
                                        if !value.is_empty() && !entry.subjects.contains(&value) {
                                            entry.subjects.push(value);
                                        }
                                    }
                                }
                            }
                        }
                        "indirectacquisition" => {
                            let mut acq_type = String::new();
                            for attr in e.attributes() {
                                let attr = attr.map_err(|err| OpdsError::Xml(format!("{err}")))?;
                                if local_name(attr.key.as_ref()).eq_ignore_ascii_case("type") {
                                    acq_type = attr_value(&attr, &self.reader)?;
                                }
                            }
                            let builder = AcquisitionBuilder {
                                type_: acq_type,
                                children: Vec::new(),
                            };
                            if let Some(parent) = current_acquisition.last_mut() {
                                parent.children.push(builder);
                            } else if let Some(ref mut link) = current_link {
                                link.child_acquisitions.push(builder.into_acquisition());
                            }
                        }
                        "unavailable" | "available" | "holdable" => {
                            let mut state = local.clone();
                            let mut since = None;
                            let mut until = None;
                            for attr in e.attributes() {
                                let attr = attr.map_err(|err| OpdsError::Xml(format!("{err}")))?;
                                let name = local_name(attr.key.as_ref());
                                if name.eq_ignore_ascii_case("status")
                                    || name.eq_ignore_ascii_case("state")
                                {
                                    state = attr_value(&attr, &self.reader)?;
                                } else if name.eq_ignore_ascii_case("since") {
                                    if let Some(dt) = parse_date(&attr_value(&attr, &self.reader)?)
                                    {
                                        since = Some(dt);
                                    }
                                } else if (name.eq_ignore_ascii_case("date")
                                    || name.eq_ignore_ascii_case("until"))
                                    && let Some(dt) = parse_date(&attr_value(&attr, &self.reader)?)
                                {
                                    until = Some(dt);
                                }
                            }
                            let availability = Availability {
                                state,
                                since,
                                until,
                            };
                            if let Some(ref mut link) = current_link {
                                link.availability = Some(availability);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(_)) => {
                    // Entity references split text events; we collect full content
                    // when the element ends via read_text_into, so ignore stray Text
                    // events that arrive between Start and End.
                }
                Ok(Event::CData(e)) => {
                    if let Some(ref _element) = current_element {
                        let text = e
                            .xml10_content()
                            .map_err(|err| OpdsError::Xml(format!("{err}")))?
                            .into_owned();
                        text_buffer.push(text);
                    }
                }
                Ok(Event::End(e)) => {
                    let name = e.name();
                    let local = local_name(name.as_ref()).to_lowercase();

                    // Flush accumulated text when we close the element that opened it.
                    if !text_buffer.is_empty()
                        && let Some(ref element) = current_element
                        && element == &local
                    {
                        let text = text_buffer.concat().trim().to_string();
                        text_buffer.clear();
                        handle_text_element(
                            local.as_str(),
                            &text,
                            &mut current_entry,
                            &mut metadata,
                            &mut current_author,
                        );
                        element_stack.pop();
                        current_element = element_stack.last().cloned();
                        continue;
                    }

                    if local == "indirectacquisition" {
                        if let Some(builder) = current_acquisition.pop() {
                            let acq = builder.into_acquisition();
                            if let Some(parent) = current_acquisition.last_mut() {
                                parent.children.push(AcquisitionBuilder {
                                    type_: acq.type_,
                                    children: acq
                                        .child
                                        .into_iter()
                                        .map(|c| AcquisitionBuilder {
                                            type_: c.type_,
                                            children: Vec::new(),
                                        })
                                        .collect(),
                                });
                            } else if let Some(link) = current_link.as_mut() {
                                link.child_acquisitions.push(acq);
                            }
                        }
                        element_stack.pop();
                        current_element = element_stack.last().cloned();
                        continue;
                    }

                    match local.as_str() {
                        "feed" => break,
                        "entry" => {
                            if let Some(entry) = current_entry.take() {
                                if entry.is_publication() {
                                    feed.publications.push(entry.into_publication());
                                } else if let Some(link) = entry.into_navigation_link() {
                                    feed.navigation.push(link);
                                }
                            }
                        }
                        "link" => {
                            if let Some(link) = current_link.take() {
                                flush_link(
                                    &mut current_entry,
                                    &mut feed,
                                    link,
                                    "feed",
                                    element_stack.last().map(String::as_str),
                                );
                            }
                        }
                        "author" | "contributor" => {
                            if let Some(author) = current_author.take()
                                && !author.name.is_empty()
                            {
                                let author = author.into_author();
                                if let Some(entry) = current_entry.as_mut() {
                                    entry.authors.push(author);
                                }
                            }
                        }
                        _ => {}
                    }

                    element_stack.pop();
                    current_element = element_stack.last().cloned();
                }
                Ok(Event::Eof) => break,
                Err(err) => return Err(OpdsError::Xml(format!("{err}"))),
                _ => {}
            }
        }

        feed.metadata.title = metadata.title;
        feed.metadata.identifier = metadata.identifier;
        feed.metadata.modified = metadata.modified;
        feed.metadata.number_of_items = metadata.number_of_items;
        feed.metadata.items_per_page = metadata.items_per_page;
        feed.metadata.current_page = metadata.current_page;
        Ok(feed)
    }

    fn parse_entry_document(&mut self) -> Result<Publication, OpdsError> {
        let mut feed = self.parse_feed()?;
        feed.publications
            .pop()
            .ok_or_else(|| OpdsError::MissingField("entry".to_string()))
    }
}

fn parse_link_start<R>(
    e: &quick_xml::events::BytesStart<'_>,
    reader: &Reader<R>,
) -> Result<PartialLink, OpdsError> {
    let mut link = PartialLink::default();
    for attr in e.attributes() {
        let attr = attr.map_err(|err| OpdsError::Xml(format!("{err}")))?;
        let name = local_name(attr.key.as_ref());
        let value = attr_value(&attr, reader)?;
        match name.to_lowercase().as_str() {
            "href" => link.href = value,
            "rel" => {
                for rel in value.split_whitespace() {
                    link.rel.push(rel_from_atom(rel).into_owned());
                }
            }
            "type" => link.type_ = value,
            "title" => link.title = Some(value),
            "hreflang" => link.hreflang = Some(value),
            "length" => {
                if let Ok(n) = value.parse::<u64>() {
                    link.length = Some(n);
                }
            }
            "facetgroup" => link.facet_group = Some(value),
            "activefacet" => link.active_facet = Some(value),
            _ => {}
        }
    }
    Ok(link)
}

fn flush_link(
    current_entry: &mut Option<PartialPublication>,
    feed: &mut Feed,
    partial: PartialLink,
    default_target: &str,
    _parent: Option<&str>,
) {
    let link = partial.into_link();
    if let Some(entry) = current_entry.as_mut() {
        if link
            .rel
            .iter()
            .any(|r| r == "image" || r == "image/thumbnail")
        {
            entry.images.push(link);
        } else {
            entry.links.push(link);
        }
    } else if default_target == "feed" {
        feed.links.push(link);
    }
}

#[derive(Default)]
struct PartialMetadata {
    title: String,
    identifier: Option<String>,
    modified: Option<OffsetDateTime>,
    number_of_items: Option<u64>,
    items_per_page: Option<u64>,
    current_page: Option<u64>,
}

#[derive(Default)]
struct PartialPublication {
    title: String,
    identifier: Option<String>,
    description: Option<String>,
    language: Option<String>,
    modified: Option<OffsetDateTime>,
    published: Option<OffsetDateTime>,
    authors: Vec<Author>,
    subjects: Vec<String>,
    links: Vec<Link>,
    images: Vec<Link>,
}

impl PartialPublication {
    fn is_publication(&self) -> bool {
        self.links.is_empty()
            || self.links.iter().any(|l| {
                l.rel.iter().any(|r| {
                    matches!(
                        r.as_str(),
                        "acquisition"
                            | "open-access"
                            | "borrow"
                            | "buy"
                            | "sample"
                            | "subscribe"
                            | "image"
                            | "image/thumbnail"
                    )
                }) || l.type_.starts_with("image/")
            })
    }

    fn into_publication(self) -> Publication {
        Publication {
            metadata: PublicationMetadata {
                title: self.title,
                identifier: self.identifier,
                description: self.description,
                language: self.language,
                publisher: None,
                modified: self.modified,
                published: self.published,
                authors: self.authors,
                subjects: self.subjects,
                translator: None,
                narrator: None,
                illustrator: None,
                editor: None,
                belongs_to: None,
            },
            links: self.links,
            images: self.images,
        }
    }

    /// Convert a navigation `<entry>` into a compact navigation link. OPDS 1.x
    /// navigation entries wrap a single subsection link; the entry title names
    /// the target. Entries with no link have nothing to point at and are
    /// dropped.
    fn into_navigation_link(self) -> Option<Link> {
        let title = (!self.title.is_empty()).then_some(self.title);
        self.links.into_iter().next().map(|mut link| {
            if link.title.is_none() {
                link.title = title;
            }
            link
        })
    }
}

#[derive(Default)]
struct PartialLink {
    href: String,
    type_: String,
    rel: Vec<String>,
    title: Option<String>,
    hreflang: Option<String>,
    length: Option<u64>,
    facet_group: Option<String>,
    active_facet: Option<String>,
    child_acquisitions: Vec<Acquisition>,
    price: Option<Price>,
    availability: Option<Availability>,
    number_of_items: Option<u64>,
}

impl PartialLink {
    fn into_link(self) -> Link {
        let properties = if self.child_acquisitions.is_empty()
            && self.price.is_none()
            && self.availability.is_none()
            && self.number_of_items.is_none()
        {
            None
        } else {
            Some(Properties {
                number_of_items: self.number_of_items,
                price: self.price,
                indirect_acquisition: if self.child_acquisitions.is_empty() {
                    None
                } else {
                    Some(self.child_acquisitions)
                },
                availability: self.availability,
            })
        };

        Link {
            href: self.href,
            type_: self.type_,
            templated: false,
            rel: self.rel,
            title: self.title,
            hreflang: self.hreflang,
            length: self.length,
            properties,
        }
    }
}

#[derive(Default)]
struct AuthorBuilder {
    name: String,
    role: Option<String>,
}

impl AuthorBuilder {
    fn into_author(self) -> Author {
        Author {
            name: self.name,
            sort_as: None,
            role: self.role,
            links: Vec::new(),
        }
    }
}

#[derive(Default)]
struct AcquisitionBuilder {
    type_: String,
    children: Vec<AcquisitionBuilder>,
}

impl AcquisitionBuilder {
    fn into_acquisition(self) -> Acquisition {
        Acquisition {
            type_: self.type_,
            child: self
                .children
                .into_iter()
                .map(AcquisitionBuilder::into_acquisition)
                .collect(),
        }
    }
}

fn local_name(name: &[u8]) -> &str {
    let s = std::str::from_utf8(name).unwrap_or("");
    s.split_once(':').map_or(s, |(_, local)| local)
}

/// Parse a lenient date: RFC 3339, `yyyy-mm-dd`, or `yyyy`.
pub fn parse_date(value: &str) -> Option<OffsetDateTime> {
    // FIXME: This seems excessive; let's make use of what `time` provides for parsing date
    // The formats would be RFC 3339, `YYYY-MM-DD` and just `YYYY`
    let value = value.trim();
    if value.is_empty() {
        return None;
    }

    // Try RFC 3339 first.
    if let Ok(dt) = OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339) {
        return Some(dt);
    }

    // Try `yyyy-mm-dd`.
    let mut parts = value.split('-');
    if let (Some(y), Some(m), Some(d), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
        && let (Ok(year), Ok(month), Ok(day)) = (y.parse::<i32>(), m.parse::<u8>(), d.parse::<u8>())
        && let Ok(date) = Date::from_calendar_date(year, month.try_into().ok()?, day)
    {
        return Some(OffsetDateTime::new_utc(date, Time::MIDNIGHT));
    }

    // Try `yyyy`.
    if let Ok(year) = value.parse::<i32>() {
        // limit to a sensible historical range.
        if (0..=10_000).contains(&year)
            && let Ok(date) = Date::from_calendar_date(year, time::Month::January, 1)
        {
            return Some(OffsetDateTime::new_utc(date, Time::MIDNIGHT));
        }
    }

    None
}

// Silence unused warning for `Collection` and `Series` imports if they end
// up unused in this iteration of the parser; the types may participate in a
// future extension to parse `dcterms:Alternative` series metadata.
#[allow(dead_code)]
fn _unused_collection() -> Collection {
    Collection {
        metadata: Metadata {
            title: String::new(),
            identifier: None,
            type_: None,
            description: None,
            modified: None,
            number_of_items: None,
            items_per_page: None,
            current_page: None,
        },
        links: Vec::new(),
    }
}

#[allow(dead_code)]
fn _unused_series() -> Series {
    Series {
        name: String::new(),
        position: None,
    }
}

#[allow(dead_code)]
fn _unused_hash_map() -> HashMap<String, String> {
    HashMap::new()
}
