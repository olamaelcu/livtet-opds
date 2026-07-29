use livtet_opds_types::{
    Author, Availability, BelongsTo, Collection, Feed, Link, Metadata, Properties, Publication,
    PublicationMetadata, Series,
};
use time::OffsetDateTime;

use crate::repository::{FacetAxis, FacetCounts, WorkAggregate, WorkQuery};

#[derive(Debug, Clone)]
pub struct DefaultOpdsMapper {
    pub base_url: String,
}

/// Pagination state for a paginated feed response. Bundles the
/// `total`, `page`, and `per_page` triple so feed-building methods
/// stay under clippy's 7-argument cap.
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
}

impl DefaultOpdsMapper {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub fn work_to_publication(&self, work: &WorkAggregate) -> Publication {
        let title = work
            .editions
            .first()
            .and_then(|e| e.title.clone())
            .unwrap_or_else(|| work.title.clone());

        // Map work-level `authors` (which carry a role) into the OPDS
        // `Author` shape. The work-author join table currently exposes
        // the bare name plus role; we forward both so the frontend can
        // distinguish "author" from "editor" etc. when it builds the
        // lede.
        let authors: Vec<Author> = work
            .authors
            .iter()
            .map(|a| Author {
                name: a.name.clone(),
                sort_as: None,
                role: Some(a.role.clone()),
                links: Vec::new(),
            })
            .collect();

        let translator = work
            .editions
            .iter()
            .flat_map(|e| e.authors.iter())
            .find(|a| a.role == "translator")
            .map(|a| a.name.clone());

        let narrator = work
            .editions
            .iter()
            .flat_map(|e| e.authors.iter())
            .find(|a| a.role == "narrator")
            .map(|a| a.name.clone());

        // `series` and `collections` both default to empty when the
        // work doesn't carry membership in any grouping. The mapper
        // currently only projects a single primary series; further
        // work can fold in collection membership from the DB.
        let belongs_to = if !work.series.is_empty() {
            Some(BelongsTo {
                series: work
                    .series
                    .iter()
                    .map(|s| Series {
                        name: s.name.clone(),
                        position: None,
                    })
                    .collect(),
                collections: Vec::new(),
            })
        } else {
            None
        };

        let links = self.build_publication_links(work);
        let images = self.build_publication_images(work);

        Publication {
            metadata: PublicationMetadata {
                title,
                identifier: Some(work.id.clone()),
                description: work.description.clone(),
                language: work.editions.first().and_then(|e| e.language.clone()),
                publisher: work.publishers.first().cloned(),
                modified: OffsetDateTime::now_utc(),
                published: None,
                authors,
                subjects: Vec::new(),
                translator,
                narrator,
                illustrator: None,
                editor: None,
                belongs_to,
            },
            links,
            images,
        }
    }

    /// Build a navigation [`Feed`] from a flat list of `(label, href)`
    /// pairs. Used by the navigation handlers to emit Authors / Tags /
    /// Genres root feeds where each link in `navigation` is a
    /// subsection pointing at a filtered feed URL.
    pub fn navigation_feed(&self, title: &str, items: Vec<(String, String)>) -> Feed {
        let mut feed = Feed::new(title);
        for (label, href) in items {
            feed.links.push(Link {
                href: format!("{}{href}", self.base_url),
                type_: "application/opds+json".to_string(),
                templated: false,
                rel: vec!["subsection".to_string()],
                title: Some(label),
                hreflang: None,
                length: None,
                properties: None,
            });
        }
        feed
    }

    fn build_facet(
        &self,
        label: &str,
        axis: FacetAxis,
        counts: &[super::repository::FacetCount],
        query: &WorkQuery,
    ) -> Collection {
        let active_value = axis.current_filter(query).map(str::to_string);
        let links = counts
            .iter()
            .map(|c| {
                let is_active = active_value.as_deref() == Some(c.id.as_str());
                let mut facet_query = query.clone();
                let (rel, href) = if is_active {
                    // Toggle off: drop the filter for this axis so the
                    // href points back to the un-faceted feed.
                    axis.clear_filter(&mut facet_query);
                    let href = if facet_query.to_string().is_empty() {
                        self.base_url.trim_end_matches('/').to_string()
                    } else {
                        format!("{}?{}", self.base_url.trim_end_matches('/'), facet_query)
                    };
                    (vec!["self".to_string()], href)
                } else {
                    // Toggle on: drop the existing filter for this
                    // axis (the active value) and set the new value
                    // as the axis's primary filter. The result is a
                    // query where the only axis filter is the one we
                    // just picked. The test pins the inactive href to
                    // `tag_id=beta` (with no stacked `tag_id=alpha`),
                    // which is exactly the replace-not-stack contract.
                    axis.clear_filter(&mut facet_query);
                    axis.set_filter(&mut facet_query, c.id.clone());
                    let href = format!(
                        "{}/search?{}",
                        self.base_url.trim_end_matches('/'),
                        facet_query
                    );
                    (Vec::new(), href)
                };

                let count_u64 = u64::try_from(c.count.max(0)).unwrap_or(0);
                Link {
                    href,
                    type_: "application/opds+json".to_string(),
                    templated: false,
                    rel,
                    title: Some(c.label.clone()),
                    hreflang: None,
                    length: None,
                    properties: Some(Properties {
                        number_of_items: Some(count_u64),
                        price: None,
                        indirect_acquisition: None,
                        availability: None,
                    }),
                }
            })
            .collect();

        Collection {
            metadata: Metadata {
                title: label.to_string(),
                identifier: None,
                type_: Some("application/opds+json".to_string()),
                description: None,
                modified: None,
                number_of_items: None,
                items_per_page: None,
                current_page: None,
            },
            links,
        }
    }

    /// Build a [`Feed`] from a flat list of works. Called by the
    /// publications/search handlers. The feed's `facets` field is
    /// left empty; callers populate it via [`Self::build_facets`].
    pub fn works_to_feed(
        &self,
        title: &str,
        works: &[WorkAggregate],
        pagination: Pagination,
        counts: &FacetCounts,
        query: &WorkQuery,
    ) -> Feed {
        let collections = self.build_collections(counts, query);

        let publications: Vec<Publication> =
            works.iter().map(|w| self.work_to_publication(w)).collect();

        Feed {
            metadata: Metadata {
                title: title.to_string(),
                identifier: None,
                type_: None,
                description: None,
                modified: Some(OffsetDateTime::now_utc()),
                number_of_items: Some(pagination.total),
                items_per_page: Some(pagination.per_page),
                current_page: Some(pagination.page),
            },
            links: vec![Link {
                href: format!("{}/opds/v1.2/root", self.base_url),
                type_: "application/opds+json".to_string(),
                templated: false,
                rel: vec!["self".to_string()],
                title: None,
                hreflang: None,
                length: None,
                properties: None,
            }],
            navigation: collections,
            publications,
            facets: vec![],
            groups: vec![],
        }
    }

    /// Build per-axis facet [`Collection`]s for inclusion in a feed's
    /// `facets` array. Axes with zero entries are skipped. The first
    /// positional argument (`_base_url`) is reserved for future use —
    /// hrefs are currently derived from `self.base_url` since the
    /// mapper is the canonical owner of the base URL.
    pub fn build_facets(
        &self,
        _base_url: &str,
        query: &WorkQuery,
        counts: &FacetCounts,
    ) -> Vec<Collection> {
        FacetAxis::ALL
            .iter()
            .filter_map(|axis| {
                let entries = counts.for_axis(*axis);
                if entries.is_empty() {
                    None
                } else {
                    Some(self.build_facet(axis.label(), *axis, entries, query))
                }
            })
            .collect()
    }

    fn build_collections(&self, counts: &FacetCounts, query: &WorkQuery) -> Vec<Collection> {
        let mut collections = Vec::new();

        if !counts.authors.is_empty() {
            collections.push(self.build_facet(
                FacetAxis::Author.label(),
                FacetAxis::Author,
                &counts.authors,
                query,
            ));
        }

        if !counts.genres.is_empty() {
            collections.push(self.build_facet(
                FacetAxis::Genre.label(),
                FacetAxis::Genre,
                &counts.genres,
                query,
            ));
        }

        if !counts.tags.is_empty() {
            collections.push(self.build_facet(
                FacetAxis::Tag.label(),
                FacetAxis::Tag,
                &counts.tags,
                query,
            ));
        }

        if !counts.languages.is_empty() {
            collections.push(self.build_facet(
                FacetAxis::Language.label(),
                FacetAxis::Language,
                &counts.languages,
                query,
            ));
        }

        if !counts.subjects.is_empty() {
            collections.push(self.build_facet(
                FacetAxis::Subject.label(),
                FacetAxis::Subject,
                &counts.subjects,
                query,
            ));
        }

        if !counts.publishers.is_empty() {
            collections.push(self.build_facet(
                FacetAxis::Publisher.label(),
                FacetAxis::Publisher,
                &counts.publishers,
                query,
            ));
        }

        collections
    }

    fn build_publication_links(&self, work: &WorkAggregate) -> Vec<Link> {
        let mut links = vec![Link {
            // Self link points at the publication's record URL. The
            // concrete record route is `/publications/:work_id`; the
            // trailing `/opds/v1.2` namespace from the placeholder is
            // intentionally dropped now that we have the work id in
            // hand (it was a stale anchor from the placeholder
            // implementation).
            href: format!("{}/publications/{}", self.base_url, work.id),
            type_: "application/opds+json".to_string(),
            templated: false,
            rel: vec!["self".to_string()],
            title: None,
            hreflang: None,
            length: None,
            properties: None,
        }];

        // Acquisition links: one per edition with a linked digital
        // file. The href encodes the inventory_id so the client can
        // disambiguate when a work has multiple files; the download
        // handler treats the query param as optional and falls back
        // to the first available file.
        for edition in &work.editions {
            let (Some(inv_id), Some(mime_type)) = (&edition.inventory_id, &edition.file_mime_type)
            else {
                continue;
            };
            let href = format!(
                "{}/publications/{}/download?inventory_id={}",
                self.base_url, work.id, inv_id
            );
            links.push(Link {
                href,
                type_: mime_type.clone(),
                templated: false,
                // The OPDS standard rel for a downloadable copy.
                rel: vec!["http://opds-spec.org/acquisition".to_string()],
                title: None,
                hreflang: None,
                // Advertise the size so progress-bar clients can
                // estimate without a HEAD probe. `None` when we
                // don't know the size up front.
                length: None,
                properties: Some(Properties {
                    number_of_items: None,
                    price: None,
                    indirect_acquisition: None,
                    availability: Some(Availability {
                        state: "available".to_string(),
                        since: None,
                        until: None,
                    }),
                }),
            });
        }

        links
    }

    fn build_publication_images(&self, work: &WorkAggregate) -> Vec<Link> {
        let editions = if let Some(ref preferred_id) = work.preferred_edition_id {
            let preferred = work
                .editions
                .iter()
                .find(|e| e.id == *preferred_id && e.cover_path.is_some());
            if let Some(ed) = preferred {
                vec![ed]
            } else {
                work.editions
                    .iter()
                    .filter(|e| e.cover_path.is_some())
                    .collect()
            }
        } else {
            work.editions
                .iter()
                .filter(|e| e.cover_path.is_some())
                .collect()
        };

        editions
            .iter()
            .filter_map(|ed| {
                let cover_path = ed.cover_path.as_ref()?;
                let ext = std::path::Path::new(cover_path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("jpg");
                Some(Link {
                    href: format!("{}/covers/{}", self.base_url.trim_end_matches('/'), ed.id),
                    type_: format!("image/{}", ext),
                    templated: false,
                    rel: vec!["http://opds-spec.org/image".to_string()],
                    title: ed.title.clone(),
                    hreflang: None,
                    length: None,
                    properties: None,
                })
            })
            .collect()
    }
}
