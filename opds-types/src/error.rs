use thiserror::Error;

#[derive(Debug, Error)]
pub enum OpdsError {
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Invalid link relation: {0}")]
    InvalidRelation(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("HTTP error: {0}")]
    Http(u16),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Search index error: {0}")]
    Search(String),

    #[error("XML parse error: {0}")]
    Xml(String),

    #[error("Unsupported response format: {0}")]
    UnsupportedFormat(String),
}

#[cfg(feature = "fake")]
impl fake::Dummy<fake::Faker> for OpdsError {
    fn dummy_with_rng<R: fake::RngExt + ?Sized>(_: &fake::Faker, _: &mut R) -> Self {
        // Return a simple variant that doesn't require Dummy for external types.
        OpdsError::MissingField("fake".to_string())
    }
}
