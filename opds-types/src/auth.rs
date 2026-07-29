use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The OPDS Authentication Document served on `GET /authentication`
/// and returned as the body of a 401 when auth is required but
/// missing or invalid.
///
/// Wire shape (snake_case-free — this is the OPDS Authentication
/// Document spec, not the OPDS 2 Feed shape):
/// ```json
/// {
///   "id": "http://localhost:3121/authentication",
///   "title": "Livtet Catalog",
///   "description": "Enter your Livtet credentials",
///   "authentication": [{
///     "type": "http://opds-spec.org/auth/basic",
///     "labels": { "login": "Username", "password": "Password" }
///   }]
/// }
/// ```
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthenticationDocument {
    pub id: String,
    pub title: String,
    pub description: String,
    pub authentication: Vec<AuthenticationMethod>,
}

/// A single authentication option in the OPDS Authentication
/// Document. `labels` are localized form-field names keyed by
/// their purpose (e.g. `"login"`, `"password"`). `BTreeMap` is
/// used over `HashMap` so the serialized form has deterministic
/// key order — easier to diff in tests and prettier on the wire.
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthenticationMethod {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub labels: BTreeMap<String, String>,
}
