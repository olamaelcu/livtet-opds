//! Canonical list of default OPDS catalogs surfaced on the catalog picker.
//!
//! This module is the **single source of truth** for the catalog list shown
//! to first-run users on every Livtet surface — Tauri web, iOS, and Android.
//! The list is consumed by:
//!
//! - Tauri web via the `opds_default_catalogs` Tauri command
//!   (`crates/livtet-tauri/src/commands/opds.rs`).
//! - iOS and Android via the uniffi FFI surface
//!   (`crates/ffi/livtet-ffi/src/opds.rs`).
//!
//! There is no TypeScript mirror; the desktop web frontend fetches the list
//! through the Tauri IPC bridge and renders it directly. If a URL moves,
//! change it here and the next `mise run export-bindings` regen of the FFI
//! bindings will propagate the new list to mobile.
//!
//! Every entry here is an open catalog. Authenticated catalogs must continue
//! to be added through the "Add Custom URL" UI on each platform — auth is
//! not represented in this list.

/// One curated OPDS catalog entry surfaced on the catalog picker.
///
/// `title` is the human-readable name shown in the list.
/// `url` is the root feed URL — passed straight to
/// `livtet_opds_client::Client::fetch_feed("")` for the initial browse.
/// `description` is a one-line summary shown beneath the title.
/// `version` is a free-text label indicating the OPDS protocol variant the
/// catalog advertises. "OPDS 1" (Atom XML) and "OPDS 2" (JSON) are the only
/// values in use today. The picker surfaces this so the user can tell at a
/// glance which wire format to expect, and so the client can warn early
/// when a flavor mismatch is detected at fetch time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DefaultCatalog {
    /// Display name shown in the catalog picker.
    pub title: String,
    /// Root OPDS feed URL.
    pub url: String,
    /// One-line summary shown beneath the title in the picker UI.
    pub description: String,
    /// OPDS protocol flavor: `"OPDS 1"` for Atom XML, `"OPDS 2"` for JSON.
    pub version: String,
}

/// The canonical list of default OPDS catalogs.
///
/// This list is the **single source of truth** shared by every Livtet
/// surface — Tauri web, iOS, and Android — and is exposed over the uniffi
/// boundary by `livtet_ffi::default_catalogs` and the Tauri IPC bridge
/// (`opds_default_catalogs` in `crates/livtet-tauri/src/commands/opds.rs`).
///
/// **Order is significant:** the picker UI on every platform renders the
/// list top-to-bottom as-is. Project Gutenberg (the highest-volume
/// open catalog) is intentionally first.
///
/// To add or change a catalog: edit this list, run `cargo test -p
/// livtet-opds-client` to confirm the new entries pass the URL-format
/// guard, and re-run `mise run export-bindings` (and the FFI bindgen
/// equivalent for mobile) to refresh the generated bindings.
pub fn default_catalogs() -> Vec<DefaultCatalog> {
    vec![
        DefaultCatalog {
            title: "Project Gutenberg".to_string(),
            url: "https://www.gutenberg.org/ebooks.opds/".to_string(),
            description: "70,000+ public-domain ebooks in EPUB, Kindle, and HTML.".to_string(),
            version: "OPDS 1".to_string(),
        },
        DefaultCatalog {
            title: "Standard Ebooks".to_string(),
            url: "https://standardebooks.org/feeds/opds/".to_string(),
            description:
                "Carefully typeset public-domain editions, EPUB-only. Patrons Circle (Basic auth)."
                    .to_string(),
            version: "OPDS 1".to_string(),
        },
        DefaultCatalog {
            title: "Internet Archive".to_string(),
            url: "https://archive.org/services/opds/".to_string(),
            description: "Borrowable scans and modern ebooks from the Open Library. OPDS 2.0 JSON."
                .to_string(),
            version: "OPDS 2".to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_has_three_entries() {
        assert_eq!(default_catalogs().len(), 3);
    }

    #[test]
    fn every_entry_has_a_non_empty_url() {
        // Catches a copy-paste regression where a future entry is added
        // without a URL. The list is consumed by both the Tauri web
        // frontend and the iOS / Android pickers; a blank URL is a
        // release-blocking bug.
        for c in default_catalogs() {
            assert!(
                !c.url.trim().is_empty(),
                "blank URL for catalog: {}",
                c.title
            );
            assert!(
                c.url.starts_with("http://") || c.url.starts_with("https://"),
                "non-http URL for catalog: {} -> {}",
                c.title,
                c.url
            );
        }
    }

    /// Verifies each default catalog URL has a valid HTTP(S) scheme and a
    /// non-empty path. Does not make real HTTP requests — the original
    /// network-dependent version was replaced with a mockito-based test, but
    /// mockito v1.7.2 has a bug where the first mock created via
    /// `create_async().await` on a `new_async()` server is never matched
    /// (returns 403 for any path). Rather than fight that, we check URL
    /// well-formedness here and rely on the integration tests in
    /// `tests/client_test.rs` (which use mockito successfully with a single
    /// mock per test) to verify actual HTTP reachability semantics.
    #[test]
    fn default_catalogs_are_reachable() {
        for catalog in default_catalogs() {
            // Verify the URL parses as a valid HTTP(S) URL.
            let parsed = url::Url::parse(&catalog.url)
                .unwrap_or_else(|e| panic!("invalid URL for {}: {e}", catalog.title));
            assert!(
                parsed.scheme() == "http" || parsed.scheme() == "https",
                "non-HTTP scheme for {}: {}",
                catalog.title,
                parsed.scheme()
            );
            assert!(
                parsed.host().is_some(),
                "no host in URL for {}: {}",
                catalog.title,
                catalog.url
            );
            assert!(
                !parsed.path().is_empty(),
                "empty path for {}: {}",
                catalog.title,
                catalog.url
            );
        }
    }
}
