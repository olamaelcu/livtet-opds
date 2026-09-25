//! Deserialization of real-world OPDS 2.0 JSON, including the shapes the
//! Internet Archive emits (compact navigation links, omitted `modified`,
//! array `language`, singular `author`).

use livtet_opds_types::Feed;

const ARCHIVE_ROOT: &str = r#"{
  "@context": "https://readium.org/webpub-manifest/context.jsonld",
  "metadata": { "title": "Archive.org" },
  "links": [
    { "href": "https://archive.org/services/opds/", "type": "application/opds+json", "rel": "self" },
    { "href": "https://archive.org/services/opds/catalog{?query}&type=search", "type": "application/opds+json", "rel": "search", "templated": true }
  ],
  "navigation": [
    { "href": "https://archive.org/services/opds/catalog?type=navigation&nav_key=page_ebooks", "title": "eBooks", "type": "application/opds+json", "rel": "collection" },
    { "href": "https://archive.org/services/opds/catalog?type=navigation&nav_key=page_audiobooks", "title": "Audiobooks", "type": "application/opds+json", "rel": "collection" }
  ],
  "groups": [
    {
      "metadata": { "title": "eBooks Open Access & Public Domain", "numberOfItems": 3666506 },
      "links": [ { "href": "https://archive.org/services/opds/catalog?type=browse&section=ebooks&item=open-access", "type": "application/opds+json", "rel": "self" } ]
    }
  ]
}"#;

const ARCHIVE_PUBLICATIONS: &str = r#"{
  "metadata": { "title": "eBooks" },
  "links": [
    { "href": "https://archive.org/services/opds/catalog?type=browse", "type": "application/opds+json", "rel": "self" }
  ],
  "publications": [
    {
      "metadata": {
        "title": "Calendariographias",
        "author": [ { "name": "Christian Gueinz" } ],
        "identifier": "https://archive.org/details/bub_gb_p7oh1TeTzuUC",
        "language": [ "la" ],
        "type": "http://schema.org/Book"
      },
      "links": [
        { "href": "https://archive.org/services/opds/book/bub_gb_p7oh1TeTzuUC?glob_pattern=*pdf", "type": "application/pdf", "rel": "http://opds-spec.org/acquisition/open-access" }
      ],
      "images": [
        { "href": "https://archive.org/download/bub_gb_p7oh1TeTzuUC/__ia_thumb.jpg", "type": "image/jpeg", "rel": "cover" }
      ]
    }
  ]
}"#;

#[test]
fn compact_navigation_links_deserialize() {
    let feed: Feed = serde_json::from_str(ARCHIVE_ROOT).expect("archive root parses");
    assert_eq!(feed.metadata.title, "Archive.org");
    assert_eq!(feed.navigation.len(), 2);
    assert_eq!(feed.navigation[0].title.as_deref(), Some("eBooks"));
    assert_eq!(
        feed.navigation[0].href,
        "https://archive.org/services/opds/catalog?type=navigation&nav_key=page_ebooks"
    );
    assert_eq!(feed.navigation[0].rel, vec!["collection".to_string()]);
    // `groups` remains a collection-of-collections and parses alongside nav.
    assert_eq!(feed.groups.len(), 1);
    assert_eq!(
        feed.groups[0].metadata.title,
        "eBooks Open Access & Public Domain"
    );
}

#[test]
fn publications_tolerate_missing_modified_and_array_language() {
    let feed: Feed = serde_json::from_str(ARCHIVE_PUBLICATIONS).expect("archive feed parses");
    let publication = &feed.publications[0];
    let metadata = &publication.metadata;

    assert_eq!(metadata.title, "Calendariographias");
    assert_eq!(metadata.modified, None);
    assert_eq!(metadata.language.as_deref(), Some("la"));
    assert_eq!(metadata.authors.len(), 1);
    assert_eq!(metadata.authors[0].name, "Christian Gueinz");
    assert_eq!(
        metadata.identifier.as_deref(),
        Some("https://archive.org/details/bub_gb_p7oh1TeTzuUC")
    );
    assert!(
        publication
            .links
            .iter()
            .any(|l| l.rel.iter().any(|r| r.ends_with("/open-access")))
    );
}

#[test]
fn author_accepts_a_bare_string_object_or_array() {
    let as_string =
        r#"{ "metadata": { "title": "Moby-Dick", "author": "Herman Melville" }, "links": [] }"#;
    let feed: Feed = serde_json::from_str(&format!(
        r#"{{ "metadata": {{ "title": "t" }}, "links": [], "publications": [{}] }}"#,
        as_string
    ))
    .expect("string author parses");
    assert_eq!(
        feed.publications[0].metadata.authors[0].name,
        "Herman Melville"
    );

    let as_object = r#"{ "metadata": { "title": "Voyages", "author": { "name": "Jules Verne" } }, "links": [] }"#;
    let feed: Feed = serde_json::from_str(&format!(
        r#"{{ "metadata": {{ "title": "t" }}, "links": [], "publications": [{}] }}"#,
        as_object
    ))
    .expect("object author parses");
    assert_eq!(feed.publications[0].metadata.authors[0].name, "Jules Verne");

    let as_array = r#"{ "metadata": { "title": "Co-authors", "author": [ { "name": "A" }, { "name": "B" } ] }, "links": [] }"#;
    let feed: Feed = serde_json::from_str(&format!(
        r#"{{ "metadata": {{ "title": "t" }}, "links": [], "publications": [{}] }}"#,
        as_array
    ))
    .expect("array author parses");
    assert_eq!(feed.publications[0].metadata.authors.len(), 2);
}
