//! Main search logic.

use crate::delta::compute_delta;
use crate::embeddings::{Embedder, create_embedder};
use crate::helix_backend::SyncStats;
use crate::loader::load_decisions;
use crate::storage::{DecisionStorage, HelixDecisionStorage};
use crate::types::{ChainResponse, RelatedResponse, SearchResponse, SearchResult, Status};
use anyhow::Result;
use std::path::Path;
use std::time::Instant;

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

    pub fn sync(&mut self, dir: &Path) -> Result<SyncStats> {
        let start = Instant::now();
        let decisions = load_decisions(dir)?;
        let scanned = decisions.len() as u32;
        let stored_hashes = self.storage.get_hashes()?;
        let repo_root = self.storage.repo_root().to_path_buf();
        let delta = compute_delta(
            &repo_root,
            decisions,
            stored_hashes,
            self.storage.manifest(),
        );

        let deleted = delta.to_remove.len() as u32;
        if !delta.to_remove.is_empty() {
            self.storage.remove(delta.to_remove)?;
        }

        let renamed = delta.renamed.len() as u32;
        for (old_path, new_path) in &delta.renamed {
            self.storage.handle_rename(old_path, new_path)?;
        }

        let added = delta.to_add.len() as u32;
        let modified = delta.to_modify.len() as u32;

        let all_to_index: Vec<_> = delta.to_add.into_iter().chain(delta.to_modify).collect();

        if !all_to_index.is_empty() {
            let mut decisions_with_embeddings = Vec::new();
            for mut decision in all_to_index {
                let embedding = self.embedder.embed(&decision.body)?;
                decision.embedding = Some(embedding);
                decisions_with_embeddings.push(decision);
            }
            self.storage.index(decisions_with_embeddings)?;
        }

        Ok(SyncStats {
            scanned,
            added,
            modified,
            deleted,
            renamed,
            unchanged: delta.unchanged_count,
            errors: 0,
            duration_ms: start.elapsed().as_millis() as u64,
        })
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
