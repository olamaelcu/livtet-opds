use serde::{Deserialize, Deserializer, Serialize};
use time::OffsetDateTime;

use crate::{
    feed::Collection,
    link::Link,
    serde_util::{OneOrMany, deserialize_language},
};

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

    /// OPDS 2.0 emits `language` as a string or an array of BCP47 tags.
    #[serde(
        default,
        deserialize_with = "deserialize_language",
        skip_serializing_if = "Option::is_none"
    )]
    pub language: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub publisher: Option<String>,

    /// OPDS 2.0 §2.1 marks `modified` as required, but real catalogues
    /// (notably the Internet Archive) omit it. Absent means unknown.
    #[serde(
        default,
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub modified: Option<OffsetDateTime>,

    #[serde(
        default,
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub published: Option<OffsetDateTime>,

    /// OPDS 2.0 spells this `author` and allows a bare string, a single
    /// Contributor object, or an array of either. It is normalized to a list.
    #[serde(
        default,
        alias = "author",
        deserialize_with = "deserialize_authors",
        skip_serializing_if = "Vec::is_empty"
    )]
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

/// A contributor as OPDS 2.0 emits it: either a full Contributor object or a
/// bare display name.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum Contributor {
    Named(Author),
    Plain(String),
}

/// Normalize an OPDS 2.0 `author` into the internal `Vec<Author>`, accepting
/// a string, a Contributor object, or an array of either.
fn deserialize_authors<'de, D>(deserializer: D) -> Result<Vec<Author>, D::Error>
where
    D: Deserializer<'de>,
{
    let parsed = Option::<OneOrMany<Contributor>>::deserialize(deserializer)?;
    Ok(parsed
        .map(|value| {
            value
                .into_vec()
                .into_iter()
                .map(|contributor| match contributor {
                    Contributor::Named(author) => author,
                    Contributor::Plain(name) => Author {
                        name,
                        sort_as: None,
                        role: None,
                        links: Vec::new(),
                    },
                })
                .collect()
        })
        .unwrap_or_default())
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
