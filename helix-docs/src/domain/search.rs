use serde::{Deserialize, Serialize};

use super::{ChunkId, ChunkPosition, DocId};

#[derive(Debug, Clone)]
pub struct SearchQuery {
    pub query: String,
    pub library: Option<String>,
    pub version: Option<String>,
    pub mode: SearchMode,
    pub limit: usize,
}

impl SearchQuery {
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            library: None,
            version: None,
            mode: SearchMode::default(),
            limit: 10,
        }
    }

    #[must_use]
    pub fn with_library(mut self, library: impl Into<String>) -> Self {
        self.library = Some(library.into());
        self
    }

    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    #[must_use]
    pub const fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        self
    }

    #[must_use]
    pub const fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum SearchMode {
    #[default]
    Hybrid,
    Word,
    Vector,
}

impl std::str::FromStr for SearchMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "hybrid" => Ok(Self::Hybrid),
            "word" | "bm25" | "keyword" => Ok(Self::Word),
            "vector" | "semantic" | "embedding" => Ok(Self::Vector),
            _ => Err(format!("Unknown search mode: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk_id: ChunkId,
    pub doc_id: DocId,
    pub doc_path: String,
    pub doc_title: Option<String>,
    pub library: String,
    pub version: Option<String>,
    pub text: String,
    pub score: f32,
    pub position: ChunkPosition,
}
