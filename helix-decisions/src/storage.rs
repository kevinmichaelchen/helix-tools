use crate::types::{ChainNode, Decision, DecisionMetadata, RelatedDecision, RelationType};
use anyhow::Result;
use helix_storage::{JsonFileBackend, StorageConfig, StorageNode, VectorStorage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub trait DecisionStorage: Send + Sync {
    fn index(&mut self, decisions: Vec<Decision>) -> Result<()>;
    fn remove(&mut self, paths: Vec<String>) -> Result<()>;
    fn search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Decision, f32)>>;
    fn get_hashes(&self) -> Result<HashMap<String, String>>;
    fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>>;
    fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredDecision {
    metadata: DecisionMetadata,
    body: String,
    file_path: String,
}

impl From<&Decision> for StoredDecision {
    fn from(d: &Decision) -> Self {
        Self {
            metadata: d.metadata.clone(),
            body: d.body.clone(),
            file_path: d.file_path.to_string_lossy().to_string(),
        }
    }
}

impl StoredDecision {
    fn to_decision(&self, embedding: Option<Vec<f32>>, content_hash: &str) -> Decision {
        Decision {
            metadata: self.metadata.clone(),
            body: self.body.clone(),
            file_path: PathBuf::from(&self.file_path),
            content_hash: content_hash.to_string(),
            embedding,
        }
    }
}

pub struct PersistentDecisionStorage {
    backend: JsonFileBackend<StoredDecision>,
    decisions_cache: Vec<Decision>,
    decision_id_to_idx: HashMap<u32, usize>,
}

impl PersistentDecisionStorage {
    pub fn open() -> Result<Self> {
        let config = StorageConfig::project_local("decisions")
            .map_err(|e| anyhow::anyhow!("Failed to create storage config: {e}"))?;
        Self::open_with_config(config)
    }

    pub fn open_with_config(config: StorageConfig) -> Result<Self> {
        let backend = JsonFileBackend::open(&config)
            .map_err(|e| anyhow::anyhow!("Failed to open storage: {e}"))?;

        let mut storage = Self {
            backend,
            decisions_cache: Vec::new(),
            decision_id_to_idx: HashMap::new(),
        };
        storage.load_cache()?;
        Ok(storage)
    }

    fn load_cache(&mut self) -> Result<()> {
        self.decisions_cache.clear();
        self.decision_id_to_idx.clear();

        let ids = self
            .backend
            .list_ids()
            .map_err(|e| anyhow::anyhow!("Failed to list IDs: {e}"))?;

        for id in ids {
            if let Some(node) = self
                .backend
                .get(&id)
                .map_err(|e| anyhow::anyhow!("Failed to get node: {e}"))?
            {
                let decision = node.data.to_decision(node.embedding, &node.content_hash);
                let idx = self.decisions_cache.len();
                self.decision_id_to_idx.insert(decision.metadata.id, idx);
                self.decisions_cache.push(decision);
            }
        }

        Ok(())
    }

    fn rebuild_id_map(&mut self) {
        self.decision_id_to_idx.clear();
        for (idx, decision) in self.decisions_cache.iter().enumerate() {
            self.decision_id_to_idx.insert(decision.metadata.id, idx);
        }
    }
}

impl DecisionStorage for PersistentDecisionStorage {
    fn index(&mut self, decisions: Vec<Decision>) -> Result<()> {
        for decision in decisions {
            let id = decision.file_path.to_string_lossy().to_string();
            let node = StorageNode {
                id: id.clone(),
                data: StoredDecision::from(&decision),
                embedding: decision.embedding.clone(),
                content_hash: decision.content_hash.clone(),
            };

            self.backend
                .insert(node)
                .map_err(|e| anyhow::anyhow!("Failed to insert: {e}"))?;

            if let Some(&idx) = self.decision_id_to_idx.get(&decision.metadata.id) {
                self.decisions_cache[idx] = decision;
            } else {
                let idx = self.decisions_cache.len();
                self.decision_id_to_idx.insert(decision.metadata.id, idx);
                self.decisions_cache.push(decision);
            }
        }
        Ok(())
    }

    fn remove(&mut self, paths: Vec<String>) -> Result<()> {
        let path_refs: Vec<&str> = paths.iter().map(String::as_str).collect();
        self.backend
            .remove_batch(&path_refs)
            .map_err(|e| anyhow::anyhow!("Failed to remove: {e}"))?;

        for path in &paths {
            self.decisions_cache
                .retain(|d| d.file_path.to_string_lossy() != *path);
        }
        self.rebuild_id_map();
        Ok(())
    }

    fn search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Decision, f32)>> {
        let results = self
            .backend
            .search(&embedding, limit)
            .map_err(|e| anyhow::anyhow!("Search failed: {e}"))?;

        Ok(results
            .into_iter()
            .map(|(node, score)| {
                let decision = node.data.to_decision(node.embedding, &node.content_hash);
                (decision, score)
            })
            .collect())
    }

    fn get_hashes(&self) -> Result<HashMap<String, String>> {
        self.backend
            .get_all_hashes()
            .map_err(|e| anyhow::anyhow!("Failed to get hashes: {e}"))
    }

    fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>> {
        let mut chain = Vec::new();
        let mut current_id = Some(decision_id);
        let mut visited = std::collections::HashSet::new();

        while let Some(id) = current_id {
            if visited.contains(&id) {
                break;
            }
            visited.insert(id);

            if let Some(&idx) = self.decision_id_to_idx.get(&id) {
                let decision = &self.decisions_cache[idx];

                let superseded_ids: Vec<u32> = decision
                    .metadata
                    .supersedes
                    .as_ref()
                    .map(|s| s.to_vec())
                    .unwrap_or_default();

                chain.push(ChainNode {
                    id: decision.metadata.id,
                    title: decision.metadata.title.clone(),
                    status: decision.metadata.status.clone(),
                    date: decision.metadata.date,
                    is_current: false,
                });

                current_id = superseded_ids.first().copied();
            } else {
                break;
            }
        }

        if let Some(last) = chain.last_mut() {
            last.is_current = true;
        }

        Ok(chain)
    }

    fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>> {
        let mut related = Vec::new();

        let Some(&idx) = self.decision_id_to_idx.get(&decision_id) else {
            return Ok(related);
        };

        let decision = &self.decisions_cache[idx];

        let add_relations = |ids: &Option<crate::types::OneOrMany<u32>>,
                             rel_type: RelationType,
                             related: &mut Vec<RelatedDecision>,
                             decisions: &[Decision],
                             id_map: &HashMap<u32, usize>| {
            if let Some(ids) = ids {
                for target_id in ids.to_vec() {
                    if let Some(&target_idx) = id_map.get(&target_id) {
                        related.push(RelatedDecision {
                            id: target_id,
                            title: decisions[target_idx].metadata.title.clone(),
                            relation: rel_type,
                        });
                    }
                }
            }
        };

        add_relations(
            &decision.metadata.supersedes,
            RelationType::Supersedes,
            &mut related,
            &self.decisions_cache,
            &self.decision_id_to_idx,
        );
        add_relations(
            &decision.metadata.amends,
            RelationType::Amends,
            &mut related,
            &self.decisions_cache,
            &self.decision_id_to_idx,
        );
        add_relations(
            &decision.metadata.depends_on,
            RelationType::DependsOn,
            &mut related,
            &self.decisions_cache,
            &self.decision_id_to_idx,
        );
        add_relations(
            &decision.metadata.related_to,
            RelationType::RelatedTo,
            &mut related,
            &self.decisions_cache,
            &self.decision_id_to_idx,
        );

        for (other_idx, other_decision) in self.decisions_cache.iter().enumerate() {
            if other_idx == idx {
                continue;
            }

            if other_decision
                .metadata
                .supersedes
                .as_ref()
                .is_some_and(|s| s.to_vec().contains(&decision_id))
            {
                related.push(RelatedDecision {
                    id: other_decision.metadata.id,
                    title: other_decision.metadata.title.clone(),
                    relation: RelationType::Supersedes,
                });
            }
        }

        Ok(related)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Status;
    use chrono::NaiveDate;
    use helix_storage::StorageMode;
    use tempfile::TempDir;

    fn create_test_decision(id: u32, title: &str) -> Decision {
        Decision {
            metadata: DecisionMetadata {
                id,
                uuid: None,
                title: title.to_string(),
                status: Status::Proposed,
                date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                deciders: vec![],
                tags: vec![],
                content_hash: None,
                git_commit: None,
                supersedes: None,
                superseded_by: None,
                amends: None,
                depends_on: None,
                related_to: None,
            },
            body: format!("Body of {title}"),
            file_path: PathBuf::from(format!(".decisions/{id:03}-{title}.md")),
            content_hash: format!("hash-{id}"),
            embedding: Some(vec![id as f32 / 10.0, 0.5, 0.5]),
        }
    }

    #[test]
    fn test_index_and_search() {
        let temp = TempDir::new().unwrap();
        let config = StorageConfig {
            mode: StorageMode::ProjectLocal {
                tool_name: "decisions".to_string(),
            },
            base_path: temp.path().to_path_buf(),
        };

        let mut storage = PersistentDecisionStorage::open_with_config(config).unwrap();

        let decision = create_test_decision(1, "test-decision");
        storage.index(vec![decision]).unwrap();

        let results = storage.search(vec![0.1, 0.5, 0.5], 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.metadata.id, 1);
    }

    #[test]
    fn test_persistence() {
        let temp = TempDir::new().unwrap();
        let config = StorageConfig {
            mode: StorageMode::ProjectLocal {
                tool_name: "decisions".to_string(),
            },
            base_path: temp.path().to_path_buf(),
        };

        {
            let mut storage = PersistentDecisionStorage::open_with_config(config.clone()).unwrap();
            storage
                .index(vec![create_test_decision(1, "persisted")])
                .unwrap();
        }

        let storage = PersistentDecisionStorage::open_with_config(config).unwrap();
        let hashes = storage.get_hashes().unwrap();
        assert_eq!(hashes.len(), 1);
    }
}
