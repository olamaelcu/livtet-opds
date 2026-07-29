//! Atom (OPDS 1) renderer.

use livtet_opds_atom::{FeedKind, render_feed as render_feed_atom};
use livtet_opds_types::{Feed, OpdsError};
use poem::{Response, http::StatusCode};

pub fn render_feed(feed: &Feed, kind: FeedKind) -> Result<Response, OpdsError> {
    let xml = render_feed_atom(feed, kind)?;
    Ok(Response::builder()
        .content_type("application/atom+xml;profile=opds-catalog")
        .status(StatusCode::OK)
        .body(xml))
}
