use async_trait::async_trait;
use livtet_opds_types::OpdsError;

#[derive(Debug, Clone)]
pub struct AuthorInfo {
    pub id: String,
    pub name: String,
    pub role: String,
}

#[derive(Debug, Clone)]
pub struct EditionAggregate {
    pub id: String,
    pub title: Option<String>,
    pub isbn: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub published_date: Option<String>,
    pub page_count: Option<i32>,
    pub format: Option<String>,
    pub authors: Vec<AuthorInfo>,
    pub cover_path: Option<String>,
    pub file_path: Option<String>,
    pub identifiers: Vec<String>,
    /// The `digital_inventory` row id for this edition's linked file, if
    /// any. `None` when the edition has no `digital_inventory` row, or when
    /// the row exists but `file_path` was nulled out by the pruning helper.
    /// Used by the OPDS acquisition link and download endpoint to address
    /// a specific file when a work has multiple editions/files.
    pub inventory_id: Option<String>,
    /// MIME type detected from `file_path`'s extension at query time.
    /// `None` when there's no linked file or when the extension is
    /// unrecognized (downstream code falls back to
    /// `application/octet-stream`). Drives the OPDS acquisition link's
    /// `type` field and the download endpoint's `Content-Type`.
    pub file_mime_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SeriesInfo {
    pub id: String,
    pub name: String,
    pub position: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct WorkAggregate {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub sort_title: Option<String>,
    pub preferred_edition_id: Option<String>,
    pub authors: Vec<AuthorInfo>,
    pub editions: Vec<EditionAggregate>,
    pub tags: Vec<String>,
    pub genres: Vec<String>,
    pub subjects: Vec<String>,
    pub publishers: Vec<String>,
    pub series: Vec<SeriesInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct WorkQuery {
    pub author_id: Option<String>,
    pub genre_id: Option<String>,
    pub tag_id: Option<String>,
    pub format_id: Option<String>,
    pub language_id: Option<String>,
    pub publisher_id: Option<String>,
    pub subject_id: Option<String>,
    pub series_id: Option<String>,
    pub sort_by: Option<String>,
    pub sort_direction: Option<String>,
    pub page: Option<u64>,
    pub limit: Option<u64>,
    pub search_query: Option<String>,
    /// Facet axis being toggled in the active request. Set by the mapper
    /// when building per-facet hrefs so the toggle links only need to
    /// mutate the active axis. `None` for the un-faceted feed.
    pub facet: Option<FacetAxis>,
    /// Facet value (typically the axis row's `id`) being toggled. Set
    /// alongside [`WorkQuery::facet`] by the mapper when emitting
    /// hrefs for a specific (axis, value) pair.
    pub value: Option<String>,
}

impl std::fmt::Display for WorkQuery {
    /// Render the query as a URL-encoded query string. `None` fields
    /// are omitted. Used by the mapper to build per-facet hrefs.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts: Vec<String> = Vec::new();
        if let Some(v) = &self.author_id {
            parts.push(format!("author_id={v}"));
        }
        if let Some(v) = &self.genre_id {
            parts.push(format!("genre_id={v}"));
        }
        if let Some(v) = &self.tag_id {
            parts.push(format!("tag_id={v}"));
        }
        if let Some(v) = &self.format_id {
            parts.push(format!("format_id={v}"));
        }
        if let Some(v) = &self.language_id {
            parts.push(format!("language_id={v}"));
        }
        if let Some(v) = &self.publisher_id {
            parts.push(format!("publisher_id={v}"));
        }
        if let Some(v) = &self.subject_id {
            parts.push(format!("subject_id={v}"));
        }
        if let Some(v) = &self.series_id {
            parts.push(format!("series_id={v}"));
        }
        if let Some(v) = &self.sort_by {
            parts.push(format!("sort_by={v}"));
        }
        if let Some(v) = &self.sort_direction {
            parts.push(format!("sort_direction={v}"));
        }
        if let Some(v) = self.page {
            parts.push(format!("page={v}"));
        }
        if let Some(v) = self.limit {
            parts.push(format!("limit={v}"));
        }
        if let Some(v) = &self.search_query {
            parts.push(format!("q={v}"));
        }
        if let Some(axis) = self.facet {
            parts.push(format!("facet={}", axis.query_key()));
        }
        if let Some(v) = &self.value {
            parts.push(format!("value={v}"));
        }
        write!(f, "{}", parts.join("&"))
    }
}

/// Catalog axes that the OPDS feed can expose as facets. Each axis maps to a
/// single `Collection` in the feed's `facets` array, and its values are
/// computed by a `GROUP BY` query against the junction (or per-edition)
/// table. The set of axes is intentionally small and stable: bumping it
/// requires a corresponding change in the mapper so the per-axis title
/// (`"Authors"`, `"Tags"`, ...) stays in sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FacetAxis {
    Author,
    Tag,
    Genre,
    Subject,
    Publisher,
    Format,
    Language,
}

impl FacetAxis {
    /// All axes, in the order they're emitted by [`OpdsRepository::compute_all_facets`].
    pub const ALL: &'static [FacetAxis] = &[
        FacetAxis::Author,
        FacetAxis::Tag,
        FacetAxis::Genre,
        FacetAxis::Subject,
        FacetAxis::Publisher,
        FacetAxis::Format,
        FacetAxis::Language,
    ];

    /// Human-readable label used as the `Collection.metadata.title` for
    /// a facet axis. Kept in sync with the `for_axis` slice on
    /// [`FacetCounts`] so the mapper can render every axis title
    /// without a hand-rolled dispatch.
    pub fn label(&self) -> &'static str {
        match self {
            FacetAxis::Author => "Authors",
            FacetAxis::Tag => "Tags",
            FacetAxis::Genre => "Genres",
            FacetAxis::Subject => "Subjects",
            FacetAxis::Publisher => "Publishers",
            FacetAxis::Format => "Formats",
            FacetAxis::Language => "Languages",
        }
    }

    /// Query-string key used to filter the feed on this axis. The mapper
    /// uses this to build the toggle-on / toggle-off href for each facet
    /// value.
    pub fn query_key(&self) -> &'static str {
        match self {
            FacetAxis::Author => "author_id",
            FacetAxis::Tag => "tag_id",
            FacetAxis::Genre => "genre_id",
            FacetAxis::Subject => "subject_id",
            FacetAxis::Publisher => "publisher_id",
            FacetAxis::Format => "format_id",
            FacetAxis::Language => "language_id",
        }
    }

    /// The current value of the [`WorkQuery`] filter for this axis, if set.
    /// The mapper compares each facet value's `id` against this string to
    /// decide whether the link is `active` and which way the href should
    /// toggle.
    pub fn current_filter<'a>(&self, query: &'a WorkQuery) -> Option<&'a str> {
        match self {
            FacetAxis::Author => query.author_id.as_deref(),
            FacetAxis::Tag => query.tag_id.as_deref(),
            FacetAxis::Genre => query.genre_id.as_deref(),
            FacetAxis::Subject => query.subject_id.as_deref(),
            FacetAxis::Publisher => query.publisher_id.as_deref(),
            FacetAxis::Format => query.format_id.as_deref(),
            FacetAxis::Language => query.language_id.as_deref(),
        }
    }

    /// Clear this axis's filter from `query`, leaving every other axis
    /// untouched. Used by the mapper when emitting a toggle-off href for
    /// the currently-active facet value.
    pub fn clear_filter(&self, query: &mut WorkQuery) {
        match self {
            FacetAxis::Author => query.author_id = None,
            FacetAxis::Tag => query.tag_id = None,
            FacetAxis::Genre => query.genre_id = None,
            FacetAxis::Subject => query.subject_id = None,
            FacetAxis::Publisher => query.publisher_id = None,
            FacetAxis::Format => query.format_id = None,
            FacetAxis::Language => query.language_id = None,
        }
    }

    /// Set this axis's primary filter to `value`. Mirror of
    /// [`Self::clear_filter`], used by the mapper when emitting a
    /// toggle-on href that replaces the active value with a new one.
    /// The `facet` / `value` tracking fields are not touched here —
    /// they're internal bookkeeping for the toggle-link builder and the
    /// library's display-name resolution, not the wire-format filter
    /// the client actually parses.
    pub fn set_filter(&self, query: &mut WorkQuery, value: String) {
        match self {
            FacetAxis::Author => query.author_id = Some(value),
            FacetAxis::Tag => query.tag_id = Some(value),
            FacetAxis::Genre => query.genre_id = Some(value),
            FacetAxis::Subject => query.subject_id = Some(value),
            FacetAxis::Publisher => query.publisher_id = Some(value),
            FacetAxis::Format => query.format_id = Some(value),
            FacetAxis::Language => query.language_id = Some(value),
        }
    }
}

/// A single (value, count) pair in a facet axis. `active` is true when the
/// value matches the corresponding filter on the active [`WorkQuery`], which
/// the mapper uses to flip the link's `rel` to `["self"]` so clients can
/// render a checkbox-like state.
#[derive(Debug, Clone)]
pub struct FacetCount {
    pub id: String,
    pub label: String,
    pub count: i64,
    pub active: bool,
}

/// One [`FacetCount`] per axis. An empty `Vec` means that axis is omitted
/// from the feed (or, in the mapper, the per-axis `Collection` is suppressed
/// when it would have fewer than 2 links). The fields are intentionally
/// public so handlers / services can inspect or merge the counts.
#[derive(Debug, Clone, Default)]
pub struct FacetCounts {
    pub authors: Vec<FacetCount>,
    pub tags: Vec<FacetCount>,
    pub genres: Vec<FacetCount>,
    pub subjects: Vec<FacetCount>,
    pub publishers: Vec<FacetCount>,
    pub formats: Vec<FacetCount>,
    pub languages: Vec<FacetCount>,
    /// Total number of works matching the parent query, regardless of the
    /// facet-axis filters. The mapper uses this to populate
    /// `Metadata::number_of_items` so clients can render "X of Y" labels
    /// against the unfiltered set. `None` means the repository didn't
    /// compute it (e.g. when only facet counts are requested).
    pub total: Option<u64>,
}

impl FacetCounts {
    /// Borrow the slice of [`FacetCount`]s for the given axis. Useful for
    /// the mapper, which iterates `FacetAxis::ALL` and looks up the right
    /// field without an `enum -> field` dispatch.
    pub fn for_axis(&self, axis: FacetAxis) -> &[FacetCount] {
        match axis {
            FacetAxis::Author => &self.authors,
            FacetAxis::Tag => &self.tags,
            FacetAxis::Genre => &self.genres,
            FacetAxis::Subject => &self.subjects,
            FacetAxis::Publisher => &self.publishers,
            FacetAxis::Format => &self.formats,
            FacetAxis::Language => &self.languages,
        }
    }
}

#[async_trait]
pub trait OpdsRepository: Send + Sync {
    async fn list_works(&self, query: &WorkQuery) -> Result<Vec<WorkAggregate>, OpdsError>;

    async fn get_work(&self, id: &str) -> Result<Option<WorkAggregate>, OpdsError>;

    async fn count_works(&self, query: &WorkQuery) -> Result<u64, OpdsError>;

    async fn list_authors(&self) -> Result<Vec<AuthorInfo>, OpdsError>;

    async fn list_genres(&self) -> Result<Vec<String>, OpdsError>;

    async fn list_tags(&self) -> Result<Vec<String>, OpdsError>;

    /// Resolve and return the bytes for a work's linked file.
    ///
    /// `work_id` is the work's ULID (matches the `id` field on
    /// [`WorkAggregate`]). `inventory_id`, when provided, pins the
    /// selection to a specific `digital_inventory` row; otherwise the
    /// first edition with a linked file (ordered by edition creation
    /// time, matching [`Self::get_work`]'s output) is used.
    ///
    /// Returns `(file_bytes, mime_type, filename)` on success:
    /// - `mime_type` is detected from the path's extension at query time
    ///   (and matches [`EditionAggregate::file_mime_type`]).
    /// - `filename` is the basename of `digital_inventory.file_path`.
    ///
    /// Errors:
    /// - [`OpdsError::MissingField`] when the work has no linked file,
    ///   when `inventory_id` doesn't match any edition, or when the
    ///   linked file is missing on disk (broken symlink). The pruning
    ///   helper is invoked on broken links so subsequent calls see a
    ///   consistent view.
    /// - [`OpdsError::Database`] for DB / I/O failures.
    async fn get_linked_file(
        &self,
        work_id: &str,
        inventory_id: Option<&str>,
    ) -> Result<(Vec<u8>, String, String), OpdsError>;

    /// Compute per-axis facet counts scoped to the active work set implied
    /// by `query`. Each axis returns at most 50 entries, sorted by `count`
    /// descending then `label` ascending. Values that match the current
    /// filter for their axis are flagged `active: true`.
    async fn compute_facets(
        &self,
        query: &WorkQuery,
        axes: &[FacetAxis],
    ) -> Result<FacetCounts, OpdsError>;

    /// Default convenience: compute every axis. Implementations can
    /// override if a faster path exists.
    async fn compute_all_facets(&self, query: &WorkQuery) -> Result<FacetCounts, OpdsError> {
        self.compute_facets(query, FacetAxis::ALL).await
    }
}
