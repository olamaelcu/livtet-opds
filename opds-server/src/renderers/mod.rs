//! Renderers for OPDS responses.

pub mod atom;
pub mod json;
pub mod opensearch;

pub use livtet_opds_atom::FeedKind;
use livtet_opds_types::{Feed, OpdsError};
use poem::Response;

use crate::content_negotiation::Renderer;

/// Render an OPDS feed into a Poem response.
pub fn render_feed(feed: &Feed, renderer: Renderer, kind: FeedKind) -> Result<Response, OpdsError> {
    match renderer {
        Renderer::Json => json::render_feed(feed),
        Renderer::Atom => atom::render_feed(feed, kind),
    }
}

/// Render the OpenSearch description document.
pub fn render_opensearch(template: &str) -> Result<Response, OpdsError> {
    opensearch::render(template)
}
