//! Shared OPDS 1.x link-relation mapping.
//!
//! OPDS 1.x uses full URI relations (e.g.
//! `http://opds-spec.org/acquisition`). Inside Livtet's internal model we
//! store the short form (e.g. `acquisition`) to keep JSON payloads tidy and
//! to avoid hard-coding OPDS 1.x URIs in unrelated code. This module
//! translates between the two representations.

use std::borrow::Cow;

/// Base namespace for OPDS 1.x link relations.
pub const BASE: &str = "http://opds-spec.org";

/// Generic acquisition link relation.
pub const ACQUISITION: &str = "http://opds-spec.org/acquisition";

/// Open-access acquisition link relation.
pub const ACQUISITION_OPEN_ACCESS: &str = "http://opds-spec.org/acquisition/open-access";

/// Borrow acquisition link relation.
pub const ACQUISITION_BORROW: &str = "http://opds-spec.org/acquisition/borrow";

/// Buy acquisition link relation.
pub const ACQUISITION_BUY: &str = "http://opds-spec.org/acquisition/buy";

/// Sample acquisition link relation.
pub const ACQUISITION_SAMPLE: &str = "http://opds-spec.org/acquisition/sample";

/// Subscribe acquisition link relation.
pub const ACQUISITION_SUBSCRIBE: &str = "http://opds-spec.org/acquisition/subscribe";

/// Cover image link relation.
pub const IMAGE: &str = "http://opds-spec.org/image";

/// Thumbnail image link relation.
pub const IMAGE_THUMBNAIL: &str = "http://opds-spec.org/image/thumbnail";

/// Featured catalog link relation.
pub const FEATURED: &str = "http://opds-spec.org/featured";

/// Recommended catalog link relation.
pub const RECOMMENDED: &str = "http://opds-spec.org/recommended";

/// New publications catalog link relation.
pub const NEW: &str = "http://opds-spec.org/new";

/// Popular publications catalog link relation.
pub const POPULAR: &str = "http://opds-spec.org/popular";

/// Shelves / user collections link relation.
pub const SHELVES: &str = "http://opds-spec.org/shelves";

/// Convert a short OPDS relation into its full Atom URI.
///
/// # Examples
///
/// ```
/// use livtet_opds_atom::rel::rel_to_atom;
///
/// assert_eq!(rel_to_atom("acquisition"), "http://opds-spec.org/acquisition");
/// assert_eq!(rel_to_atom("self"), "self");
/// ```
pub fn rel_to_atom(short: &str) -> Cow<'_, str> {
    match short {
        "acquisition" => Cow::Borrowed(ACQUISITION),
        "open-access" => Cow::Borrowed(ACQUISITION_OPEN_ACCESS),
        "borrow" => Cow::Borrowed(ACQUISITION_BORROW),
        "buy" => Cow::Borrowed(ACQUISITION_BUY),
        "sample" => Cow::Borrowed(ACQUISITION_SAMPLE),
        "subscribe" => Cow::Borrowed(ACQUISITION_SUBSCRIBE),
        "image" => Cow::Borrowed(IMAGE),
        "image/thumbnail" => Cow::Borrowed(IMAGE_THUMBNAIL),
        "featured" => Cow::Borrowed(FEATURED),
        "recommended" => Cow::Borrowed(RECOMMENDED),
        "new" => Cow::Borrowed(NEW),
        "popular" => Cow::Borrowed(POPULAR),
        "shelves" => Cow::Borrowed(SHELVES),
        // Standard Atom / IANA relations and unknown values pass through unchanged.
        _ => Cow::Borrowed(short),
    }
}

/// Convert a full OPDS 1.x Atom URI relation into the short internal form.
///
/// # Examples
///
/// ```
/// use livtet_opds_atom::rel::rel_from_atom;
///
/// assert_eq!(rel_from_atom("http://opds-spec.org/acquisition"), "acquisition");
/// assert_eq!(rel_from_atom("self"), "self");
/// ```
pub fn rel_from_atom(full: &str) -> Cow<'_, str> {
    match full {
        ACQUISITION => Cow::Borrowed("acquisition"),
        ACQUISITION_OPEN_ACCESS => Cow::Borrowed("open-access"),
        ACQUISITION_BORROW => Cow::Borrowed("borrow"),
        ACQUISITION_BUY => Cow::Borrowed("buy"),
        ACQUISITION_SAMPLE => Cow::Borrowed("sample"),
        ACQUISITION_SUBSCRIBE => Cow::Borrowed("subscribe"),
        IMAGE => Cow::Borrowed("image"),
        IMAGE_THUMBNAIL => Cow::Borrowed("image/thumbnail"),
        FEATURED => Cow::Borrowed("featured"),
        RECOMMENDED => Cow::Borrowed("recommended"),
        NEW => Cow::Borrowed("new"),
        POPULAR => Cow::Borrowed("popular"),
        SHELVES => Cow::Borrowed("shelves"),
        // Unknown values pass through unchanged.
        _ => Cow::Borrowed(full),
    }
}
