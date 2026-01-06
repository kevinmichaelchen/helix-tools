use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{DocId, SourceId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: DocId,
    pub source_id: SourceId,
    pub path: String,
    pub title: Option<String>,
    pub content: String,
    pub content_hash: String,
    pub version: Option<String>,
    pub fetched_at: DateTime<Utc>,
    pub last_accessed_at: DateTime<Utc>,
    pub metadata: DocumentMetadata,
}

impl Document {
    pub fn new(source_id: SourceId, path: String, content: String) -> Self {
        let content_hash = blake3::hash(content.as_bytes()).to_hex().to_string();
        let metadata = DocumentMetadata {
            size_bytes: content.len(),
            line_count: content.lines().count(),
            ..Default::default()
        };

        Self {
            id: DocId::generate(),
            source_id,
            path,
            title: None,
            content,
            content_hash,
            version: None,
            fetched_at: Utc::now(),
            last_accessed_at: Utc::now(),
            metadata,
        }
    }

    #[must_use]
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub language: Option<String>,
    pub size_bytes: usize,
    pub line_count: usize,
}
