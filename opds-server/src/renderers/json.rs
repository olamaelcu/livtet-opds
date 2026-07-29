//! JSON (OPDS 2) renderer.

use livtet_opds_types::{Feed, OpdsError};
use poem::{Response, http::StatusCode};

pub fn render_feed(feed: &Feed) -> Result<Response, OpdsError> {
    let body = serde_json::to_vec(feed).map_err(OpdsError::from)?;
    Ok(Response::builder()
        .content_type("application/opds+json")
        .status(StatusCode::OK)
        .body(body))
}
