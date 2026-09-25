use livtet_opds_atom::{FeedKind, parse_feed, render_feed};
use livtet_opds_types::{
    Acquisition, Author, Availability, Feed, Link, Metadata, Price, Properties, Publication,
    PublicationMetadata,
};

fn sample_feed() -> Feed {
    Feed {
        metadata: Metadata {
            title: "Test Feed".to_string(),
            identifier: Some("urn:test".to_string()),
            type_: Some("application/opds+json".to_string()),
            description: None,
            modified: Some(time::OffsetDateTime::now_utc()),
            number_of_items: None,
            items_per_page: None,
            current_page: None,
        },
        links: vec![Link {
            href: "http://example.com/root".to_string(),
            type_: "application/atom+xml;profile=opds-catalog;kind=navigation".to_string(),
            templated: false,
            rel: vec!["self".to_string()],
            title: Some("Home".to_string()),
            hreflang: None,
            length: None,
            properties: None,
        }],
        publications: Vec::new(),
        navigation: vec![Link {
            href: "http://example.com/sub".to_string(),
            type_: "application/atom+xml;profile=opds-catalog;kind=acquisition".to_string(),
            templated: false,
            rel: vec!["subsection".to_string()],
            title: Some("Subsection".to_string()),
            hreflang: None,
            length: None,
            properties: None,
        }],
        groups: Vec::new(),
        facets: Vec::new(),
    }
}

fn full_feed() -> Feed {
    Feed {
        metadata: Metadata {
            title: "Full Feed".to_string(),
            identifier: Some("urn:full".to_string()),
            type_: Some("application/opds+json".to_string()),
            description: Some("A full feed".to_string()),
            modified: Some(time::OffsetDateTime::now_utc()),
            number_of_items: Some(42),
            items_per_page: Some(10),
            current_page: Some(1),
        },
        links: vec![
            Link {
                href: "http://example.com/self".to_string(),
                type_: "application/atom+xml;profile=opds-catalog;kind=navigation".to_string(),
                templated: false,
                rel: vec!["self".to_string()],
                title: Some("Self".to_string()),
                hreflang: Some("en".to_string()),
                length: Some(1234),
                properties: None,
            },
            Link {
                href: "http://example.com/search?q={searchTerms}".to_string(),
                type_: "application/opensearchdescription+xml".to_string(),
                templated: true,
                rel: vec!["search".to_string()],
                title: Some("Search".to_string()),
                hreflang: None,
                length: None,
                properties: None,
            },
        ],
        publications: vec![Publication {
            metadata: PublicationMetadata {
                title: "Book One".to_string(),
                identifier: Some("urn:book:1".to_string()),
                description: Some("A great book".to_string()),
                language: Some("en".to_string()),
                publisher: Some("Publisher Inc".to_string()),
                modified: Some(time::OffsetDateTime::now_utc()),
                published: Some(time::OffsetDateTime::now_utc()),
                authors: vec![Author {
                    name: "Author One".to_string(),
                    sort_as: None,
                    role: Some("author".to_string()),
                    links: Vec::new(),
                }],
                subjects: vec!["Fiction".to_string(), "Drama".to_string()],
                translator: None,
                narrator: None,
                illustrator: None,
                editor: None,
                belongs_to: None,
            },
            links: vec![Link {
                href: "http://example.com/book1.acq".to_string(),
                type_: "application/epub+zip".to_string(),
                templated: false,
                rel: vec!["acquisition".to_string()],
                title: Some("Download".to_string()),
                hreflang: None,
                length: Some(500000),
                properties: Some(Properties {
                    number_of_items: None,
                    price: Some(Price {
                        currency: "USD".to_string(),
                        value: 9.99,
                    }),
                    indirect_acquisition: Some(vec![Acquisition {
                        type_: "application/vnd.adobe.adept+xml".to_string(),
                        child: Vec::new(),
                    }]),
                    availability: Some(Availability {
                        state: "available".to_string(),
                        since: None,
                        until: None,
                    }),
                }),
            }],
            images: vec![Link {
                href: "http://example.com/book1.jpg".to_string(),
                type_: "image/jpeg".to_string(),
                templated: false,
                rel: vec!["image".to_string()],
                title: Some("Cover".to_string()),
                hreflang: None,
                length: None,
                properties: None,
            }],
        }],
        navigation: Vec::new(),
        groups: Vec::new(),
        facets: Vec::new(),
    }
}

#[test]
fn navigation_feed_round_trip() {
    let feed = sample_feed();
    let xml = render_feed(&feed, FeedKind::Navigation).expect("render");
    let parsed = parse_feed(&xml).expect("parse");

    assert_eq!(parsed.metadata.title, "Test Feed");
    assert!(
        parsed
            .navigation
            .iter()
            .any(|l| l.title.as_deref() == Some("Subsection"))
    );
    assert!(
        parsed
            .navigation
            .iter()
            .any(|l| l.href == "http://example.com/sub")
    );
    assert!(
        parsed
            .links
            .iter()
            .any(|l| l.rel.contains(&"self".to_string()))
    );
}

#[test]
fn xml_declaration_and_atom_namespace_present() {
    let xml = render_feed(&sample_feed(), FeedKind::Navigation).expect("render");
    assert!(xml.starts_with("<?xml"));
    assert!(xml.contains("xmlns=\"http://www.w3.org/2005/Atom\""));
    assert!(xml.contains("<feed "));
}

#[test]
fn full_feed_round_trip_preserves_all_fields() {
    let feed = full_feed();
    let xml = render_feed(&feed, FeedKind::Acquisition).expect("render");
    let parsed = parse_feed(&xml).expect("parse");

    // Metadata
    assert_eq!(parsed.metadata.title, "Full Feed");
    assert_eq!(parsed.metadata.identifier, Some("urn:full".to_string()));
    assert_eq!(parsed.metadata.number_of_items, Some(42));
    assert_eq!(parsed.metadata.items_per_page, Some(10));
    assert_eq!(parsed.metadata.current_page, Some(0));

    // Links
    let self_link = parsed
        .links
        .iter()
        .find(|l| l.rel.contains(&"self".to_string()))
        .expect("self link");
    assert_eq!(self_link.href, "http://example.com/self");
    assert_eq!(self_link.title, Some("Self".to_string()));

    // Publications
    assert_eq!(parsed.publications.len(), 1);
    let book = &parsed.publications[0];
    assert_eq!(book.metadata.title, "Book One");
    assert_eq!(book.metadata.identifier, Some("urn:book:1".to_string()));
    assert_eq!(book.metadata.description, Some("A great book".to_string()));
    assert_eq!(book.metadata.language, Some("en".to_string()));
    assert_eq!(book.metadata.authors.len(), 1);
    assert_eq!(book.metadata.authors[0].name, "Author One");

    // Acquisition link with properties
    let acq = book
        .links
        .iter()
        .find(|l| l.rel.contains(&"acquisition".to_string()))
        .expect("acquisition link");
    assert_eq!(acq.href, "http://example.com/book1.acq");
    assert_eq!(acq.type_, "application/epub+zip");
    assert_eq!(acq.length, Some(500000));
    let props = acq.properties.as_ref().expect("properties");
    assert_eq!(
        props
            .indirect_acquisition
            .as_ref()
            .map(|a| a[0].type_.as_str()),
        Some("application/vnd.adobe.adept+xml")
    );
    assert_eq!(
        props.availability.as_ref().map(|a| a.state.as_str()),
        Some("available")
    );

    // Image link
    let img = book
        .images
        .iter()
        .find(|l| l.rel.contains(&"image".to_string()))
        .expect("image link");
    assert_eq!(img.href, "http://example.com/book1.jpg");
    assert_eq!(img.type_, "image/jpeg");
}
