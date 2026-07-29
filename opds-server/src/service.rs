use livtet_opds_types::{Feed, OpdsError, Publication};

use crate::{
    SharedOpdsRepository,
    handlers::{
        navigation::NavigationHandler, publications::PublicationsHandler, search::SearchHandler,
    },
    mapper::DefaultOpdsMapper,
};

pub struct OpdsService {
    pub repository: SharedOpdsRepository,
    pub mapper: DefaultOpdsMapper,
    pub publications: PublicationsHandler,
    pub navigation: NavigationHandler,
    pub search: SearchHandler,
}

impl OpdsService {
    pub fn new(repository: SharedOpdsRepository, base_url: String) -> Self {
        let mapper = DefaultOpdsMapper::new(base_url.clone());
        let publications = PublicationsHandler {
            repository: repository.clone(),
            mapper: mapper.clone(),
        };
        let base_path = url::Url::parse(&base_url)
            .map(|u| u.path().to_string())
            .unwrap_or_else(|_| "/opds".to_string());
        let navigation = NavigationHandler {
            repository: repository.clone(),
            base_path,
        };
        let search = SearchHandler {
            repository: repository.clone(),
            mapper: mapper.clone(),
        };
        Self {
            repository,
            mapper,
            publications,
            navigation,
            search,
        }
    }

    pub async fn get_feed(&self) -> Result<Feed, OpdsError> {
        self.publications.handle(&Default::default()).await
    }

    pub async fn get_publication(&self, id: &str) -> Result<Publication, OpdsError> {
        self.publications.get_publication(id).await
    }

    pub async fn search(&self, query: &str) -> Result<Feed, OpdsError> {
        self.search.handle(query).await
    }

    pub async fn get_linked_file(
        &self,
        work_id: &str,
        inventory_id: Option<&str>,
    ) -> Result<(Vec<u8>, String, String), OpdsError> {
        self.repository.get_linked_file(work_id, inventory_id).await
    }
}
