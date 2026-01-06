use async_trait::async_trait;

use crate::domain::{Chunk, DocId, SearchQuery, SearchResult};
use crate::error::Result;

#[async_trait]
pub trait SearchIndex: Send + Sync {
    async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    async fn search_bm25(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>>;
    async fn search_vector(&self, embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    async fn index_document(&self, doc_id: &DocId, chunks: &[Chunk]) -> Result<()>;
    async fn remove_document(&self, doc_id: &DocId) -> Result<()>;
}
