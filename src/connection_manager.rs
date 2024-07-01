use crate::encryption::{decrypt_connection_string, encrypt_connection_string};
use crate::errors::ConnectionManagerError;
use log::info;
use mongodb::{options::ClientOptions, Client};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::runtime::Runtime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub connection_string: String,
}

pub struct ConnectionManager {
    conn: Connection,
}

impl ConnectionManager {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, ConnectionManagerError> {
        let conn = Connection::open(db_path)?;
        let manager = ConnectionManager { conn };
        manager.initialize_table()?;
        Ok(manager)
    }

    fn initialize_table(&self) -> Result<(), ConnectionManagerError> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS connection_profiles (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                connection_string TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    pub fn add_profile(
        &self,
        profile: &ConnectionProfile,
        key: &[u8],
    ) -> Result<(), ConnectionManagerError> {
        let encrypted_string = encrypt_connection_string(&profile.connection_string, key)
            .map_err(ConnectionManagerError::EncryptionError)?;
        self.conn.execute(
            "INSERT INTO connection_profiles (id, name, connection_string) VALUES (?, ?, ?)",
            params![profile.id, profile.name, encrypted_string],
        )?;
        Ok(())
    }

    pub fn update_profile(
        &self,
        profile: &ConnectionProfile,
        key: &[u8],
    ) -> Result<(), ConnectionManagerError> {
        let encrypted_string = encrypt_connection_string(&profile.connection_string, key)
            .map_err(ConnectionManagerError::EncryptionError)?;
        self.conn.execute(
            "UPDATE connection_profiles SET name = ?, connection_string = ? WHERE id = ?",
            params![profile.name, encrypted_string, profile.id],
        )?;
        Ok(())
    }

    pub fn delete_profile(&self, id: &str) -> Result<(), ConnectionManagerError> {
        self.conn
            .execute("DELETE FROM connection_profiles WHERE id = ?", params![id])?;
        info!("Deleted connection profile with id: {}", id);
        Ok(())
    }

    pub fn get_profiles(
        &self,
        key: &[u8],
    ) -> Result<Vec<ConnectionProfile>, ConnectionManagerError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, connection_string FROM connection_profiles ORDER BY name")?;

        let profile_iter = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            let encrypted_string: String = row.get(2)?;
            Ok((id, name, encrypted_string))
        })?;

        let mut profiles = Vec::new();
        for result in profile_iter {
            let (id, name, encrypted_string) = result?;
            let connection_string = decrypt_connection_string(&encrypted_string, key)
                .map_err(ConnectionManagerError::EncryptionError)?;
            profiles.push(ConnectionProfile {
                id,
                name,
                connection_string,
            });
        }

        Ok(profiles)
    }

    pub fn test_connection(&self, connection_string: &str) -> Result<(), ConnectionManagerError> {
        let runtime =
            Runtime::new().map_err(|e| ConnectionManagerError::RuntimeError(e.to_string()))?;

        runtime.block_on(async {
            let client_options = ClientOptions::parse(connection_string)
                .await
                .map_err(|e| ConnectionManagerError::MongoDBError(e.to_string()))?;

            Client::with_options(client_options)
                .map_err(|e| ConnectionManagerError::MongoDBError(e.to_string()))?
                .list_database_names(None, None)
                .await
                .map_err(|e| ConnectionManagerError::MongoDBError(e.to_string()))?;

            Ok(())
        })
    }
}
