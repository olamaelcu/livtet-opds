use livtet_opds_types::{Feed, OpdsError};

use crate::SharedOpdsRepository;

pub struct NavigationHandler {
    pub repository: SharedOpdsRepository,
    pub base_path: String,
}

impl NavigationHandler {
    pub async fn handle_authors(&self) -> Result<Feed, OpdsError> {
        let authors = self.repository.list_authors().await?;
        let items: Vec<(String, String)> = authors
            .into_iter()
            .map(|a| (a.name, format!("{}?author_id={}", self.base_path, a.id)))
            .collect();
        Ok(crate::mapper::DefaultOpdsMapper::new("").navigation_feed("Authors", items))
    }

    pub async fn handle_tags(&self) -> Result<Feed, OpdsError> {
        let tags = self.repository.list_tags().await?;
        let items: Vec<(String, String)> = tags
            .into_iter()
            .map(|t| (t.clone(), format!("{}?tag_id={}", self.base_path, t)))
            .collect();
        Ok(crate::mapper::DefaultOpdsMapper::new("").navigation_feed("Tags", items))
    }

    pub async fn handle_genres(&self) -> Result<Feed, OpdsError> {
        let genres = self.repository.list_genres().await?;
        let items: Vec<(String, String)> = genres
            .into_iter()
            .map(|g| (g.clone(), format!("{}?genre_id={}", self.base_path, g)))
            .collect();
        Ok(crate::mapper::DefaultOpdsMapper::new("").navigation_feed("Genres", items))
    }
}
