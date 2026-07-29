use livtet_opds_types::{Feed, OpdsError, Publication};

use crate::{
    SharedOpdsRepository,
    mapper::{DefaultOpdsMapper, Pagination},
    repository::WorkQuery,
};

pub struct PublicationsHandler {
    pub repository: SharedOpdsRepository,
    pub mapper: DefaultOpdsMapper,
}

impl PublicationsHandler {
    pub async fn handle(&self, query: &WorkQuery) -> Result<Feed, OpdsError> {
        let page = query.page.unwrap_or(1).max(1);
        let per_page = query.limit.unwrap_or(50).min(200);

        let mut q = query.clone();
        q.page = Some(page);
        q.limit = Some(per_page);

        // Run list, count, and facet computation concurrently. The three
        // operations are independent reads against the same SQLite
        // connection; `tokio::try_join!` polls them in parallel and
        // short-circuits on the first error. SQLite serializes writes
        // but concurrent reads from the same connection are allowed.
        let (works, total, counts) = tokio::try_join!(
            self.repository.list_works(&q),
            self.repository.count_works(&q),
            self.repository.compute_all_facets(&q),
        )?;

        let mut feed = self.mapper.works_to_feed(
            "Livtet Catalog",
            &works,
            Pagination {
                total,
                page,
                per_page,
            },
            &counts,
            &q,
        );
        feed.facets = self.mapper.build_facets(&self.mapper.base_url, &q, &counts);
        Ok(feed)
    }

    pub async fn get_publication(&self, id: &str) -> Result<Publication, OpdsError> {
        let work = self
            .repository
            .get_work(id)
            .await?
            .ok_or_else(|| OpdsError::MissingField(format!("work: {}", id)))?;
        Ok(self.mapper.work_to_publication(&work))
    }
}
