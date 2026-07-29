use std::sync::Arc;

use livtet_opds_types::{Feed, OpdsError};

use crate::Client;

pub struct PaginationManager {
    client: Arc<Client>,
    current_url: String,
    total_pages: Option<u64>,
    #[allow(dead_code)]
    items_per_page: Option<u64>,
    page_count: u64,
}

impl PaginationManager {
    pub async fn start(client: Arc<Client>, path: &str) -> Result<Self, OpdsError> {
        let feed = client.fetch_feed(path).await?;
        let total_pages = feed
            .metadata
            .number_of_items
            .zip(feed.metadata.items_per_page)
            .map(|(total, per_page)| total.div_ceil(per_page));

        Ok(Self {
            client,
            current_url: path.to_string(),
            total_pages,
            items_per_page: feed.metadata.items_per_page,
            page_count: 1,
        })
    }

    pub fn current_page_number(&self) -> u64 {
        self.page_count
    }

    pub fn total_pages(&self) -> Option<u64> {
        self.total_pages
    }

    pub fn has_next(&self) -> bool {
        self.total_pages
            .map(|t| self.page_count < t)
            .unwrap_or(true)
    }

    pub async fn next(&mut self) -> Result<Feed, OpdsError> {
        let feed = self.client.fetch_feed(&self.current_url).await?;
        self.current_url = feed
            .next_page()
            .map(|l| extract_path(&l.href))
            .unwrap_or_else(|| self.current_url.clone());
        self.page_count += 1;
        Ok(feed)
    }
}

fn extract_path(href: &str) -> String {
    url::Url::parse(href)
        .ok()
        .map(|u| u.path().to_string())
        .unwrap_or_else(|| href.to_string())
}
