use std::sync::Arc;

use livtet_opds_client::{Client, PaginationManager};
use livtet_opds_types::{Feed, Link, Metadata};

fn feed_with_pagination(
    total: Option<u64>,
    per_page: Option<u64>,
    next_href: Option<&str>,
) -> String {
    let mut links = vec![Link::new(
        "/feed",
        "application/opds+json",
        vec!["self".into()],
    )];
    if let Some(href) = next_href {
        links.push(Link::new(
            href,
            "application/opds+json",
            vec!["next".into()],
        ));
    }
    let feed = Feed {
        metadata: Metadata {
            title: "Paged".into(),
            identifier: None,
            type_: None,
            description: None,
            modified: None,
            number_of_items: total,
            items_per_page: per_page,
            current_page: None,
        },
        links,
        navigation: vec![],
        publications: vec![],
        facets: vec![],
        groups: vec![],
    };
    serde_json::to_string(&feed).unwrap()
}

async fn setup_pagination_test(
    total: Option<u64>,
    per_page: Option<u64>,
    next_href: Option<&str>,
) -> (mockito::ServerGuard, PaginationManager) {
    let body = feed_with_pagination(total, per_page, next_href);
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/feed")
        .with_status(200)
        .with_header("Content-Type", "application/opds+json")
        .with_body(&body)
        .create_async()
        .await;

    let client = Arc::new(Client::new(server.url()).unwrap());
    let pm = PaginationManager::start(client, "/feed").await.unwrap();
    (server, pm)
}

#[tokio::test]
async fn total_pages_exact_division() {
    let (_server, pm) = setup_pagination_test(Some(100), Some(10), None).await;
    assert_eq!(pm.total_pages(), Some(10));
}

#[tokio::test]
async fn total_pages_ceiling_division() {
    let (_server, pm) = setup_pagination_test(Some(101), Some(10), None).await;
    assert_eq!(pm.total_pages(), Some(11));
}

#[tokio::test]
async fn total_pages_zero_items() {
    let (_server, pm) = setup_pagination_test(Some(0), Some(10), None).await;
    assert_eq!(pm.total_pages(), Some(0));
}

#[tokio::test]
async fn no_total_means_unknown_and_has_next_true() {
    let (_server, pm) = setup_pagination_test(None, Some(10), None).await;
    assert_eq!(pm.total_pages(), None);
    assert!(pm.has_next());
}

#[tokio::test]
async fn current_page_starts_at_one() {
    let (_server, pm) = setup_pagination_test(Some(50), Some(10), None).await;
    assert_eq!(pm.current_page_number(), 1);
}

#[tokio::test]
async fn next_with_templated_link() {
    let body = feed_with_pagination(Some(20), Some(10), Some("http://localhost/feed?page=2"));
    let body2 = feed_with_pagination(Some(20), Some(10), None);
    let mut server = mockito::Server::new_async().await;
    let _m1 = server
        .mock("GET", "/feed")
        .with_status(200)
        .with_header("Content-Type", "application/opds+json")
        .with_body(&body)
        .expect(2)
        .create_async()
        .await;
    let _m2 = server
        .mock("GET", "/")
        .with_status(200)
        .with_header("Content-Type", "application/opds+json")
        .with_body(&body2)
        .create_async()
        .await;

    let client = Arc::new(Client::new(server.url() + "/feed").unwrap());
    let mut pm = PaginationManager::start(client.clone(), "/feed")
        .await
        .unwrap();

    let feed = pm.next().await.unwrap();
    assert_eq!(pm.current_page_number(), 2);
    assert_eq!(feed.metadata.title, "Paged");
}

#[tokio::test]
async fn next_with_malformed_link_does_not_panic() {
    let body = feed_with_pagination(None, None, Some("not a url {{{"));
    let mut server = mockito::Server::new_async().await;
    let _mock = server
        .mock("GET", "/feed")
        .with_status(200)
        .with_header("Content-Type", "application/opds+json")
        .with_body(&body)
        .expect(2)
        .create_async()
        .await;

    let client = Arc::new(Client::new(server.url()).unwrap());
    let mut pm = PaginationManager::start(client, "/feed").await.unwrap();

    let result = pm.next().await;
    assert!(result.is_ok(), "malformed next link should not panic");
}
