//! Serde helpers for shapes that real OPDS servers emit but the strict model
//! spells differently.
//!
//! OPDS 2.0 is a living standard and its adopters (notably the Internet
//! Archive) deviate from the canonical Readium shapes in a few predictable
//! ways: `navigation` is a compact link array, contributor `author` is a
//! string-or-object-or-array, and `language` is a string-or-array. These
//! helpers normalize those forms at the boundary.

use serde::{Deserialize, Deserializer};

/// A value that may appear either as a single `T` or as an array of `T`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T> OneOrMany<T> {
    /// Flatten into a list, wrapping a lone value in a one-element vector.
    pub fn into_vec(self) -> Vec<T> {
        match self {
            OneOrMany::One(value) => vec![value],
            OneOrMany::Many(values) => values,
        }
    }
}

/// Deserialize an optional `language` that may be a string or an array of
/// strings, normalizing to the first non-empty value.
pub fn deserialize_language<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let parsed = Option::<OneOrMany<String>>::deserialize(deserializer)?;
    Ok(parsed.and_then(|value| value.into_vec().into_iter().find(|s| !s.is_empty())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct Holder {
        #[serde(default, deserialize_with = "deserialize_language")]
        language: Option<String>,
    }

    #[test]
    fn language_accepts_a_string() {
        let holder: Holder = serde_json::from_str(r#"{"language":"en"}"#).unwrap();
        assert_eq!(holder.language.as_deref(), Some("en"));
    }

    #[test]
    fn language_accepts_an_array() {
        let holder: Holder = serde_json::from_str(r#"{"language":["la","en"]}"#).unwrap();
        assert_eq!(holder.language.as_deref(), Some("la"));
    }

    #[test]
    fn language_is_none_when_absent_or_null() {
        let absent: Holder = serde_json::from_str("{}").unwrap();
        assert_eq!(absent.language, None);
        let null: Holder = serde_json::from_str(r#"{"language":null}"#).unwrap();
        assert_eq!(null.language, None);
    }

    #[test]
    fn one_or_many_flattens_both_shapes() {
        let one: OneOrMany<u8> = serde_json::from_str("7").unwrap();
        assert_eq!(one.into_vec(), vec![7]);
        let many: OneOrMany<u8> = serde_json::from_str("[1,2]").unwrap();
        assert_eq!(many.into_vec(), vec![1, 2]);
    }
}
