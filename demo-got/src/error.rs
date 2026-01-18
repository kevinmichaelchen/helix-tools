//! Error types for demo-got.

use std::path::PathBuf;

/// Errors that can occur in demo-got.
#[derive(Debug, thiserror::Error)]
pub enum GotError {
    #[error("Person not found: {0}")]
    PersonNotFound(String),

    #[error("House not found: {0}")]
    HouseNotFound(String),

    #[error("Database not initialized. Run 'demo-got ingest' first.")]
    DatabaseNotInitialized,

    #[error("Failed to load family tree from {path}: {source}")]
    LoadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse YAML: {0}")]
    YamlError(#[from] serde_yaml::Error),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid relationship: {0}")]
    InvalidRelationship(String),

    #[error("Embedding error: {0}")]
    EmbeddingError(String),

    #[error("Vector search error: {0}")]
    VectorSearchError(String),
}

impl GotError {
    /// Returns an appropriate exit code for this error.
    #[must_use]
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::PersonNotFound(_) | Self::HouseNotFound(_) => 1,
            Self::DatabaseNotInitialized => 2,
            Self::LoadError { .. } | Self::YamlError(_) => 3,
            Self::DatabaseError(_) | Self::SerializationError(_) => 4,
            Self::IoError(_) => 5,
            Self::InvalidRelationship(_) => 6,
            Self::EmbeddingError(_) | Self::VectorSearchError(_) => 7,
        }
    }
}

/// Result type for demo-got operations.
pub type Result<T> = std::result::Result<T, GotError>;
