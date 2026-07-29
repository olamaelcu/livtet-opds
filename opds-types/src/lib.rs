pub mod auth;
pub mod error;
pub mod feed;
pub mod link;
pub mod publication;

pub use auth::{AuthenticationDocument, AuthenticationMethod};
pub use error::OpdsError;
pub use feed::{Collection, Feed, Metadata};
pub use link::{Acquisition, Availability, Link, Price, Properties};
pub use publication::{Author, BelongsTo, Publication, PublicationMetadata, Series};
