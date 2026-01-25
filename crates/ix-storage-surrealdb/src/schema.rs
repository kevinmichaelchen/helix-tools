//! `SurrealDB` schema definitions for Ixchel entities.

/// Initialize the database schema.
///
/// Creates:
/// - `entity` table with SCHEMAFULL mode
/// - `sync_manifest` table for incremental sync tracking
/// - HNSW vector index for similarity search
/// - Relationship edge definitions
pub const SCHEMA_INIT: &str = r"
-- Use the ixchel namespace and database
DEFINE NAMESPACE IF NOT EXISTS ixchel;
USE NS ixchel;
DEFINE DATABASE IF NOT EXISTS main;
USE DB main;

-- Entity table (SCHEMAFULL for strict typing)
DEFINE TABLE IF NOT EXISTS entity SCHEMAFULL;

-- Entity fields
DEFINE FIELD IF NOT EXISTS entity_id ON entity TYPE string ASSERT $value != NONE;
DEFINE FIELD IF NOT EXISTS kind ON entity TYPE string;
DEFINE FIELD IF NOT EXISTS title ON entity TYPE string;
DEFINE FIELD IF NOT EXISTS status ON entity TYPE string;
DEFINE FIELD IF NOT EXISTS file_path ON entity TYPE string;
DEFINE FIELD IF NOT EXISTS content_hash ON entity TYPE string;
DEFINE FIELD IF NOT EXISTS tags ON entity TYPE array<string>;
DEFINE FIELD IF NOT EXISTS body ON entity TYPE string;
DEFINE FIELD IF NOT EXISTS embedding ON entity TYPE array<float>;

-- Unique index on entity_id field
DEFINE INDEX IF NOT EXISTS entity_id_idx ON entity FIELDS entity_id UNIQUE;

-- Relationship table (for graph edges)
DEFINE TABLE IF NOT EXISTS relates SCHEMAFULL TYPE RELATION IN entity OUT entity;
DEFINE FIELD IF NOT EXISTS label ON relates TYPE string;
DEFINE INDEX IF NOT EXISTS relates_label_idx ON relates FIELDS label;

-- Sync manifest table for incremental sync tracking
DEFINE TABLE IF NOT EXISTS sync_manifest SCHEMAFULL;
DEFINE FIELD IF NOT EXISTS entity_id ON sync_manifest TYPE string ASSERT $value != NONE;
DEFINE FIELD IF NOT EXISTS content_hash ON sync_manifest TYPE string;
DEFINE FIELD IF NOT EXISTS file_path ON sync_manifest TYPE string;
DEFINE FIELD IF NOT EXISTS last_synced ON sync_manifest TYPE int;
DEFINE INDEX IF NOT EXISTS manifest_entity_id_idx ON sync_manifest FIELDS entity_id UNIQUE;
";

/// Create HNSW vector index with the specified dimension.
///
/// This must be called after schema init with the actual embedding dimension.
/// Uses cosine distance for semantic similarity search.
pub fn hnsw_index_query(dimension: usize) -> String {
    // HNSW parameters:
    // - DIMENSION: embedding vector size
    // - DIST COSINE: use cosine distance (smaller = more similar)
    // - M 16: max connections per layer (default)
    // - EFC 150: ef_construction (index build quality)
    format!(
        r"DEFINE INDEX IF NOT EXISTS entity_embedding_idx ON entity FIELDS embedding HNSW DIMENSION {dimension} DIST COSINE M 16 EFC 150;"
    )
}
