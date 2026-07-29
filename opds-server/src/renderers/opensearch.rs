//! OpenSearch description document renderer.

use livtet_opds_atom::render_description;
use livtet_opds_types::OpdsError;
use poem::{Response, http::StatusCode};

const SHORT_NAME: &str = "Livtet Catalog";
const DESCRIPTION: &str = "Search the Livtet OPDS catalog";

pub fn render(template: &str) -> Result<Response, OpdsError> {
    let xml = render_description(template, SHORT_NAME, DESCRIPTION);
    Ok(Response::builder()
        .content_type("application/opensearchdescription+xml")
        .status(StatusCode::OK)
        .body(xml))
}
