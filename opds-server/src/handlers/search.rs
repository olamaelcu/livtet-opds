use livtet_opds_types::{Feed, OpdsError};

use crate::{
    SharedOpdsRepository,
    mapper::{DefaultOpdsMapper, Pagination},
    repository::WorkQuery,
};

pub struct SearchHandler {
    pub repository: SharedOpdsRepository,
    pub mapper: DefaultOpdsMapper,
}

impl SearchHandler {
    pub async fn handle(&self, query: &str) -> Result<Feed, OpdsError> {
        let work_query = WorkQuery {
            search_query: Some(query.to_string()),
            page: Some(1),
            limit: Some(50),
            ..Default::default()
        };

        let works = self.repository.list_works(&work_query).await?;
        let total = self.repository.count_works(&work_query).await?;

        Ok(self.mapper.works_to_feed(
            "Livtet Search",
            &works,
            Pagination {
                total,
                page: 1,
                per_page: 50,
            },
            &crate::repository::FacetCounts::default(),
            &work_query,
        ))
    }
}
