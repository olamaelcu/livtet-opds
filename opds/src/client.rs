use livtet_opds_types::{Feed, OpdsError, Publication};
use reqwest::Client as HttpClient;
use url::Url;

use crate::auth::{ExtraHeaders, OpdsAuth};

const ACCEPT_FEED: &str =
    "application/opds+json, application/atom+xml;profile=opds-catalog;q=0.9, */*;q=0.1";

const ACCEPT_PUBLICATION: &str =
    "application/opds-publication+json, application/atom+xml;profile=opds-catalog;q=0.8, */*;q=0.1";

pub struct Client {
    http: HttpClient,
    base_url: Url,
    auth: OpdsAuth,
    extra_headers: ExtraHeaders,
}

impl Client {
    pub fn new(base_url: impl Into<String>) -> Result<Self, OpdsError> {
        Self::with_http(base_url, HttpClient::new())
    }

    /// Build a client with an externally supplied `reqwest::Client`.
    /// Used by desktop and mobile to inject a shared agent carrying a
    /// consistent User-Agent.
    pub fn with_http(base_url: impl Into<String>, http: HttpClient) -> Result<Self, OpdsError> {
        let base_url = Url::parse(&base_url.into())?;
        Ok(Self {
            http,
            base_url,
            auth: OpdsAuth::None,
            extra_headers: ExtraHeaders::new(),
        })
    }

    pub fn auth(mut self, auth: OpdsAuth) -> Self {
        self.auth = auth;
        self
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra_headers.add(name, value);
        self
    }

    pub(crate) fn request(&self, path: &str) -> reqwest::RequestBuilder {
        let url = self.base_url.join(path).expect("relative path is valid");
        let builder = self.http.get(url);
        let builder = self.auth.apply(builder);
        self.extra_headers.apply(builder)
    }

    pub fn base_path(&self) -> &str {
        self.base_url.path()
    }

    pub async fn fetch_feed(&self, path: &str) -> Result<Feed, OpdsError> {
        let response = self
            .request(path)
            .header("Accept", ACCEPT_FEED)
            .send()
            .await
            .map_err(|e| OpdsError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OpdsError::Http(response.status().as_u16()));
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());
        let body = response
            .text()
            .await
            .map_err(|e| OpdsError::Network(e.to_string()))?;

        Self::parse_feed_response(content_type.as_deref(), &body)
    }

    pub async fn fetch_publication(&self, path: &str) -> Result<Publication, OpdsError> {
        let response = self
            .request(path)
            .header("Accept", ACCEPT_PUBLICATION)
            .send()
            .await
            .map_err(|e| OpdsError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(OpdsError::Http(response.status().as_u16()));
        }

        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());
        let body = response
            .text()
            .await
            .map_err(|e| OpdsError::Network(e.to_string()))?;

        Self::parse_publication_response(content_type.as_deref(), &body)
    }

    fn parse_feed_response(content_type: Option<&str>, body: &str) -> Result<Feed, OpdsError> {
        match detect_format(content_type, body) {
            BodyFormat::Json => serde_json::from_str(body).map_err(OpdsError::from),
            #[cfg(feature = "atom")]
            BodyFormat::Atom => livtet_opds_atom::parse_feed(body),
            #[cfg(not(feature = "atom"))]
            BodyFormat::Atom => Err(OpdsError::UnsupportedFormat(
                "Atom feeds require the `atom` feature".to_string(),
            )),
        }
    }

    fn parse_publication_response(
        content_type: Option<&str>,
        body: &str,
    ) -> Result<Publication, OpdsError> {
        match detect_format(content_type, body) {
            BodyFormat::Json => serde_json::from_str(body).map_err(OpdsError::from),
            #[cfg(feature = "atom")]
            BodyFormat::Atom => {
                let feed = livtet_opds_atom::parse_feed(body)?;
                feed.publications
                    .into_iter()
                    .next()
                    .ok_or_else(|| OpdsError::UnsupportedFormat("empty Atom feed".to_string()))
            }
            #[cfg(not(feature = "atom"))]
            BodyFormat::Atom => Err(OpdsError::UnsupportedFormat(
                "Atom feeds require the `atom` feature".to_string(),
            )),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BodyFormat {
    Json,
    Atom,
}

fn detect_format(content_type: Option<&str>, body: &str) -> BodyFormat {
    if let Some(ct) = content_type {
        let ct = ct.split(';').next().unwrap_or(ct).trim().to_lowercase();
        if ct == "application/atom+xml" || ct == "application/xml" || ct == "text/xml" {
            return BodyFormat::Atom;
        }
        if ct == "application/opds+json" || ct == "application/json" {
            return BodyFormat::Json;
        }
    }

    let trimmed = body.trim_start();
    if trimmed.starts_with("<?xml")
        || trimmed.starts_with("<feed")
        || trimmed.starts_with("<atom:feed")
    {
        BodyFormat::Atom
    } else {
        BodyFormat::Json
    }
}
