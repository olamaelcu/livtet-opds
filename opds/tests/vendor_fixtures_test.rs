mod common;

use roxmltree::Document;

const ATOM_NS: &str = "http://www.w3.org/2005/Atom";

#[test]
fn root_catalog_parses_as_atom_feed() {
    let xml = common::fixture("catalog/root.xml");
    let doc = Document::parse(&xml).expect("root.xml should parse as XML");
    let root = doc.root_element();
    assert_eq!(root.tag_name().name(), "feed");
    assert!(
        root.has_tag_name((ATOM_NS, "feed")),
        "root element must be <feed> in the Atom namespace, got {:?}",
        root.tag_name()
    );
    let entry_count = root
        .children()
        .filter(|n| n.is_element() && n.has_tag_name((ATOM_NS, "entry")))
        .count();
    assert!(
        entry_count >= 1,
        "root catalog must contain at least one <entry>, found {entry_count}"
    );
}

#[test]
fn root_catalog_title_matches_expected_seed() {
    let xml = common::fixture("catalog/root.xml");
    let doc = Document::parse(&xml).expect("root.xml should parse as XML");
    let title = doc
        .root_element()
        .children()
        .find(|n| n.is_element() && n.has_tag_name((ATOM_NS, "title")))
        .expect("root.xml must have a <title>");
    assert_eq!(title.text(), Some("Test Catalog Root"));
}
