//! `SurrealDB` record types for Ixchel entities.

use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

/// Entity record stored in `SurrealDB`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRecord {
    /// `SurrealDB` record ID (e.g., `entity:src-abc123`)
    #[serde(skip_serializing)]
    pub record_id: Option<RecordId>,

    /// Entity identifier (e.g., `src-abc123`)
    pub entity_id: String,

    /// Entity kind (decision, issue, source, etc.)
    pub kind: String,

    /// Entity title
    pub title: String,

    /// Entity status (e.g., "accepted", "open")
    pub status: String,

    /// File path relative to repo root
    pub file_path: String,

    /// Content hash for change detection
    pub content_hash: String,

    /// Tags associated with the entity
    pub tags: Vec<String>,

    /// Markdown body content
    pub body: String,

    /// Embedding vector for similarity search
    pub embedding: Vec<f32>,
}

/// Search result from vector similarity query.
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    /// Entity identifier
    pub entity_id: String,

    /// Entity kind
    pub kind: Option<String>,

    /// Entity title
    pub title: String,

    /// Distance from query vector (lower = more similar for cosine distance)
    pub distance: f64,
}

/// Result of an outgoing/incoming neighbor query.
#[derive(Debug, Clone, Deserialize)]
pub struct NeighborResult {
    /// Entity identifier of the neighbor
    pub entity_id: String,
}

/// Manifest record stored in `SurrealDB` for incremental sync tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestRecord {
    /// Entity identifier
    pub entity_id: String,
    /// Content hash (blake3) of the file at last sync
    pub content_hash: String,
    /// Relative file path from repo root
    pub file_path: String,
    /// Unix timestamp (seconds) of last sync
    pub last_synced: i64,
}
