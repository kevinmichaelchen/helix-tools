use crate::helix_backend::HelixDecisionBackend;
use crate::manifest::ManifestEntry;
use crate::types::{ChainNode, Decision, RelatedDecision, RelationType};
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub trait DecisionStorage: Send + Sync {
    fn index(&mut self, decisions: Vec<Decision>) -> Result<()>;
    fn remove(&mut self, paths: Vec<String>) -> Result<()>;
    fn search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Decision, f32)>>;
    fn get_hashes(&self) -> Result<HashMap<String, String>>;
    fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>>;
    fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>>;
    fn repo_root(&self) -> &Path;
    fn manifest(&self) -> &crate::manifest::IndexManifest;
    fn handle_rename(&mut self, old_path: &str, new_path: &str) -> Result<()>;
}

pub struct HelixDecisionStorage {
    backend: HelixDecisionBackend,
}

impl HelixDecisionStorage {
    pub fn open(repo_root: &Path) -> Result<Self> {
        let backend = HelixDecisionBackend::new(repo_root)?;
        Ok(Self { backend })
    }

    pub fn backend(&self) -> &HelixDecisionBackend {
        &self.backend
    }

    pub fn backend_mut(&mut self) -> &mut HelixDecisionBackend {
        &mut self.backend
    }

    fn resolve_relationship_node_ids(&self, decision: &Decision) -> Vec<(RelationType, u128)> {
        let mut relationships = Vec::new();

        let resolve_ids = |ids: &Option<crate::types::OneOrMany<u32>>,
                           rel_type: RelationType,
                           backend: &HelixDecisionBackend|
         -> Vec<(RelationType, u128)> {
            let mut result = Vec::new();
            if let Some(id_list) = ids {
                for target_id in id_list.to_vec() {
                    if let Ok(Some(node_id)) = backend.find_node_id_by_decision_id(target_id) {
                        result.push((rel_type, node_id));
                    }
                }
            }
            result
        };

        relationships.extend(resolve_ids(
            &decision.metadata.supersedes,
            RelationType::Supersedes,
            &self.backend,
        ));
        relationships.extend(resolve_ids(
            &decision.metadata.amends,
            RelationType::Amends,
            &self.backend,
        ));
        relationships.extend(resolve_ids(
            &decision.metadata.depends_on,
            RelationType::DependsOn,
            &self.backend,
        ));
        relationships.extend(resolve_ids(
            &decision.metadata.related_to,
            RelationType::RelatedTo,
            &self.backend,
        ));

        relationships
    }
}

impl DecisionStorage for HelixDecisionStorage {
    fn index(&mut self, decisions: Vec<Decision>) -> Result<()> {
        let repo_root = self.backend.repo_root().to_path_buf();
        let embedding_model = self.backend.embedding_model().to_string();

        let mut manifest_entries: Vec<ManifestEntry> = Vec::new();
        let mut decision_node_ids: Vec<(u32, u128)> = Vec::new();

        {
            let mut wtxn = self.backend.begin_write_txn()?;

            for decision in &decisions {
                let embedding = decision
                    .embedding
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Decision missing embedding"))?;

                let normalized_path =
                    HelixDecisionBackend::normalize_path(&repo_root, &decision.file_path);

                let existing_entry = self
                    .backend
                    .manifest()
                    .get(&std::path::PathBuf::from(&normalized_path))
                    .cloned();

                let (node_id, vector_id) = self.backend.upsert_decision_node(
                    &mut wtxn,
                    decision,
                    embedding,
                    existing_entry.as_ref(),
                )?;

                decision_node_ids.push((decision.metadata.id, node_id));

                manifest_entries.push(ManifestEntry {
                    file_path: std::path::PathBuf::from(&normalized_path),
                    mtime: crate::manifest::get_mtime(&decision.file_path),
                    size: crate::manifest::get_size(&decision.file_path),
                    content_hash: decision.content_hash.clone(),
                    decision_id: decision.metadata.id,
                    uuid: decision.metadata.uuid.clone(),
                    vector_id: Some(vector_id.to_string()),
                    node_id: Some(node_id),
                    embedding_model: embedding_model.clone(),
                    indexer_version: crate::manifest::INDEXER_VERSION,
                });
            }

            self.backend.commit_txn(wtxn)?;
        }

        for entry in manifest_entries {
            self.backend.manifest_mut().upsert(entry);
        }

        let mut node_ids_and_relationships: Vec<(u128, Vec<(RelationType, u128)>)> = Vec::new();
        for decision in &decisions {
            let relationships = self.resolve_relationship_node_ids(decision);
            if let Some(&(_, node_id)) = decision_node_ids
                .iter()
                .find(|(id, _)| *id == decision.metadata.id)
            {
                node_ids_and_relationships.push((node_id, relationships));
            }
        }

        {
            let mut wtxn = self.backend.begin_write_txn()?;
            for (node_id, relationships) in node_ids_and_relationships {
                self.backend.remove_outgoing_edges(&mut wtxn, node_id)?;
                if !relationships.is_empty() {
                    self.backend
                        .create_relationship_edges(&mut wtxn, node_id, &relationships)?;
                }
            }
            self.backend.commit_txn(wtxn)?;
        }

        self.backend.commit_manifest()?;

        Ok(())
    }

    fn remove(&mut self, paths: Vec<String>) -> Result<()> {
        let entries_to_remove: Vec<(std::path::PathBuf, ManifestEntry)> = paths
            .iter()
            .filter_map(|path| {
                let path_buf = std::path::PathBuf::from(path);
                self.backend
                    .manifest()
                    .get(&path_buf)
                    .cloned()
                    .map(|entry| (path_buf, entry))
            })
            .collect();

        {
            let mut wtxn = self.backend.begin_write_txn()?;
            for (_, entry) in &entries_to_remove {
                self.backend.delete_decision_node(&mut wtxn, entry)?;
            }
            self.backend.commit_txn(wtxn)?;
        }

        for (path_buf, _) in entries_to_remove {
            self.backend.manifest_mut().remove(&path_buf);
        }

        self.backend.commit_manifest()?;

        Ok(())
    }

    fn search(&self, embedding: Vec<f32>, limit: usize) -> Result<Vec<(Decision, f32)>> {
        self.backend.search(&embedding, limit)
    }

    fn get_hashes(&self) -> Result<HashMap<String, String>> {
        self.backend.get_hashes()
    }

    fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>> {
        self.backend.get_chain(decision_id)
    }

    fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>> {
        self.backend.get_related(decision_id)
    }

    fn repo_root(&self) -> &Path {
        self.backend.repo_root()
    }

    fn manifest(&self) -> &crate::manifest::IndexManifest {
        self.backend.manifest()
    }

    fn handle_rename(&mut self, old_path: &str, new_path: &str) -> Result<()> {
        if let Some(mut entry) = self
            .backend
            .manifest_mut()
            .remove(&std::path::PathBuf::from(old_path))
        {
            entry.file_path = std::path::PathBuf::from(new_path);
            self.backend.manifest_mut().upsert(entry);
            self.backend.commit_manifest()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{DecisionMetadata, Status};
    use chrono::NaiveDate;
    use std::path::PathBuf;
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
        let mut storage = HelixDecisionStorage::open(temp.path()).unwrap();

        let decision = create_test_decision(1, "test-decision");
        storage.index(vec![decision]).unwrap();

        let results = storage.search(vec![0.1, 0.5, 0.5], 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.metadata.id, 1);
    }

    #[test]
    fn test_persistence() {
        let temp = TempDir::new().unwrap();

        {
            let mut storage = HelixDecisionStorage::open(temp.path()).unwrap();
            storage
                .index(vec![create_test_decision(1, "persisted")])
                .unwrap();
        }

        let storage = HelixDecisionStorage::open(temp.path()).unwrap();
        let hashes = storage.get_hashes().unwrap();
        assert_eq!(hashes.len(), 1);
    }
}
