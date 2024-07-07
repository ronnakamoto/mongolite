use redb::{Database, Error as RedbError, ReadableTable, TableDefinition, TableError};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

const PROFILES_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("profiles");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionProfile {
    pub id: String,
    pub name: String,
    pub connection_string: String,
}

pub struct ConnectionProfileManager {
    db: Database,
    profiles: Vec<ConnectionProfile>,
}

impl ConnectionProfileManager {
    pub fn new() -> Rc<RefCell<Self>> {
        let db_path = PathBuf::from("mongolite_profiles.redb");
        let db = Database::create(db_path).expect("Failed to create or open database");

        // Ensure the table exists
        let write_txn = db.begin_write().expect("Failed to begin write transaction");
        {
            write_txn
                .open_table(PROFILES_TABLE)
                .expect("Failed to open or create profiles table");
        }
        write_txn.commit().expect("Failed to commit transaction");

        let mut manager = Self {
            db,
            profiles: Vec::new(),
        };
        manager.load_profiles();
        Rc::new(RefCell::new(manager))
    }

    pub fn load_profiles(&mut self) {
        let read_txn = self
            .db
            .begin_read()
            .expect("Failed to begin read transaction");
        let table = read_txn
            .open_table(PROFILES_TABLE)
            .expect("Failed to open table");

        self.profiles.clear();
        for result in table.iter().expect("Failed to iterate over table") {
            let (_, value_bytes) = result.expect("Failed to read table entry");
            let profile: ConnectionProfile =
                bincode::deserialize(value_bytes.value()).expect("Failed to deserialize profile");
            self.profiles.push(profile);
        }
    }

    pub fn save_profile(&mut self, profile: &ConnectionProfile) {
        let serialized = bincode::serialize(profile).expect("Failed to serialize profile");

        let write_txn = self
            .db
            .begin_write()
            .expect("Failed to begin write transaction");
        {
            let mut table = write_txn
                .open_table(PROFILES_TABLE)
                .expect("Failed to open table");
            table
                .insert(&profile.id as &str, serialized.as_slice())
                .expect("Failed to insert profile");
        }
        write_txn.commit().expect("Failed to commit transaction");

        self.load_profiles();
    }

    pub fn delete_profile(&mut self, profile_id: &str) {
        let write_txn = self
            .db
            .begin_write()
            .expect("Failed to begin write transaction");
        {
            let mut table = write_txn
                .open_table(PROFILES_TABLE)
                .expect("Failed to open table");
            table.remove(profile_id).expect("Failed to remove profile");
        }
        write_txn.commit().expect("Failed to commit transaction");

        self.load_profiles();
    }

    pub fn get_profiles(&self) -> &[ConnectionProfile] {
        &self.profiles
    }
}
