use thiserror::Error;

#[derive(Error, Debug)]
pub enum MongoLiteError {
    #[error("MongoDB error: {0}")]
    MongoDBError(#[from] mongodb::error::Error),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Unexpected error: {0}")]
    UnexpectedError(String),

    #[error("{0}")]
    StringError(String),
}

impl From<&str> for MongoLiteError {
    fn from(error: &str) -> Self {
        MongoLiteError::StringError(error.to_string())
    }
}

pub type Result<T> = std::result::Result<T, MongoLiteError>;
