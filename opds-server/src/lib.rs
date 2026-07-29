pub mod auth;
pub mod content_negotiation;
pub mod handler;
pub mod handlers;
pub mod mapper;
pub mod poem_server;
pub mod renderers;
pub mod repository;
pub mod repository_live;
pub mod routes;
pub mod service;

pub use auth::AuthConfig;
pub use handler::{OpdsHandler, OpdsServer};
pub use mapper::DefaultOpdsMapper;
pub use poem_server::PoemOpdsServer;
pub use repository::{
    AuthorInfo, EditionAggregate, FacetAxis, FacetCount, FacetCounts, OpdsRepository, SeriesInfo,
    WorkAggregate, WorkQuery,
};
pub use repository_live::LivtetOpdsRepository;
pub use service::OpdsService;

/// Convenience alias for `Arc<dyn OpdsRepository>`, the trait-object
/// shape shared by every handler that takes a repository.
pub type SharedOpdsRepository = std::sync::Arc<dyn OpdsRepository>;

/// Convenience alias for `Arc<dyn OpdsHandler>`, the trait-object
/// shape wired into `PoemOpdsServer` and the route handlers.
pub type SharedOpdsHandler = std::sync::Arc<dyn OpdsHandler>;
