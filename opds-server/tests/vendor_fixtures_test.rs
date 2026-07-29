mod common;

use roxmltree::Document;

const ATOM_NS: &str = "http://www.w3.org/2005/Atom";

#[test]
fn acquisition_feed_parses_as_atom_feed() {
    let xml = common::fixture("catalog/acquisition/main.xml");
    let doc = Document::parse(&xml).expect("acquisition/main.xml should parse as XML");
    let root = doc.root_element();
    assert_eq!(root.tag_name().name(), "feed");
    assert!(
        root.has_tag_name((ATOM_NS, "feed")),
        "acquisition root element must be <feed> in the Atom namespace, got {:?}",
        root.tag_name()
    );
    let entry_count = root
        .children()
        .filter(|n| n.is_element() && n.has_tag_name((ATOM_NS, "entry")))
        .count();
    assert!(
        entry_count >= 1,
        "acquisition feed must contain at least one <entry>, found {entry_count}"
    );
}

#[test]
fn acquisition_feed_has_opensearch_total_results() {
    let xml = common::fixture("catalog/acquisition/main.xml");
    let doc = Document::parse(&xml).expect("acquisition/main.xml should parse as XML");
    let total = doc
        .descendants()
        .find(|n| {
            n.is_element()
                && n.has_tag_name(("http://a9.com/-/spec/opensearch/1.1/", "totalResults"))
        })
        .expect("acquisition feed must have an OpenSearch <totalResults> element");
    assert_eq!(total.text(), Some("18"));
}
