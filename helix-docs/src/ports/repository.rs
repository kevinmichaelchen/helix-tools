use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::domain::{Chunk, ChunkId, DocId, Document, Source, SourceId};
use crate::error::Result;

#[async_trait]
pub trait SourceRepository: Send + Sync {
    async fn create(&self, source: &Source) -> Result<SourceId>;
    async fn get(&self, id: &SourceId) -> Result<Option<Source>>;
    async fn get_by_url(&self, url: &str) -> Result<Option<Source>>;
    async fn list(&self) -> Result<Vec<Source>>;
    async fn update(&self, source: &Source) -> Result<()>;
    async fn delete(&self, id: &SourceId) -> Result<()>;
}

#[async_trait]
pub trait DocumentRepository: Send + Sync {
    async fn upsert(&self, doc: &Document) -> Result<DocId>;
    async fn get(&self, id: &DocId) -> Result<Option<Document>>;
    async fn get_by_path(&self, source_id: &SourceId, path: &str) -> Result<Option<Document>>;
    async fn list_by_source(&self, source_id: &SourceId) -> Result<Vec<Document>>;
    async fn list_by_library(&self, pattern: &str) -> Result<Vec<Document>>;
    async fn delete(&self, id: &DocId) -> Result<()>;
    async fn delete_by_source(&self, source_id: &SourceId) -> Result<()>;
    async fn list_stale(&self, since: DateTime<Utc>) -> Result<Vec<Document>>;
}

#[async_trait]
pub trait ChunkRepository: Send + Sync {
    async fn create_for_document(&self, doc_id: &DocId, chunks: &[Chunk]) -> Result<()>;
    async fn get_by_document(&self, doc_id: &DocId) -> Result<Vec<Chunk>>;
    async fn get(&self, id: &ChunkId) -> Result<Option<Chunk>>;
    async fn count_without_embeddings(&self) -> Result<usize>;
    async fn list_needing_embeddings(&self, limit: usize, offset: usize) -> Result<Vec<Chunk>>;
}

#[async_trait]
pub trait EmbeddingRepository: Send + Sync {
    async fn store(&self, embeddings: &[(ChunkId, Vec<f32>)]) -> Result<()>;
    async fn has_embedding(&self, chunk_id: &ChunkId) -> Result<bool>;
    async fn delete_by_document(&self, doc_id: &DocId) -> Result<()>;
}
