use livtet_opds_types::*;

#[test]
fn feed_round_trip() {
    let mut feed = Feed::new("Test Catalog");
    feed.metadata.identifier = Some("urn:test:1".to_string());
    feed.links.push(Link::new(
        "/feed.json",
        "application/opds+json",
        vec!["self".to_string()],
    ));

    let json = serde_json::to_string(&feed).unwrap();
    let parsed: Feed = serde_json::from_str(&json).unwrap();

    assert_eq!(feed.metadata.title, parsed.metadata.title);
    assert_eq!(feed.links[0].href, parsed.links[0].href);
}

#[test]
fn link_with_templated_flag() {
    let mut link = Link::new(
        "/search{?query}",
        "application/opds+json",
        vec!["search".to_string()],
    );
    link.templated = true;

    let json = serde_json::to_string(&link).unwrap();
    assert!(json.contains("\"templated\":true"));

    let parsed: Link = serde_json::from_str(&json).unwrap();
    assert!(parsed.templated);
    assert!(parsed.is_search());
}

#[test]
fn publication_round_trip() {
    let pub_ = Publication {
        metadata: PublicationMetadata {
            title: "Test Book".to_string(),
            identifier: Some("urn:isbn:123".to_string()),
            language: Some("en".to_string()),
            authors: vec![Author {
                name: "Test Author".to_string(),
                sort_as: None,
                role: None,
                links: vec![],
            }],
            subjects: vec![],
            description: None,
            publisher: None,
            modified: time::OffsetDateTime::now_utc(),
            published: None,
            translator: None,
            narrator: None,
            illustrator: None,
            editor: None,
            belongs_to: None,
        },
        links: vec![Link::new(
            "/book.epub",
            "application/epub+zip",
            vec!["http://opds-spec.org/acquisition".to_string()],
        )],
        images: vec![],
    };

    let json = serde_json::to_string(&pub_).unwrap();
    let parsed: Publication = serde_json::from_str(&json).unwrap();

    assert_eq!(pub_.metadata.title, parsed.metadata.title);
    assert_eq!(pub_.links[0].href, parsed.links[0].href);
    assert_eq!(pub_.metadata.authors, parsed.metadata.authors);
}

#[test]
fn authentication_document_round_trip() {
    use std::collections::BTreeMap;

    let mut labels = BTreeMap::new();
    labels.insert("login".to_string(), "Username".to_string());
    labels.insert("password".to_string(), "Password".to_string());

    let doc = AuthenticationDocument {
        id: "http://localhost:3121/authentication".to_string(),
        title: "Livtet Catalog".to_string(),
        description: "Enter your Livtet credentials".to_string(),
        authentication: vec![AuthenticationMethod {
            type_: "http://opds-spec.org/auth/basic".to_string(),
            labels,
        }],
    };

    let json = serde_json::to_string(&doc).unwrap();
    let parsed: AuthenticationDocument = serde_json::from_str(&json).unwrap();

    assert_eq!(doc, parsed);
    assert_eq!(parsed.authentication.len(), 1);
    assert_eq!(
        parsed.authentication[0]
            .labels
            .get("login")
            .map(String::as_str),
        Some("Username")
    );
}
