use crate::utils::error::{MongoLiteError, Result};
use mongodb::{Client, Database};

pub struct DatabaseService {
    client: Option<Client>,
}

impl DatabaseService {
    pub fn new() -> Self {
        Self { client: None }
    }

    pub async fn connect(&mut self, connection_string: &str) -> Result<()> {
        self.client = Some(Client::with_uri_str(connection_string).await?);
        Ok(())
    }

    pub async fn list_databases(&self) -> Result<Vec<String>> {
        if let Some(client) = &self.client {
            let db_names = client.list_database_names(None, None).await?;
            Ok(db_names)
        } else {
            Err(MongoLiteError::from("Not connected to any database"))
        }
    }

    pub fn get_database(&self, name: &str) -> Option<Database> {
        self.client.as_ref().map(|client| client.database(name))
    }
}
