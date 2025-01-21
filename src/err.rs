use chrono::ParseError;
use thiserror::Error;

pub type RepoResult<T> = Result<T, RepositoryError>;

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error("InternalLibSqlError {0}")]
    InternalLibSqlError(#[from] libsql::Error),
    #[error("Deserializing error {0}")]
    DeserializationError(#[from] serde::de::value::Error),
}

pub type StudyServiceResult<T> = Result<T, StudyServiceError>;

#[derive(Debug, Error)]
pub enum StudyServiceError {
    #[error("Repository error: {0}")]
    RepositoryError(#[from] RepositoryError),
    #[error("Error parsing date: {0}")]
    ParseDateError(#[from] ParseError),
}
