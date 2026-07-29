use async_trait::async_trait;
use livtet_opds_types::{Feed, OpdsError, Publication};

#[async_trait]
pub trait OpdsHandler: Send + Sync {
    async fn get_root_feed(&self) -> Result<Feed, OpdsError>;

    async fn get_publication(&self, id: &str) -> Result<Publication, OpdsError>;

    async fn search(&self, query: &str) -> Result<Feed, OpdsError>;

    async fn get_navigation_authors(&self) -> Result<Feed, OpdsError>;

    async fn get_navigation_tags(&self) -> Result<Feed, OpdsError>;

    async fn get_navigation_genres(&self) -> Result<Feed, OpdsError>;

    /// Resolve and return the bytes for a work's linked file.
    ///
    /// Returns `(bytes, mime_type, filename)` for the OPDS download
    /// endpoint. `inventory_id` is `None` when the client didn't pick
    /// a specific edition; the handler picks the first available
    /// linked file in that case. See [`crate::repository::OpdsRepository::get_linked_file`]
    /// for the precise selection rules and error semantics.
    async fn get_linked_file(
        &self,
        work_id: &str,
        inventory_id: Option<&str>,
    ) -> Result<(Vec<u8>, String, String), OpdsError>;
}

#[async_trait]
pub trait OpdsServer: Send + Sync {
    async fn start(&mut self, addr: std::net::SocketAddr) -> Result<(), OpdsError>;
    async fn stop(&mut self) -> Result<(), OpdsError>;
    fn is_running(&self) -> bool;
}
