use thiserror::Error;

#[derive(Error, Debug)]
pub enum HelixDocsError {
    #[error("Source not found: {0}")]
    SourceNotFound(String),

    #[error("Source already exists: {0}")]
    SourceExists(String),

    #[error("Invalid source URL: {0}")]
    InvalidSourceUrl(String),

    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("Rate limited, retry after {0} seconds")]
    RateLimited(u64),

    #[error("Document not found: {0} in {1}")]
    DocumentNotFound(String, String),

    #[error("Library not found: {0}")]
    LibraryNotFound(String),

    #[error("No embeddings available, run `helix-docs ingest --embed` first")]
    NoEmbeddings,

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}

impl HelixDocsError {
    pub const fn exit_code(&self) -> i32 {
        match self {
            Self::SourceNotFound(_) | Self::DocumentNotFound { .. } => 1,
            Self::SourceExists(_) => 2,
            Self::InvalidSourceUrl(_) | Self::Config(_) => 3,
            Self::GitHubApi(_) | Self::RateLimited(_) => 4,
            Self::NoEmbeddings => 5,
            Self::LibraryNotFound(_) => 6,
            Self::Io(_) | Self::Serialization(_) | Self::Http(_) => 10,
        }
    }
}

pub type Result<T> = std::result::Result<T, HelixDocsError>;
