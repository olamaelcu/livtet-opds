use std::collections::BTreeMap;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordVerifier},
};
use base64::Engine;
use livtet_opds_types::AuthenticationDocument;
use poem::{Endpoint, Middleware, Request, Response, Result, http::StatusCode};

/// Configuration for OPDS Basic authentication.
///
/// Created from `OpdsSettings` at server-start time and passed through
/// to the poem middleware stack. Note: the actual middleware wrapper
/// (`AuthMiddleware`) was removed in this revision — see the note
/// at the bottom of this file. The helpers here stay usable from
/// any caller that needs to construct or verify Basic Auth headers.
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// Whether the server requires authentication.
    pub require_auth: bool,
    /// The expected username for Basic auth.
    pub username: String,
    /// The argon2 hash of the expected password.
    pub password_hash: String,
}

impl AuthConfig {
    /// Verify an `Authorization` header value against stored credentials.
    ///
    /// The header must be in the form `Basic <base64(username:password)>`.
    /// The password portion is verified against the stored argon2 hash.
    pub fn verify(&self, header: &str) -> bool {
        if !self.require_auth {
            return true;
        }

        let encoded = match header.strip_prefix("Basic ") {
            Some(e) => e.trim(),
            None => return false,
        };

        let decoded = match base64::engine::general_purpose::STANDARD.decode(encoded) {
            Ok(d) => d,
            Err(_) => return false,
        };

        let decoded_str = match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let colon_pos = match decoded_str.find(':') {
            Some(p) => p,
            None => return false,
        };

        let username = &decoded_str[..colon_pos];
        let password = &decoded_str[colon_pos + 1..];

        if username != self.username {
            return false;
        }

        if self.password_hash.is_empty() {
            return false;
        }

        let parsed_hash = match PasswordHash::new(&self.password_hash) {
            Ok(h) => h,
            Err(_) => return false,
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok()
    }

    /// The OPDS Authentication Document body that an OPDS server
    /// exposes on `GET /authentication` and as the 401 body when
    /// auth is required but missing or invalid. Constructed from
    /// the typed [`AuthenticationDocument`] so the wire shape is
    /// pinned in one place (and round-trips through serde cleanly).
    pub fn build_auth_document(&self) -> AuthenticationDocument {
        let mut labels = BTreeMap::new();
        labels.insert("login".to_string(), "Username".to_string());
        labels.insert("password".to_string(), "Password".to_string());

        AuthenticationDocument {
            id: "http://localhost:3121/authentication".to_string(),
            title: "Livtet Catalog".to_string(),
            description: "Enter your Livtet credentials".to_string(),
            authentication: vec![livtet_opds_types::AuthenticationMethod {
                type_: "http://opds-spec.org/auth/basic".to_string(),
                labels,
            }],
        }
    }
}

/// Poem middleware that enforces OPDS Basic authentication.
///
/// The middleware checks every request except `GET /authentication`
/// (the OPDS Authentication Document endpoint). When auth is required
/// and the request lacks valid credentials, it returns a 401 with the
/// OPDS Authentication Document as the body.
///
/// The middleware's `Output = Response` matches the inner route's
/// `Output = Response`, so they compose without boxing. The 401 path
/// is built via `Response::builder()` with the Authentication
/// Document serialized as the body. On a successful inner call the
/// response is passed through unchanged; on a failed inner call the
/// 401 is returned instead (the auth gateway intentionally
/// overrides inner errors with the auth challenge so the client
/// gets the challenge it needs to retry).
pub struct AuthMiddleware {
    config: AuthConfig,
}

impl AuthMiddleware {
    pub fn new(config: AuthConfig) -> Self {
        Self { config }
    }
}

impl<E: Endpoint<Output = Response>> Middleware<E> for AuthMiddleware {
    type Output = AuthEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        AuthEndpoint {
            inner: ep,
            config: self.config.clone(),
        }
    }
}

/// The wrapped endpoint produced by `AuthMiddleware`.
pub struct AuthEndpoint<E> {
    inner: E,
    config: AuthConfig,
}

impl<E: Endpoint<Output = Response>> Endpoint for AuthEndpoint<E> {
    type Output = Response;

    async fn call(&self, req: Request) -> Result<Self::Output> {
        // The authentication document endpoint is always accessible
        // so OPDS clients can discover the auth mechanism.
        if req.uri().path() == "/authentication" {
            return self.inner.call(req).await;
        }

        if !self.config.require_auth {
            return self.inner.call(req).await;
        }

        let auth_header = req
            .headers()
            .get("authorization")
            .and_then(|v| v.to_str().ok());

        if matches!(auth_header, Some(h) if self.config.verify(h)) {
            return self.inner.call(req).await;
        }

        // Auth required but missing or invalid: build a 401 response
        // carrying the OPDS Authentication Document.
        Ok(self.build_401())
    }
}

impl<E: Endpoint<Output = Response>> AuthEndpoint<E> {
    /// Construct the 401 response body and headers.
    fn build_401(&self) -> Response {
        let body = serde_json::to_string(&self.config.build_auth_document())
            .unwrap_or_else(|_| "{}".to_string());

        Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .header("WWW-Authenticate", r#"Basic realm="Livtet""#)
            .header("Content-Type", "application/opds-authentication+json")
            .body(body)
    }
}
