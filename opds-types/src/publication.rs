use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{feed::Collection, link::Link};

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Author {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_as: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<Link>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PublicationMetadata {
    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// Required by OPDS 2.0 §2.1 — every publication entry MUST carry a
    /// `modified` timestamp so clients can detect catalogue changes.
    #[serde(with = "time::serde::rfc3339")]
    #[cfg_attr(feature = "specta", specta(type = String))]
    pub modified: OffsetDateTime,

    #[serde(
        default,
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub published: Option<OffsetDateTime>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<Author>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subjects: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub translator: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrator: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub illustrator: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub belongs_to: Option<BelongsTo>,
}

impl PublicationMetadata {
    /// Render the `authors` list as a single human-readable string
    /// (e.g. for a card lede). Returns an empty string when there
    /// are no authors; callers decide whether to substitute
    /// `"Unknown author"`.
    pub fn flat_authors(&self) -> String {
        self.authors
            .iter()
            .map(|a| a.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BelongsTo {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub series: Vec<Series>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<Collection>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Series {
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<f32>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Publication {
    pub metadata: PublicationMetadata,
    pub links: Vec<Link>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub images: Vec<Link>,
}
