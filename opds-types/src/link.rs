use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

pub mod rel {
    pub const SELF: &str = "self";
    pub const START: &str = "start";
    pub const UP: &str = "up";
    pub const FIRST: &str = "first";
    pub const PREVIOUS: &str = "previous";
    pub const NEXT: &str = "next";
    pub const LAST: &str = "last";
    pub const SEARCH: &str = "search";
    pub const SUBSCRIBE: &str = "subscribe";
    pub const NAVIGATION: &str = "navigation";
    pub const PUBLICATIONS: &str = "publications";
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Properties {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_items: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub indirect_acquisition: Option<Vec<Acquisition>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub availability: Option<Availability>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Price {
    pub currency: String,
    pub value: f64,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Acquisition {
    #[serde(rename = "type")]
    pub type_: String,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub child: Vec<Acquisition>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Availability {
    pub state: String,
    #[serde(
        default,
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub since: Option<OffsetDateTime>,
    #[serde(
        default,
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "specta", specta(type = Option<String>))]
    pub until: Option<OffsetDateTime>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    pub href: String,

    #[serde(rename = "type")]
    pub type_: String,

    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub templated: bool,

    #[serde(deserialize_with = "deserialize_rel")]
    #[cfg_attr(feature = "specta", specta(type = Vec<String>))]
    pub rel: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hreflang: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<Properties>,
}

impl Link {
    pub fn new(href: impl Into<String>, type_: impl Into<String>, rel: Vec<String>) -> Self {
        Self {
            href: href.into(),
            type_: type_.into(),
            templated: false,
            rel,
            title: None,
            hreflang: None,
            length: None,
            properties: None,
        }
    }

    pub fn is_search(&self) -> bool {
        self.rel.iter().any(|r| r == rel::SEARCH)
    }

    pub fn is_next(&self) -> bool {
        self.rel.iter().any(|r| r == rel::NEXT)
    }

    pub fn is_self(&self) -> bool {
        self.rel.iter().any(|r| r == rel::SELF)
    }
}

/// Deserialize `rel` as either a single string or an array of strings.
///
/// OPDS 2 / Readium manifests often emit `"rel": "search"`, while the
/// canonical OPDS spec allows an array. We normalize to `Vec<String>`
/// at the boundary so the rest of the codebase can always work with a
/// list.
fn deserialize_rel<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct RelVisitor;

    impl<'de> serde::de::Visitor<'de> for RelVisitor {
        type Value = Vec<String>;

        #[cfg_attr(test, mutants::skip)]
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or an array of strings")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![value.to_string()])
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(vec![value])
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            serde::Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(RelVisitor)
}

#[cfg(test)]
mod tests {
    use crate::link::Link;

    #[test]
    fn is_search_matches_search_rel() {
        let link = Link::new(
            "/search",
            "application/opds+json",
            vec![crate::link::rel::SEARCH.to_string()],
        );
        assert!(link.is_search());
        assert!(!link.is_next());
        assert!(!link.is_self());
    }

    #[test]
    fn is_next_matches_next_rel() {
        let link = Link::new(
            "/next",
            "application/opds+json",
            vec![crate::link::rel::NEXT.to_string()],
        );
        assert!(link.is_next());
        assert!(!link.is_search());
        assert!(!link.is_self());
    }

    #[test]
    fn is_self_matches_self_rel() {
        let link = Link::new(
            "/self",
            "application/opds+json",
            vec![crate::link::rel::SELF.to_string()],
        );
        assert!(link.is_self());
        assert!(!link.is_search());
        assert!(!link.is_next());
    }

    #[test]
    fn rel_checkers_return_false_for_unrelated_rel() {
        let link = Link::new(
            "/up",
            "application/opds+json",
            vec![crate::link::rel::UP.to_string()],
        );
        assert!(!link.is_search());
        assert!(!link.is_next());
        assert!(!link.is_self());
    }

    #[test]
    fn rel_checkers_match_among_multiple_rels() {
        let link = Link {
            href: "/multi".to_string(),
            type_: "application/opds+json".to_string(),
            templated: false,
            rel: vec!["up".to_string(), "next".to_string(), "search".to_string()],
            title: None,
            hreflang: None,
            length: None,
            properties: None,
        };
        assert!(link.is_search());
        assert!(link.is_next());
        assert!(!link.is_self());
    }

    #[test]
    fn deserialize_rel_single_string() {
        let json = serde_json::json!({"href": "/test", "type": "a/a", "rel": "search"});
        let link: Link = serde_json::from_value(json).unwrap();
        assert_eq!(link.rel, vec!["search"]);
    }

    #[test]
    fn deserialize_rel_array_of_strings() {
        let json = serde_json::json!({"href": "/test", "type": "a/a", "rel": ["search", "self"]});
        let link: Link = serde_json::from_value(json).unwrap();
        assert_eq!(link.rel, vec!["search", "self"]);
    }
}
