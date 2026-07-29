pub mod auth;
pub mod client;
pub mod default_catalogs;
pub mod download;
pub mod pagination;
pub mod search;

pub use auth::{ExtraHeaders, OpdsAuth};
pub use client::Client;
pub use default_catalogs::{DefaultCatalog, default_catalogs};
pub use download::{DownloadManager, DownloadStatus};
pub use pagination::PaginationManager;
pub use search::SearchTemplate;
