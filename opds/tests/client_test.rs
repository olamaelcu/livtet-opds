use livtet_opds_client::Client;
use livtet_opds_types::{Feed, Link, Metadata};

fn minimal_feed_json() -> String {
    let feed = Feed {
        metadata: Metadata {
            title: "Test Catalog".into(),
            identifier: None,
            type_: None,
            description: None,
            modified: None,
            number_of_items: None,
            items_per_page: None,
            current_page: None,
        },
        links: vec![Link::new(
            "/feed.json",
            "application/opds+json",
            vec!["self".into()],
        )],
        navigation: vec![],
        publications: vec![],
        facets: vec![],
        groups: vec![],
    };
    serde_json::to_string(&feed).unwrap()
}

#[test]
fn new_rejects_malformed_url() {
    let result = Client::new("not a url at all://  ");
    assert!(result.is_err(), "malformed URL should be rejected");
}

#[test]
fn new_accepts_valid_http_url() {
    let result = Client::new("http://localhost:8080/opds");
    assert!(result.is_ok(), "valid HTTP URL should be accepted");
}

#[tokio::test]
async fn fetch_feed_happy_path() {
    let mut server = mockito::Server::new_async().await;
    let body = minimal_feed_json();

    let mock = server
        .mock("GET", "/opds/feed")
        .match_header(
            "Accept",
            "application/opds+json, application/atom+xml;profile=opds-catalog;q=0.9, */*;q=0.1",
        )
        .with_status(200)
        .with_header("Content-Type", "application/opds+json")
        .with_body(&body)
        .create_async()
        .await;

    let client = Client::new(server.url()).unwrap();
    let feed = client.fetch_feed("/opds/feed").await.unwrap();

    assert_eq!(feed.metadata.title, "Test Catalog");
    mock.assert_async().await;
}

#[tokio::test]
async fn fetch_feed_404_returns_http_error() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/missing")
        .with_status(404)
        .create_async()
        .await;

    let client = Client::new(server.url()).unwrap();
    let err = client.fetch_feed("/missing").await.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("404"),
        "expected HTTP 404 in error, got: {msg}"
    );
}

#[tokio::test]
async fn fetch_feed_500_returns_http_error() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/broken")
        .with_status(500)
        .create_async()
        .await;

    let client = Client::new(server.url()).unwrap();
    let err = client.fetch_feed("/broken").await.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("500"),
        "expected HTTP 500 in error, got: {msg}"
    );
}

#[tokio::test]
async fn fetch_feed_malformed_json_returns_error() {
    let mut server = mockito::Server::new_async().await;

    let _mock = server
        .mock("GET", "/bad-json")
        .with_status(200)
        .with_header("Content-Type", "application/opds+json")
        .with_body("this is not json at all")
        .create_async()
        .await;

    let client = Client::new(server.url()).unwrap();
    let result = client.fetch_feed("/bad-json").await;
    assert!(result.is_err(), "malformed JSON should produce an error");
}

#[test]
fn base_url_join_normalizes_percent_encoded_dot_dot() {
    let base = url::Url::parse("http://localhost:8080/opds/").unwrap();
    let joined = base.join("%2e%2e/etc/passwd").unwrap();
    let path = joined.path();
    assert!(
        !path.contains(".."),
        "expected path traversal to be normalized, got path: {path}"
    );
    assert_eq!(path, "/etc/passwd");
}

#[tokio::test]
async fn fetch_feed_accepts_string_rel_values() {
    // Regression: some OPDS 2 feeds (e.g. Archive.org) emit `rel` as a
    // single string instead of a one-element array.
    let body = r#"{
        "metadata": { "title": "Archive.org" },
        "links": [
            {
                "href": "https://example.com/search{?query}",
                "type": "application/opds+json",
                "rel": "search",
                "templated": true
            }
        ]
    }"#;

    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/opds/")
        .with_status(200)
        .with_header("Content-Type", "application/opds+json")
        .with_body(body)
        .create_async()
        .await;

    let client = Client::new(server.url()).unwrap();
    let feed = client.fetch_feed("/opds/").await.unwrap();

    assert_eq!(feed.metadata.title, "Archive.org");
    let search_link = feed.links.iter().find(|l| l.is_search()).unwrap();
    assert_eq!(search_link.rel, vec!["search".to_string()]);
}
