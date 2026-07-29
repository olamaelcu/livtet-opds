use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::{link::Link, publication::Publication};

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
    pub title: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub identifier: Option<String>,

    #[serde(
        rename = "type",
        default = "default_type",
        skip_serializing_if = "Option::is_none"
    )]
    pub type_: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    #[serde(
        default,
        with = "time::serde::rfc3339::option",
        skip_serializing_if = "Option::is_none"
    )]
    #[cfg_attr(feature = "specta", specta(type = specta_serde::Phased<Option<String>, Option<String>>))]
    pub modified: Option<OffsetDateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_items: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_per_page: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_page: Option<u64>,
}

fn default_type() -> Option<String> {
    Some("application/opds+json".to_string())
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Collection {
    pub metadata: Metadata,
    pub links: Vec<Link>,
}

#[cfg_attr(feature = "specta", derive(specta::Type))]
#[cfg_attr(feature = "fake", derive(fake::Dummy))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Feed {
    pub metadata: Metadata,
    pub links: Vec<Link>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub navigation: Vec<Collection>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub publications: Vec<Publication>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub facets: Vec<Collection>,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Collection>,
}

impl Feed {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            metadata: Metadata {
                title: title.into(),
                identifier: None,
                type_: None,
                description: None,
                modified: Some(OffsetDateTime::now_utc()),
                number_of_items: None,
                items_per_page: None,
                current_page: None,
            },
            links: Vec::new(),
            navigation: Vec::new(),
            publications: Vec::new(),
            facets: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn next_page(&self) -> Option<&Link> {
        self.links.iter().find(|l| l.is_next())
    }

    pub fn search_template(&self) -> Option<&Link> {
        self.links.iter().find(|l| l.is_search() && l.templated)
    }
}

#[cfg(test)]
mod tests {
    use crate::{feed::Feed, link::Link};

    #[test]
    fn default_type_is_opds_json() {
        assert_eq!(
            super::default_type(),
            Some("application/opds+json".to_string())
        );
    }

    #[test]
    fn next_page_finds_link_with_next_rel() {
        let mut feed = Feed::new("test");
        let link = Link::new(
            "/next",
            "application/opds+json",
            vec![crate::link::rel::NEXT.to_string()],
        );
        feed.links.push(link.clone());
        assert_eq!(feed.next_page(), Some(&link));
    }

    #[test]
    fn next_page_returns_none_when_no_next_link() {
        let feed = Feed::new("test");
        assert_eq!(feed.next_page(), None);
    }

    #[test]
    fn search_template_finds_templated_search_link() {
        let mut feed = Feed::new("test");
        let link = Link {
            href: "/search{?query}".to_string(),
            type_: "application/opds+json".to_string(),
            templated: true,
            rel: vec![crate::link::rel::SEARCH.to_string()],
            title: None,
            hreflang: None,
            length: None,
            properties: None,
        };
        feed.links.push(link.clone());
        assert_eq!(feed.search_template(), Some(&link));
    }

    #[test]
    fn search_template_skips_non_templated_search_link() {
        let mut feed = Feed::new("test");
        feed.links.push(Link::new(
            "/search",
            "application/opds+json",
            vec![crate::link::rel::SEARCH.to_string()],
        ));
        assert_eq!(feed.search_template(), None);
    }

    #[test]
    fn search_template_skips_templated_non_search_link() {
        let mut feed = Feed::new("test");
        feed.links.push(Link {
            href: "/next".to_string(),
            type_: "application/opds+json".to_string(),
            templated: true,
            rel: vec![crate::link::rel::NEXT.to_string()],
            title: None,
            hreflang: None,
            length: None,
            properties: None,
        });
        assert_eq!(feed.search_template(), None);
    }
}
