//! Parse the Feedbooks OPDS 1.x test fixtures.

mod common;

use livtet_opds_atom::parse_feed;

#[test]
fn parse_feedbooks_root() {
    let body = common::fixture("catalog/root.xml");
    let feed = parse_feed(&body).expect("parse root");

    assert_eq!(feed.metadata.title, "Test Catalog Root");
    assert!(
        feed.links
            .iter()
            .any(|l| l.rel.contains(&"self".to_string())),
        "expected self link"
    );
    assert!(
        feed.navigation
            .iter()
            .any(|c| c.metadata.title == "First Acquisition feed"),
        "expected navigation collection"
    );
}

#[test]
fn parse_feedbooks_acquisition_feed() {
    let body = common::fixture("catalog/acquisition/main.xml");
    let feed = parse_feed(&body).expect("parse acquisition feed");

    assert_eq!(feed.metadata.title, "First Acquisition Feed");
    assert!(!feed.publications.is_empty(), "expected publications");

    let pride = feed
        .publications
        .iter()
        .find(|p| p.metadata.title == "Pride and Prejudice")
        .expect("Pride and Prejudice");
    assert_eq!(
        pride.metadata.authors.first().map(|a| a.name.as_str()),
        Some("Jane Austen")
    );
    assert!(
        pride
            .links
            .iter()
            .any(|l| l.rel.contains(&"acquisition".to_string())),
        "expected acquisition link"
    );
    assert!(
        pride
            .images
            .iter()
            .any(|l| l.rel.contains(&"image".to_string())),
        "expected image link"
    );
}
