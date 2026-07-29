use std::collections::BTreeMap;

use livtet_opds_types::{AuthenticationDocument, AuthenticationMethod, OpdsError};
use poem::{
    EndpointExt, Request, Response, Route,
    endpoint::BoxEndpoint,
    get, handler,
    http::StatusCode,
    middleware::AddData,
    web::{Data, Json, Path, Query},
};
use serde::Deserialize;

use crate::{
    SharedOpdsHandler, auth,
    content_negotiation::{self, Renderer},
    renderers::{self, FeedKind},
};

#[derive(Debug, Deserialize)]
pub struct SearchParams {
    pub query: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadParams {
    /// Optional `digital_inventory.id` to pin the selection. When
    /// `None`, the server picks the first edition with a linked file
    /// (matches the OPDS feed's acquisition-link ordering).
    pub inventory_id: Option<String>,
}

fn _error_response(status: StatusCode, message: &str) -> Response {
    Response::builder()
        .content_type("application/json")
        .status(status)
        .body(format!(
            "{{\"error\":\"{}\"}}",
            message.replace('"', "\\\"")
        ))
}

#[handler]
pub async fn download_publication(
    handler: Data<&SharedOpdsHandler>,
    Path(work_id): Path<String>,
    Query(params): Query<DownloadParams>,
) -> Result<Response, poem::Error> {
    match handler
        .get_linked_file(&work_id, params.inventory_id.as_deref())
        .await
    {
        Ok((bytes, mime_type, filename)) => {
            // Trigger a download in browsers via the `attachment`
            // disposition. The filename is quoted to survive spaces
            // and most punctuation; we keep it conservative (no
            // RFC 5987 encoding) because OPDS clients expect a
            // plain token.
            let disposition = format!("attachment; filename=\"{filename}\"");
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", mime_type)
                .header("Content-Disposition", disposition)
                .header("Content-Length", bytes.len().to_string())
                .body(bytes))
        }
        Err(OpdsError::MissingField(_)) => Err(poem::Error::from_string(
            "{\"error\":\"no linked file for this work\"}".to_string(),
            StatusCode::NOT_FOUND,
        )),
        Err(e) => Err(poem::Error::from_string(
            format!("{{\"error\":\"{}\"}}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

fn renderer_from_request(req: &Request) -> Renderer {
    let accept = req
        .headers()
        .get(poem::http::header::ACCEPT)
        .and_then(|v| v.to_str().ok());
    content_negotiation::pick_renderer(accept)
}

#[handler]
pub async fn get_root_feed(
    handler: Data<&SharedOpdsHandler>,
    req: &Request,
) -> Result<Response, poem::Error> {
    let renderer = renderer_from_request(req);
    match handler.get_root_feed().await {
        Ok(feed) => render_feed_or_error(&feed, renderer, FeedKind::Navigation),
        Err(e) => Err(poem::Error::from_string(
            format!("{{\"error\":\"{}\"}}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

#[handler]
pub async fn get_publication(
    handler: Data<&SharedOpdsHandler>,
    Path(id): Path<String>,
) -> Result<Json<livtet_opds_types::Publication>, poem::Error> {
    match handler.get_publication(&id).await {
        Ok(pub_) => Ok(Json(pub_)),
        Err(OpdsError::MissingField(_)) => Err(poem::Error::from_string(
            "{\"error\":\"not found\"}".to_string(),
            StatusCode::NOT_FOUND,
        )),
        Err(e) => Err(poem::Error::from_string(
            format!("{{\"error\":\"{}\"}}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

#[handler]
pub async fn search(
    handler: Data<&SharedOpdsHandler>,
    Query(params): Query<SearchParams>,
    req: &Request,
) -> Result<Response, poem::Error> {
    let renderer = renderer_from_request(req);
    let query = params.query.unwrap_or_default();
    match handler.search(&query).await {
        Ok(feed) => render_feed_or_error(&feed, renderer, FeedKind::SearchResult),
        Err(e) => Err(poem::Error::from_string(
            format!("{{\"error\":\"{}\"}}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

#[handler]
pub async fn get_navigation_authors(
    handler: Data<&SharedOpdsHandler>,
    req: &Request,
) -> Result<Response, poem::Error> {
    let renderer = renderer_from_request(req);
    match handler.get_navigation_authors().await {
        Ok(feed) => render_feed_or_error(&feed, renderer, FeedKind::Navigation),
        Err(e) => Err(poem::Error::from_string(
            format!("{{\"error\":\"{}\"}}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

#[handler]
pub async fn get_navigation_tags(
    handler: Data<&SharedOpdsHandler>,
    req: &Request,
) -> Result<Response, poem::Error> {
    let renderer = renderer_from_request(req);
    match handler.get_navigation_tags().await {
        Ok(feed) => render_feed_or_error(&feed, renderer, FeedKind::Navigation),
        Err(e) => Err(poem::Error::from_string(
            format!("{{\"error\":\"{}\"}}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

#[handler]
pub async fn get_navigation_genres(
    handler: Data<&SharedOpdsHandler>,
    req: &Request,
) -> Result<Response, poem::Error> {
    let renderer = renderer_from_request(req);
    match handler.get_navigation_genres().await {
        Ok(feed) => render_feed_or_error(&feed, renderer, FeedKind::Navigation),
        Err(e) => Err(poem::Error::from_string(
            format!("{{\"error\":\"{}\"}}", e),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

#[handler]
pub async fn search_description(req: &Request) -> Result<Response, poem::Error> {
    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:3121");
    let template = format!("http://{host}/search?q={{searchTerms}}");
    renderers::render_opensearch(&template)
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))
}

fn render_feed_or_error(
    feed: &livtet_opds_types::Feed,
    renderer: Renderer,
    kind: FeedKind,
) -> Result<Response, poem::Error> {
    renderers::render_feed(feed, renderer, kind)
        .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))
}

/// `GET /authentication` — returns the OPDS Authentication Document.
///
/// This endpoint is always served without authentication so that OPDS
/// clients can discover the supported auth mechanisms before making
/// authenticated requests.
#[handler]
pub async fn get_authentication_doc(req: &Request) -> Json<AuthenticationDocument> {
    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("localhost:3121");

    let mut labels = BTreeMap::new();
    labels.insert("login".to_string(), "Username".to_string());
    labels.insert("password".to_string(), "Password".to_string());

    Json(AuthenticationDocument {
        id: format!("http://{host}/authentication"),
        title: "Livtet Catalog".to_string(),
        description: "Enter your Livtet credentials".to_string(),
        authentication: vec![AuthenticationMethod {
            type_: "http://opds-spec.org/auth/basic".to_string(),
            labels,
        }],
    })
}

/// Build the OPDS route table.
///
/// When `auth_config.require_auth == true`, the route table is wrapped
/// in `AuthMiddleware` which enforces Basic authentication on all
/// endpoints except `GET /authentication`.
///
/// The return type is `BoxEndpoint<'static, Response>` because the
/// `Endpoint` trait isn't object-safe — we use the object-safe
/// `DynEndpoint` trait (via `EndpointExt::boxed()`) to type-erase
/// either branch. The vtable dispatch cost is negligible for an
/// OPDS server (LAN scale).
pub fn make_routes(
    handler: SharedOpdsHandler,
    auth_config: Option<crate::auth::AuthConfig>,
) -> BoxEndpoint<'static, Response> {
    let route = Route::new()
        .at("/", get(get_root_feed))
        .at("/publications/:id", get(get_publication))
        .at("/publications/:work_id/download", get(download_publication))
        .at("/search", get(search))
        .at("/search.xml", get(search_description))
        .at("/navigation/authors", get(get_navigation_authors))
        .at("/navigation/tags", get(get_navigation_tags))
        .at("/navigation/genres", get(get_navigation_genres))
        .at("/authentication", get(get_authentication_doc))
        .with(AddData::new(handler));

    if let Some(config) = auth_config.filter(|c| c.require_auth) {
        route.with(auth::AuthMiddleware::new(config)).boxed()
    } else {
        route.boxed()
    }
}
