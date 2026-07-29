//! OPDS 1.x Atom XML parse + serialize support.
//!
//! This crate lives between the format-agnostic
//! [`livtet-opds-types`](livtet_opds_types) types and the wire format. It
//! converts real-world OPDS 1.x Atom catalogs into the shared types and
//! renders those types back to valid OPDS 1.x Atom XML.

pub mod opensearch;
pub mod parse;
pub mod rel;
pub mod serialize;

pub use opensearch::{OpenSearchDescription, parse_description, render_description};
pub use parse::{parse_entry, parse_feed};
pub use rel::{rel_from_atom, rel_to_atom};
pub use serialize::{FeedKind, render_entry, render_feed};
