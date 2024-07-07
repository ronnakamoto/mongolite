use crate::utils::error::Result;
use futures_util::TryStreamExt;
use mongodb::{bson::Document, Collection};

pub struct QueryService;

impl QueryService {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute_query(
        &self,
        collection: &Collection<Document>,
        query: Document,
        projection: Option<Document>,
        sort: Option<Document>,
    ) -> Result<Vec<Document>> {
        let mut options = mongodb::options::FindOptions::default();
        options.projection = projection;
        options.sort = sort;

        let cursor = collection.find(query, options).await?;
        let results: Vec<Document> = cursor.try_collect().await?;
        Ok(results)
    }
}
