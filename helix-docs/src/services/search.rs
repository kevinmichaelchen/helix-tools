use std::sync::Arc;

use crate::domain::{SearchMode, SearchQuery, SearchResult};
use crate::error::Result;
use crate::ports::{EmbeddingGenerator, SearchIndex};

pub struct SearchService<I, E>
where
    I: SearchIndex,
    E: EmbeddingGenerator,
{
    index: Arc<I>,
    embedder: Arc<E>,
}

impl<I, E> SearchService<I, E>
where
    I: SearchIndex,
    E: EmbeddingGenerator,
{
    pub const fn new(index: Arc<I>, embedder: Arc<E>) -> Self {
        Self { index, embedder }
    }

    pub async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        match query.mode {
            SearchMode::Word => self.search_bm25(&query).await,
            SearchMode::Vector => self.search_vector(&query).await,
            SearchMode::Hybrid => self.search_hybrid(&query).await,
        }
    }

    async fn search_bm25(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        self.index.search_bm25(&query.query, query.limit).await
    }

    async fn search_vector(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let embedding = self.embedder.embed(&query.query).await?;
        self.index.search_vector(&embedding, query.limit).await
    }

    async fn search_hybrid(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        let expanded_limit = query.limit * 2;

        let (bm25_results, vector_results) = tokio::join!(
            self.index.search_bm25(&query.query, expanded_limit),
            async {
                let embedding = self.embedder.embed(&query.query).await?;
                self.index.search_vector(&embedding, expanded_limit).await
            }
        );

        let bm25 = bm25_results?;
        let vector = vector_results?;

        let fused = Self::rrf_fusion(&bm25, &vector, 60.0);

        Ok(fused.into_iter().take(query.limit).collect())
    }

    #[allow(clippy::cast_precision_loss)] // rank values are small, precision loss is acceptable
    fn rrf_fusion(bm25: &[SearchResult], vector: &[SearchResult], k: f32) -> Vec<SearchResult> {
        use std::collections::HashMap;

        let mut scores: HashMap<String, (f32, Option<SearchResult>)> = HashMap::new();

        for (rank, result) in bm25.iter().enumerate() {
            let key = result.chunk_id.to_string();
            let score = 1.0 / (k + rank as f32);
            scores
                .entry(key)
                .and_modify(|(s, _)| *s += score)
                .or_insert_with(|| (score, Some(result.clone())));
        }

        for (rank, result) in vector.iter().enumerate() {
            let key = result.chunk_id.to_string();
            let score = 1.0 / (k + rank as f32);
            scores
                .entry(key)
                .and_modify(|(s, _)| *s += score)
                .or_insert_with(|| (score, Some(result.clone())));
        }

        let mut results: Vec<_> = scores
            .into_iter()
            .filter_map(|(_, (score, result))| {
                result.map(|mut r| {
                    r.score = score;
                    r
                })
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }
}
