use livtet_opds_atom::{FeedKind, parse_feed, render_feed};
use livtet_opds_types::{Feed, Link, Metadata, Publication, PublicationMetadata};

#[test]
fn text_is_escaped_and_round_trips_unescaped() {
    let feed = Feed {
        metadata: Metadata {
            title: "A & B < C".to_string(),
            identifier: None,
            type_: None,
            description: None,
            modified: Some(time::OffsetDateTime::now_utc()),
            number_of_items: None,
            items_per_page: None,
            current_page: None,
        },
        links: vec![Link {
            href: "http://example.com?a=1&b=2".to_string(),
            type_: "application/atom+xml".to_string(),
            templated: false,
            rel: vec!["self".to_string()],
            title: Some("Link \"title\"".to_string()),
            hreflang: None,
            length: None,
            properties: None,
        }],
        publications: vec![Publication {
            metadata: PublicationMetadata {
                title: "D > E".to_string(),
                identifier: None,
                description: Some("Line 1\nLine 2".to_string()),
                language: None,
                publisher: None,
                modified: time::OffsetDateTime::now_utc(),
                published: None,
                authors: Vec::new(),
                subjects: Vec::new(),
                translator: None,
                narrator: None,
                illustrator: None,
                editor: None,
                belongs_to: None,
            },
            links: Vec::new(),
            images: Vec::new(),
        }],
        navigation: Vec::new(),
        groups: Vec::new(),
        facets: Vec::new(),
    };

    let xml = render_feed(&feed, FeedKind::Acquisition).expect("render");
    eprintln!("{xml}");
    assert!(
        xml.contains("A &amp; B &lt; C"),
        "title should escape: {xml}"
    );
    assert!(
        xml.contains("http://example.com?a=1&amp;b=2"),
        "href should escape ampersand"
    );
    assert!(
        xml.contains("Link &quot;title&quot;"),
        "title attribute should escape quotes"
    );

    let parsed = parse_feed(&xml).expect("parse escaped feed");
    assert_eq!(parsed.metadata.title, "A & B < C");
    assert_eq!(
        parsed.links[0].href, "http://example.com?a=1&b=2",
        "href should unescape"
    );
    let pub_ = &parsed.publications[0];
    assert_eq!(pub_.metadata.title, "D > E");
    assert_eq!(pub_.metadata.description.as_deref(), Some("Line 1\nLine 2"));
}
