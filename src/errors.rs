use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConnectionManagerError {
    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),
    #[error("MongoDB error: {0}")]
    MongoDBError(String),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    #[error("Encryption error: {0}")]
    EncryptionError(#[from] EncryptionError),
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Cipher creation failed")]
    CipherCreationFailed,
    #[error("Encryption failed")]
    EncryptionFailed,
    #[error("Decryption failed")]
    DecryptionFailed,
    #[error("Invalid ciphertext")]
    InvalidCiphertext,
    #[error("Invalid UTF-8")]
    InvalidUtf8,
    #[error("Invalid hex encoding")]
    InvalidHex,
}
