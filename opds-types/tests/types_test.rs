use livtet_opds_types::{link::*, publication::*};

#[test]
fn publication_metadata_minimal_round_trip() {
    let metadata = PublicationMetadata {
        title: "Minimal Book".to_string(),
        identifier: None,
        description: None,
        language: None,
        publisher: None,
        modified: Some(time::OffsetDateTime::now_utc()),
        published: None,
        authors: vec![],
        subjects: vec![],
        translator: None,
        narrator: None,
        illustrator: None,
        editor: None,
        belongs_to: None,
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let parsed: PublicationMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(metadata.title, parsed.title);
    assert!(parsed.identifier.is_none());
    assert!(parsed.belongs_to.is_none());
    assert!(parsed.authors.is_empty());
    assert!(parsed.subjects.is_empty());
}

#[test]
fn publication_minimal_round_trip() {
    let publication = Publication {
        metadata: PublicationMetadata {
            title: "Minimal Publication".to_string(),
            identifier: None,
            description: None,
            language: None,
            publisher: None,
            modified: Some(time::OffsetDateTime::now_utc()),
            published: None,
            authors: vec![],
            subjects: vec![],
            translator: None,
            narrator: None,
            illustrator: None,
            editor: None,
            belongs_to: None,
        },
        links: vec![],
        images: vec![],
    };

    let json = serde_json::to_string(&publication).unwrap();
    let parsed: Publication = serde_json::from_str(&json).unwrap();

    assert_eq!(publication.metadata.title, parsed.metadata.title);
    assert!(parsed.links.is_empty());
    assert!(parsed.images.is_empty());
}

#[test]
fn link_all_fields_round_trip() {
    let link = Link {
        href: "http://example.com/book".to_string(),
        type_: "application/atom+xml".to_string(),
        templated: false,
        rel: vec!["self".to_string()],
        title: Some("A Book Link".to_string()),
        hreflang: Some("en".to_string()),
        length: Some(12345),
        properties: Some(Properties {
            number_of_items: Some(42),
            price: Some(Price {
                currency: "USD".to_string(),
                value: 12.99,
            }),
            indirect_acquisition: Some(vec![Acquisition {
                type_: "application/epub+zip".to_string(),
                child: vec![Acquisition {
                    type_: "application/zip".to_string(),
                    child: vec![],
                }],
            }]),
            availability: None,
        }),
    };

    let json = serde_json::to_string(&link).unwrap();
    let parsed: Link = serde_json::from_str(&json).unwrap();

    assert_eq!(link.href, parsed.href);
    assert_eq!(link.hreflang, parsed.hreflang);
    assert_eq!(link.length, parsed.length);
    assert_eq!(link.properties.as_ref().unwrap().number_of_items, Some(42));
}

#[test]
fn link_minimal_round_trip() {
    let link = Link::new(
        "http://example.com".to_string(),
        "text/html".to_string(),
        vec![],
    );

    let json = serde_json::to_string(&link).unwrap();
    let parsed: Link = serde_json::from_str(&json).unwrap();

    assert_eq!(link.href, parsed.href);
    assert!(parsed.properties.is_none());
    assert!(parsed.title.is_none());
    assert!(parsed.hreflang.is_none());
    assert!(parsed.length.is_none());
}

#[test]
fn properties_construction_round_trip() {
    let price = Price {
        currency: "USD".to_string(),
        value: 9.99,
    };

    let properties = Properties {
        number_of_items: Some(100),
        price: Some(price),
        indirect_acquisition: Some(vec![Acquisition {
            type_: "epub".to_string(),
            child: vec![],
        }]),
        availability: Some(Availability {
            state: "available".to_string(),
            since: Some(time::OffsetDateTime::now_utc()),
            until: None,
        }),
    };

    let json = serde_json::to_string(&properties).unwrap();
    let parsed: Properties = serde_json::from_str(&json).unwrap();

    assert_eq!(properties.number_of_items, parsed.number_of_items);
    assert_eq!(
        properties.price.as_ref().unwrap().currency,
        parsed.price.as_ref().unwrap().currency
    );
}

#[test]
fn price_round_trip() {
    let price = Price {
        currency: "EUR".to_string(),
        value: 14.99,
    };

    let json = serde_json::to_string(&price).unwrap();
    let parsed: Price = serde_json::from_str(&json).unwrap();

    assert_eq!(price.currency, parsed.currency);
    assert_eq!(price.value, parsed.value);
}

#[test]
fn acquisition_round_trip() {
    let acquisition = Acquisition {
        type_: "epub".to_string(),
        child: vec![],
    };

    let json = serde_json::to_string(&acquisition).unwrap();
    let parsed: Acquisition = serde_json::from_str(&json).unwrap();

    assert_eq!(acquisition.type_, parsed.type_);
    assert!(parsed.child.is_empty());
}

#[test]
fn acquisition_with_recursive_children_round_trip() {
    let acquisition = Acquisition {
        type_: "application/zip".to_string(),
        child: vec![
            Acquisition {
                type_: "application/epub+zip".to_string(),
                child: vec![],
            },
            Acquisition {
                type_: "application/pdf".to_string(),
                child: vec![],
            },
        ],
    };

    let json = serde_json::to_string(&acquisition).unwrap();
    let parsed: Acquisition = serde_json::from_str(&json).unwrap();

    assert_eq!(acquisition.type_, parsed.type_);
    assert_eq!(acquisition.child.len(), parsed.child.len());
    assert_eq!(acquisition.child[0].type_, parsed.child[0].type_);
    assert_eq!(acquisition.child[1].type_, parsed.child[1].type_);
}

#[test]
fn availability_round_trip() {
    let availability = Availability {
        state: "available".to_string(),
        since: Some(time::OffsetDateTime::now_utc()),
        until: Some(time::OffsetDateTime::now_utc()),
    };

    let json = serde_json::to_string(&availability).unwrap();
    let parsed: Availability = serde_json::from_str(&json).unwrap();

    assert_eq!(availability.state, parsed.state);
}

#[test]
fn series_belongs_to_round_trip() {
    let series = Series {
        name: "Test Series".to_string(),
        position: Some(1.0),
    };

    let belongs_to = BelongsTo {
        series: vec![series],
        collections: vec![],
    };

    let json = serde_json::to_string(&belongs_to).unwrap();
    let parsed: BelongsTo = serde_json::from_str(&json).unwrap();

    assert_eq!(
        belongs_to.series.first().unwrap().name,
        parsed.series.first().unwrap().name
    );
    assert!(
        (belongs_to.series.first().unwrap().position.unwrap()
            - parsed.series.first().unwrap().position.unwrap())
        .abs()
            < f32::EPSILON
    );
}

#[test]
fn series_no_position_round_trip() {
    let series = Series {
        name: "Untracked Position Series".to_string(),
        position: None,
    };

    let belongs_to = BelongsTo {
        series: vec![series],
        collections: vec![],
    };

    let json = serde_json::to_string(&belongs_to).unwrap();
    let parsed: BelongsTo = serde_json::from_str(&json).unwrap();

    assert_eq!(
        belongs_to.series.first().unwrap().name,
        parsed.series.first().unwrap().name
    );
    assert!(parsed.series.first().unwrap().position.is_none());
}

#[test]
fn author_round_trip() {
    let author = Author {
        name: "Jane Doe".to_string(),
        sort_as: Some("Doe, Jane".to_string()),
        role: Some("author".to_string()),
        links: vec![],
    };

    let json = serde_json::to_string(&author).unwrap();
    let parsed: Author = serde_json::from_str(&json).unwrap();

    assert_eq!(author, parsed);
}

#[test]
fn author_minimal_omits_optional_fields() {
    let author = Author {
        name: "Jane Doe".to_string(),
        sort_as: None,
        role: None,
        links: vec![],
    };

    let json = serde_json::to_string(&author).unwrap();
    assert!(!json.contains("sort_as"));
    assert!(!json.contains("role"));
    assert!(!json.contains("links"));
}

#[test]
fn flat_authors_returns_comma_separated_names() {
    let metadata = PublicationMetadata {
        title: "Multi".to_string(),
        identifier: None,
        description: None,
        language: None,
        publisher: None,
        modified: Some(time::OffsetDateTime::now_utc()),
        published: None,
        authors: vec![
            Author {
                name: "Alice".to_string(),
                sort_as: None,
                role: None,
                links: vec![],
            },
            Author {
                name: "Bob".to_string(),
                sort_as: None,
                role: None,
                links: vec![],
            },
        ],
        subjects: vec![],
        translator: None,
        narrator: None,
        illustrator: None,
        editor: None,
        belongs_to: None,
    };

    assert_eq!(metadata.flat_authors(), "Alice, Bob");
}

#[test]
fn flat_authors_empty_when_no_authors() {
    let metadata = PublicationMetadata {
        title: "Solo".to_string(),
        identifier: None,
        description: None,
        language: None,
        publisher: None,
        modified: Some(time::OffsetDateTime::now_utc()),
        published: None,
        authors: vec![],
        subjects: vec![],
        translator: None,
        narrator: None,
        illustrator: None,
        editor: None,
        belongs_to: None,
    };

    assert_eq!(metadata.flat_authors(), "");
}

#[test]
fn publication_metadata_all_fields_round_trip() {
    let metadata = PublicationMetadata {
        title: "Complete Test Book".to_string(),
        identifier: Some("urn:isbn:978-0-123456-78-9".to_string()),
        description: Some("A comprehensive test book with all fields".to_string()),
        language: Some("en".to_string()),
        publisher: Some("Test Publisher".to_string()),
        modified: Some(time::OffsetDateTime::now_utc()),
        published: Some(time::OffsetDateTime::now_utc()),
        authors: vec![Author {
            name: "Test Author".to_string(),
            sort_as: None,
            role: None,
            links: vec![],
        }],
        subjects: vec!["FICTION".to_string(), "MYSTERY".to_string()],
        translator: Some("Test Translator".to_string()),
        narrator: Some("Test Narrator".to_string()),
        illustrator: Some("Test Illustrator".to_string()),
        editor: Some("Test Editor".to_string()),
        belongs_to: Some(BelongsTo {
            series: vec![Series {
                name: "Test Series".to_string(),
                position: Some(2.0),
            }],
            collections: vec![],
        }),
    };

    let json = serde_json::to_string(&metadata).unwrap();
    let parsed: PublicationMetadata = serde_json::from_str(&json).unwrap();

    assert_eq!(metadata.title, parsed.title);
    assert_eq!(metadata.identifier, parsed.identifier);
    assert_eq!(metadata.language, parsed.language);
    assert_eq!(metadata.publisher, parsed.publisher);
    assert_eq!(metadata.authors, parsed.authors);
    assert_eq!(metadata.subjects, parsed.subjects);
    assert_eq!(metadata.translator, parsed.translator);
    assert_eq!(metadata.belongs_to, parsed.belongs_to);
}

#[test]
fn templated_link() {
    let link = Link {
        href: "http://example.com/search{?query}".to_string(),
        type_: "application/opds+json".to_string(),
        templated: true,
        rel: vec!["search".to_string()],
        title: None,
        hreflang: None,
        length: None,
        properties: None,
    };

    assert!(link.is_search());
    assert!(link.templated);

    let json = serde_json::to_string(&link).unwrap();
    let parsed: Link = serde_json::from_str(&json).unwrap();

    assert!(parsed.templated);
    assert!(parsed.is_search());
}

#[test]
fn link_rel_accepts_single_string() {
    // Archive.org and several OPDS 2 / Readium feeds emit `rel` as a
    // single string rather than a one-element array.
    let json = r#"{
        "href": "/search{?query}",
        "type": "application/opds+json",
        "rel": "search",
        "templated": true
    }"#;

    let parsed: Link = serde_json::from_str(json).unwrap();
    assert_eq!(parsed.rel, vec!["search".to_string()]);
    assert!(parsed.is_search());
}

#[test]
fn link_rel_still_accepts_array_of_strings() {
    let json = r#"{
        "href": "/book.epub",
        "type": "application/epub+zip",
        "rel": ["http://opds-spec.org/acquisition", "open-access"]
    }"#;

    let parsed: Link = serde_json::from_str(json).unwrap();
    assert_eq!(
        parsed.rel,
        vec![
            "http://opds-spec.org/acquisition".to_string(),
            "open-access".to_string(),
        ]
    );
}
