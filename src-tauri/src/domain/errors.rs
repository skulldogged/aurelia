use thiserror::Error;

#[derive(Error, Debug)]
pub enum DomainError {
    #[error("Item not found: {0}")]
    NotFound(String),

    #[error("Jellyfin API error: {0}")]
    ApiError(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Sync failed: {0}")]
    SyncError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl From<redb::Error> for DomainError {
    fn from(err: redb::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<redb::TableError> for DomainError {
    fn from(err: redb::TableError) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<redb::StorageError> for DomainError {
    fn from(err: redb::StorageError) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<redb::TransactionError> for DomainError {
    fn from(err: redb::TransactionError) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<redb::CommitError> for DomainError {
    fn from(err: redb::CommitError) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<anyhow::Error> for DomainError {
    fn from(err: anyhow::Error) -> Self {
        Self::Unknown(err.to_string())
    }
}
