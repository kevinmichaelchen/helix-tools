//! Main search logic.

use crate::delta::compute_delta;
use crate::embeddings::{Embedder, create_embedder};
use crate::loader::load_decisions;
use crate::storage::{DecisionStorage, HelixDecisionStorage};
use crate::types::{ChainResponse, RelatedResponse, SearchResponse, SearchResult, Status};
use anyhow::Result;
use std::path::Path;

pub struct DecisionSearcher {
    storage: Box<dyn DecisionStorage>,
    embedder: Embedder,
}

impl DecisionSearcher {
    pub fn new(repo_root: &Path) -> Result<Self> {
        let storage = Box::new(HelixDecisionStorage::open(repo_root)?);
        let embedder = create_embedder()?;
        Ok(Self { storage, embedder })
    }

    pub fn sync(&mut self, dir: &Path) -> Result<()> {
        let decisions = load_decisions(dir)?;
        let stored_hashes = self.storage.get_hashes()?;
        let delta = compute_delta(decisions, stored_hashes);

        if !delta.to_remove.is_empty() {
            self.storage.remove(delta.to_remove)?;
        }

        if !delta.to_add.is_empty() {
            let mut decisions_with_embeddings = Vec::new();
            for mut decision in delta.to_add {
                let embedding = self.embedder.embed(&decision.body)?;
                decision.embedding = Some(embedding);
                decisions_with_embeddings.push(decision);
            }
            self.storage.index(decisions_with_embeddings)?;
        }

        Ok(())
    }

    pub fn search(
        &self,
        query: &str,
        limit: usize,
        status_filter: Option<Status>,
        tags_filter: Option<Vec<String>>,
    ) -> Result<SearchResponse> {
        let query_embedding = self.embedder.embed(query)?;
        let results = self.storage.search(query_embedding, limit * 2)?;

        let search_results: Vec<SearchResult> = results
            .into_iter()
            .filter(|(decision, _)| {
                if let Some(ref status) = status_filter
                    && &decision.metadata.status != status
                {
                    return false;
                }
                if let Some(ref tags) = tags_filter
                    && !tags.iter().all(|t| decision.metadata.tags.contains(t))
                {
                    return false;
                }
                true
            })
            .take(limit)
            .map(|(decision, score)| SearchResult {
                id: decision.metadata.id,
                uuid: decision.metadata.uuid,
                title: decision.metadata.title,
                status: decision.metadata.status,
                score,
                tags: decision.metadata.tags,
                date: decision.metadata.date,
                deciders: decision.metadata.deciders,
                file_path: decision.file_path,
                related: Vec::new(),
            })
            .collect();

        Ok(SearchResponse {
            query: query.to_string(),
            count: search_results.len(),
            results: search_results,
        })
    }

    pub fn get_chain(&self, decision_id: u32) -> Result<ChainResponse> {
        let chain = self.storage.get_chain(decision_id)?;
        Ok(ChainResponse {
            root_id: decision_id,
            chain,
        })
    }

    pub fn get_related(&self, decision_id: u32) -> Result<RelatedResponse> {
        let related = self.storage.get_related(decision_id)?;
        Ok(RelatedResponse {
            decision_id,
            related,
        })
    }
}
