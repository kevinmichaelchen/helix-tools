use crate::manifest::{IndexManifest, MANIFEST_KEY, ManifestEntry};
use crate::types::{ChainNode, Decision, DecisionMetadata, RelatedDecision, RelationType, Status};
use anyhow::{Context, Result};
use bumpalo::Bump;
use heed3;
use helix_db::{
    helix_engine::{
        storage_core::{HelixGraphStorage, storage_methods::StorageMethods},
        traversal_core::config::{Config, GraphConfig, VectorConfig},
        types::SecondaryIndex,
        vector_core::hnsw::HNSW,
    },
    protocol::value::Value,
    utils::{items::Edge, label_hash::hash_label, properties::ImmutablePropertiesMap},
};
use helix_graph_ops as graph_ops;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use uuid::Uuid;

const DEFAULT_BATCH_SIZE: usize = 100;
const NODE_LABEL: &str = "DECISION";
const EMBEDDING_MODEL: &str = "BAAI/bge-small-en-v1.5";

#[derive(Debug, Default)]
pub struct SyncStats {
    pub scanned: u32,
    pub added: u32,
    pub modified: u32,
    pub deleted: u32,
    pub renamed: u32,
    pub unchanged: u32,
    pub errors: u32,
    pub duration_ms: u64,
}

pub struct HelixDecisionBackend {
    storage: HelixGraphStorage,
    manifest: IndexManifest,
    embedding_model: String,
    batch_size: usize,
    repo_root: PathBuf,
}

impl HelixDecisionBackend {
    pub fn new(repo_root: &Path) -> Result<Self> {
        let db_path = std::env::var("HELIX_DB_PATH").unwrap_or_else(|_| {
            repo_root
                .join(".helix")
                .join("data")
                .join("decisions")
                .to_string_lossy()
                .to_string()
        });

        std::fs::create_dir_all(&db_path)
            .with_context(|| format!("Failed to create database directory: {db_path}"))?;

        let config = Config {
            vector_config: Some(VectorConfig {
                m: Some(16),
                ef_construction: Some(128),
                ef_search: Some(64),
            }),
            graph_config: Some(GraphConfig {
                secondary_indices: Some(vec![
                    SecondaryIndex::Index("id".to_string()),
                    SecondaryIndex::Index("vector_id".to_string()),
                ]),
            }),
            db_max_size_gb: Some(1),
            ..Default::default()
        };

        let version_info =
            helix_db::helix_engine::storage_core::version_info::VersionInfo::default();
        let storage = HelixGraphStorage::new(&db_path, config, version_info)
            .map_err(|e| anyhow::anyhow!("Failed to create storage: {e:?}"))?;

        let manifest = Self::load_manifest(&storage)?;

        let batch_size = std::env::var("HELIX_DECISIONS_BATCH_SIZE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_BATCH_SIZE);

        Ok(Self {
            storage,
            manifest,
            embedding_model: EMBEDDING_MODEL.to_string(),
            batch_size,
            repo_root: repo_root.to_path_buf(),
        })
    }

    fn load_manifest(storage: &HelixGraphStorage) -> Result<IndexManifest> {
        let rtxn = storage
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read transaction: {e}"))?;

        let data = storage
            .metadata_db
            .get(&rtxn, MANIFEST_KEY.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to read manifest: {e}"))?;

        Ok(data.map(IndexManifest::from_bytes).unwrap_or_default())
    }

    fn save_manifest(&self) -> Result<()> {
        let mut wtxn = self
            .storage
            .graph_env
            .write_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start write transaction: {e}"))?;

        let bytes = self.manifest.to_bytes();
        self.storage
            .metadata_db
            .put(&mut wtxn, MANIFEST_KEY.as_bytes(), &bytes)
            .map_err(|e| anyhow::anyhow!("Failed to write manifest: {e}"))?;

        wtxn.commit()
            .map_err(|e| anyhow::anyhow!("Failed to commit manifest: {e}"))?;

        Ok(())
    }

    pub fn normalize_path(repo_root: &Path, file_path: &Path) -> String {
        let relative = file_path.strip_prefix(repo_root).unwrap_or(file_path);

        let mut components: Vec<std::path::Component> = Vec::new();
        for component in relative.components() {
            match component {
                std::path::Component::CurDir => {}
                std::path::Component::ParentDir => {
                    components.pop();
                }
                c => components.push(c),
            }
        }

        let normalized: PathBuf = components.into_iter().collect();
        normalized.to_string_lossy().replace('\\', "/")
    }

    #[allow(dead_code)]
    fn get_identity(metadata: &DecisionMetadata) -> (Option<String>, u32) {
        (metadata.uuid.clone(), metadata.id)
    }

    pub fn begin_write_txn(&self) -> Result<heed3::RwTxn<'_>> {
        self.storage
            .graph_env
            .write_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start write transaction: {e}"))
    }

    pub fn commit_txn(&self, wtxn: heed3::RwTxn<'_>) -> Result<()> {
        wtxn.commit()
            .map_err(|e| anyhow::anyhow!("Failed to commit transaction: {e}"))
    }

    pub fn upsert_decision_node<'txn>(
        &'txn self,
        wtxn: &mut heed3::RwTxn<'txn>,
        decision: &Decision,
        embedding: &[f32],
        existing_entry: Option<&ManifestEntry>,
    ) -> Result<(u128, u128)> {
        let arena = Bump::new();

        let node_id = if let Some(entry) = existing_entry {
            if let Some(ref old_vector_id) = entry.vector_id
                && let Ok(old_vid) = old_vector_id.parse::<u128>()
            {
                let _ = self.storage.vectors.delete(wtxn, old_vid, &arena);
            }

            if let Some(old_node_id) = entry.node_id {
                self.delete_secondary_index_entries_internal(wtxn, old_node_id, &arena)?;
                self.storage
                    .nodes_db
                    .delete(wtxn, HelixGraphStorage::node_key(&old_node_id))
                    .map_err(|e| anyhow::anyhow!("Failed to delete old node: {e}"))?;
                old_node_id
            } else {
                Uuid::new_v4().as_u128()
            }
        } else {
            Uuid::new_v4().as_u128()
        };

        let embedding_f64: Vec<f64> = embedding.iter().map(|&x| f64::from(x)).collect();
        let label: &str = arena.alloc_str(NODE_LABEL);

        let vector =
            self.storage
                .vectors
                .insert::<fn(
                    &helix_db::helix_engine::vector_core::vector::HVector<'_>,
                    &heed3::RoTxn<'_>,
                ) -> bool>(wtxn, label, &embedding_f64, None, &arena)
                .map_err(|e| anyhow::anyhow!("Failed to insert vector: {e:?}"))?;

        let vector_id = vector.id;

        let normalized_path = Self::normalize_path(&self.repo_root, &decision.file_path);

        let tags_json = serde_json::to_string(&decision.metadata.tags).unwrap_or_default();
        let deciders_json = serde_json::to_string(&decision.metadata.deciders).unwrap_or_default();

        let mut props: Vec<(&str, Value)> = Vec::with_capacity(11);
        props.push((
            arena.alloc_str("id"),
            Value::I64(i64::from(decision.metadata.id)),
        ));
        props.push((
            arena.alloc_str("title"),
            Value::String(decision.metadata.title.clone()),
        ));
        props.push((
            arena.alloc_str("status"),
            Value::String(decision.metadata.status.to_string()),
        ));
        props.push((
            arena.alloc_str("date"),
            Value::String(decision.metadata.date.to_string()),
        ));
        props.push((arena.alloc_str("file_path"), Value::String(normalized_path)));
        props.push((
            arena.alloc_str("content_hash"),
            Value::String(decision.content_hash.clone()),
        ));
        props.push((
            arena.alloc_str("vector_id"),
            Value::String(vector_id.to_string()),
        ));
        props.push((arena.alloc_str("tags"), Value::String(tags_json)));
        props.push((arena.alloc_str("deciders"), Value::String(deciders_json)));
        props.push((
            arena.alloc_str("body"),
            Value::String(decision.body.clone()),
        ));

        if let Some(ref uuid) = decision.metadata.uuid {
            props.push((arena.alloc_str("uuid"), Value::String(uuid.clone())));
        }

        let properties = ImmutablePropertiesMap::new(props.len(), props.into_iter(), &arena);

        let node = helix_db::utils::items::Node {
            id: node_id,
            label,
            version: 1,
            properties: Some(properties),
        };

        graph_ops::put_node(&self.storage, wtxn, &node)
            .map_err(|e| anyhow::anyhow!("Failed to store node: {e}"))?;

        graph_ops::update_secondary_indices(&self.storage, wtxn, &node)
            .map_err(|e| anyhow::anyhow!("Failed to update secondary index: {e}"))?;

        Ok((node_id, vector_id))
    }

    pub fn upsert_decision_node_standalone(
        &self,
        decision: &Decision,
        embedding: &[f32],
        existing_entry: Option<&ManifestEntry>,
    ) -> Result<(u128, u128)> {
        let mut wtxn = self.begin_write_txn()?;
        let result = self.upsert_decision_node(&mut wtxn, decision, embedding, existing_entry)?;
        self.commit_txn(wtxn)?;
        Ok(result)
    }

    fn delete_secondary_index_entries_internal(
        &self,
        wtxn: &mut heed3::RwTxn<'_>,
        node_id: u128,
        arena: &Bump,
    ) -> Result<()> {
        let rtxn = self
            .storage
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read txn: {e}"))?;

        if let Ok(node) = self.storage.get_node(&rtxn, &node_id, arena) {
            for (index_name, db) in &self.storage.secondary_indices {
                if let Some(value) = node.get_property(index_name)
                    && let Ok(serialized) = bincode::serialize(value)
                {
                    let _ = db.0.delete(wtxn, &serialized);
                }
            }
        }
        Ok(())
    }

    pub fn create_relationship_edges(
        &self,
        wtxn: &mut heed3::RwTxn<'_>,
        from_node_id: u128,
        relationships: &[(RelationType, u128)],
    ) -> Result<()> {
        if relationships.is_empty() {
            return Ok(());
        }

        let arena = Bump::new();

        for (relation_type, to_node_id) in relationships {
            let edge_id = Uuid::new_v4().as_u128();
            let edge_label = arena.alloc_str(relation_type.as_edge_label());

            let edge = Edge {
                id: edge_id,
                label: edge_label,
                version: 1,
                from_node: from_node_id,
                to_node: *to_node_id,
                properties: None,
            };

            graph_ops::put_edge(&self.storage, wtxn, &edge)
                .map_err(|e| anyhow::anyhow!("Failed to store edge: {e}"))?;
        }

        Ok(())
    }

    pub fn create_relationship_edges_standalone(
        &self,
        from_node_id: u128,
        relationships: &[(RelationType, u128)],
    ) -> Result<()> {
        let mut wtxn = self.begin_write_txn()?;
        self.create_relationship_edges(&mut wtxn, from_node_id, relationships)?;
        self.commit_txn(wtxn)
    }

    pub fn remove_outgoing_edges(
        &self,
        wtxn: &mut heed3::RwTxn<'_>,
        node_id: u128,
    ) -> Result<Vec<(u128, [u8; 4])>> {
        let mut edges_to_remove = Vec::new();
        let mut in_edges_to_remove = Vec::new();

        {
            let iter = self
                .storage
                .out_edges_db
                .prefix_iter(wtxn, &node_id.to_be_bytes())
                .map_err(|e| anyhow::anyhow!("Failed to iterate out edges: {e}"))?;

            for result in iter {
                let (key, value) =
                    result.map_err(|e| anyhow::anyhow!("Failed to read edge: {e}"))?;

                if key.len() != 20 {
                    continue;
                }

                let mut label_hash = [0u8; 4];
                label_hash.copy_from_slice(&key[16..20]);

                let (edge_id, to_node_id) = HelixGraphStorage::unpack_adj_edge_data(value)
                    .map_err(|e| anyhow::anyhow!("Failed to unpack edge data: {e:?}"))?;

                edges_to_remove.push((edge_id, label_hash));
                in_edges_to_remove.push((to_node_id, label_hash, edge_id));
            }
        }

        for (edge_id, label_hash) in &edges_to_remove {
            self.storage
                .edges_db
                .delete(wtxn, HelixGraphStorage::edge_key(edge_id))
                .map_err(|e| anyhow::anyhow!("Failed to delete edge: {e}"))?;

            let out_key = HelixGraphStorage::out_edge_key(&node_id, label_hash);
            self.storage
                .out_edges_db
                .delete(wtxn, &out_key)
                .map_err(|e| anyhow::anyhow!("Failed to delete out edge: {e}"))?;
        }

        for (to_node_id, label_hash, edge_id) in &in_edges_to_remove {
            let in_key = HelixGraphStorage::in_edge_key(to_node_id, label_hash);
            let in_val = HelixGraphStorage::pack_edge_data(edge_id, &node_id);
            let _ = self
                .storage
                .in_edges_db
                .delete_one_duplicate(wtxn, &in_key, &in_val);
        }

        Ok(edges_to_remove)
    }

    pub fn remove_outgoing_edges_standalone(&self, node_id: u128) -> Result<Vec<(u128, [u8; 4])>> {
        let mut wtxn = self.begin_write_txn()?;
        let result = self.remove_outgoing_edges(&mut wtxn, node_id)?;
        self.commit_txn(wtxn)?;
        Ok(result)
    }

    pub fn delete_decision_node(
        &self,
        wtxn: &mut heed3::RwTxn<'_>,
        entry: &ManifestEntry,
    ) -> Result<()> {
        let arena = Bump::new();

        if let Some(node_id) = entry.node_id {
            self.storage
                .drop_node(wtxn, &node_id)
                .map_err(|e| anyhow::anyhow!("Failed to delete node: {e:?}"))?;
        }

        if let Some(ref vector_id_str) = entry.vector_id
            && let Ok(vector_id) = vector_id_str.parse::<u128>()
        {
            let _ = self.storage.vectors.delete(wtxn, vector_id, &arena);
        }

        Ok(())
    }

    pub fn delete_decision_node_standalone(&mut self, entry: &ManifestEntry) -> Result<()> {
        let mut wtxn = self.begin_write_txn()?;
        self.delete_decision_node(&mut wtxn, entry)?;
        self.commit_txn(wtxn)
    }

    pub fn search(&self, embedding: &[f32], limit: usize) -> Result<Vec<(Decision, f32)>> {
        let arena = Bump::new();
        let rtxn = self
            .storage
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read transaction: {e}"))?;

        let query_f64: Vec<f64> = embedding.iter().map(|&x| f64::from(x)).collect();
        let label: &str = arena.alloc_str(NODE_LABEL);

        let vector_results =
            self.storage
                .vectors
                .search::<fn(
                    &helix_db::helix_engine::vector_core::vector::HVector<'_>,
                    &heed3::RoTxn<'_>,
                ) -> bool>(&rtxn, &query_f64, limit, label, None, false, &arena)
                .map_err(|e| anyhow::anyhow!("Vector search failed: {e:?}"))?;

        let mut results = Vec::new();

        for hvector in vector_results {
            let vector_id = hvector.id;
            let distance = hvector.get_distance() as f32;
            let score = 1.0 / (1.0 + distance);

            if let Some(node_id) = self.lookup_node_by_vector_id(&rtxn, vector_id, &arena)?
                && let Ok(decision) = self.node_to_decision(&rtxn, node_id, &arena)
            {
                results.push((decision, score));
            }
        }

        Ok(results)
    }

    fn lookup_node_by_vector_id(
        &self,
        rtxn: &heed3::RoTxn<'_>,
        vector_id: u128,
        _arena: &Bump,
    ) -> Result<Option<u128>> {
        let key = Value::String(vector_id.to_string());
        graph_ops::lookup_secondary_index(&self.storage, rtxn, "vector_id", &key)
            .map_err(|e| anyhow::anyhow!("Failed to lookup vector_id: {e}"))
    }

    fn node_to_decision(
        &self,
        rtxn: &heed3::RoTxn<'_>,
        node_id: u128,
        arena: &Bump,
    ) -> Result<Decision> {
        let node = self
            .storage
            .get_node(rtxn, &node_id, arena)
            .map_err(|e| anyhow::anyhow!("Failed to get node: {e:?}"))?;

        let get_str = |name: &str| -> String {
            node.get_property(name)
                .and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        };

        let get_i64 = |name: &str| -> i64 {
            node.get_property(name)
                .and_then(|v| match v {
                    Value::I64(i) => Some(*i),
                    _ => None,
                })
                .unwrap_or_default()
        };

        let id = u32::try_from(get_i64("id")).unwrap_or_default();
        let title = get_str("title");
        let status_str = get_str("status");
        let date_str = get_str("date");
        let file_path_str = get_str("file_path");
        let content_hash = get_str("content_hash");
        let body = get_str("body");
        let tags_json = get_str("tags");
        let deciders_json = get_str("deciders");
        let uuid = node.get_property("uuid").and_then(|v| match v {
            Value::String(s) => Some(s.clone()),
            _ => None,
        });

        let status = status_str.parse().unwrap_or(Status::Proposed);
        let date = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
            .unwrap_or_else(|_| chrono::Utc::now().date_naive());
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        let deciders: Vec<String> = serde_json::from_str(&deciders_json).unwrap_or_default();

        Ok(Decision {
            metadata: DecisionMetadata {
                id,
                uuid,
                title,
                status,
                date,
                deciders,
                tags,
                content_hash: None,
                git_commit: None,
                supersedes: None,
                superseded_by: None,
                amends: None,
                depends_on: None,
                related_to: None,
            },
            body,
            file_path: PathBuf::from(file_path_str),
            content_hash,
            embedding: None,
        })
    }

    pub fn get_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>> {
        let arena = Bump::new();
        let rtxn = self
            .storage
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read transaction: {e}"))?;

        let mut chain = Vec::new();
        let mut current_id = Some(decision_id);
        let mut visited = HashSet::new();

        while let Some(id) = current_id {
            if visited.contains(&id) {
                break;
            }
            visited.insert(id);

            if let Some(node_id) = self.lookup_node_by_decision_id(&rtxn, id, &arena)? {
                if let Ok(decision) = self.node_to_decision(&rtxn, node_id, &arena) {
                    chain.push(ChainNode {
                        id: decision.metadata.id,
                        title: decision.metadata.title,
                        status: decision.metadata.status,
                        date: decision.metadata.date,
                        is_current: false,
                    });

                    let superseding_sources = self.get_incoming_edge_sources(
                        &rtxn,
                        node_id,
                        RelationType::Supersedes,
                        &arena,
                    )?;

                    current_id = if superseding_sources.is_empty() {
                        None
                    } else if superseding_sources.len() == 1 {
                        self.get_decision_id_from_node(&rtxn, superseding_sources[0], &arena)
                            .ok()
                    } else {
                        self.pick_latest_superseding_node(&rtxn, &superseding_sources, &arena)
                    };
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        if let Some(last) = chain.last_mut() {
            last.is_current = true;
        }

        Ok(chain)
    }

    fn pick_latest_superseding_node(
        &self,
        rtxn: &heed3::RoTxn<'_>,
        node_ids: &[u128],
        arena: &Bump,
    ) -> Option<u32> {
        let mut best: Option<(u32, chrono::NaiveDate)> = None;

        for &node_id in node_ids {
            if let Ok(decision) = self.node_to_decision(rtxn, node_id, arena) {
                match best {
                    None => best = Some((decision.metadata.id, decision.metadata.date)),
                    Some((_, best_date)) if decision.metadata.date > best_date => {
                        best = Some((decision.metadata.id, decision.metadata.date));
                    }
                    Some((best_id, best_date)) if decision.metadata.date == best_date => {
                        if decision.metadata.id > best_id {
                            best = Some((decision.metadata.id, decision.metadata.date));
                        }
                    }
                    _ => {}
                }
            }
        }

        best.map(|(id, _)| id)
    }

    pub fn get_related(&self, decision_id: u32) -> Result<Vec<RelatedDecision>> {
        let arena = Bump::new();
        let rtxn = self
            .storage
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read transaction: {e}"))?;

        let mut related = Vec::new();

        let Some(node_id) = self.lookup_node_by_decision_id(&rtxn, decision_id, &arena)? else {
            return Ok(related);
        };

        for relation_type in [
            RelationType::Supersedes,
            RelationType::Amends,
            RelationType::DependsOn,
            RelationType::RelatedTo,
        ] {
            let targets = self.get_outgoing_edge_targets(&rtxn, node_id, relation_type, &arena)?;
            for target_node_id in targets {
                if let Ok(decision) = self.node_to_decision(&rtxn, target_node_id, &arena) {
                    related.push(RelatedDecision {
                        id: decision.metadata.id,
                        title: decision.metadata.title,
                        relation: relation_type,
                    });
                }
            }
        }

        for relation_type in [
            RelationType::Supersedes,
            RelationType::Amends,
            RelationType::DependsOn,
            RelationType::RelatedTo,
        ] {
            let sources = self.get_incoming_edge_sources(&rtxn, node_id, relation_type, &arena)?;
            for source_node_id in sources {
                if let Ok(decision) = self.node_to_decision(&rtxn, source_node_id, &arena) {
                    related.push(RelatedDecision {
                        id: decision.metadata.id,
                        title: decision.metadata.title,
                        relation: relation_type,
                    });
                }
            }
        }

        Ok(related)
    }

    fn lookup_node_by_decision_id(
        &self,
        rtxn: &heed3::RoTxn<'_>,
        decision_id: u32,
        _arena: &Bump,
    ) -> Result<Option<u128>> {
        let key = Value::I64(i64::from(decision_id));
        graph_ops::lookup_secondary_index(&self.storage, rtxn, "id", &key)
            .map_err(|e| anyhow::anyhow!("Failed to lookup decision_id: {e}"))
    }

    fn get_decision_id_from_node(
        &self,
        rtxn: &heed3::RoTxn<'_>,
        node_id: u128,
        arena: &Bump,
    ) -> Result<u32> {
        let node = self
            .storage
            .get_node(rtxn, &node_id, arena)
            .map_err(|e| anyhow::anyhow!("Failed to get node: {e:?}"))?;

        let id = node
            .get_property("id")
            .and_then(|v| match v {
                Value::I64(i) => Some(*i),
                _ => None,
            })
            .unwrap_or_default();

        Ok(u32::try_from(id).unwrap_or_default())
    }

    fn get_outgoing_edge_targets(
        &self,
        rtxn: &heed3::RoTxn<'_>,
        node_id: u128,
        relation_type: RelationType,
        _arena: &Bump,
    ) -> Result<Vec<u128>> {
        let label_hash = hash_label(relation_type.as_edge_label(), None);
        graph_ops::outgoing_neighbors(&self.storage, rtxn, node_id, &label_hash)
            .map_err(|e| anyhow::anyhow!("Failed to read outgoing edges: {e}"))
    }

    fn get_incoming_edge_sources(
        &self,
        rtxn: &heed3::RoTxn<'_>,
        node_id: u128,
        relation_type: RelationType,
        _arena: &Bump,
    ) -> Result<Vec<u128>> {
        let label_hash = hash_label(relation_type.as_edge_label(), None);
        graph_ops::incoming_neighbors(&self.storage, rtxn, node_id, &label_hash)
            .map_err(|e| anyhow::anyhow!("Failed to read incoming edges: {e}"))
    }

    pub fn get_hashes(&self) -> Result<HashMap<String, String>> {
        let mut hashes = HashMap::new();
        for entry in self.manifest.entries() {
            hashes.insert(
                entry.file_path.to_string_lossy().to_string(),
                entry.content_hash.clone(),
            );
        }
        Ok(hashes)
    }

    pub fn manifest(&self) -> &IndexManifest {
        &self.manifest
    }

    pub fn manifest_mut(&mut self) -> &mut IndexManifest {
        &mut self.manifest
    }

    pub fn embedding_model(&self) -> &str {
        &self.embedding_model
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    pub fn commit_manifest(&self) -> Result<()> {
        self.save_manifest()
    }

    pub fn find_node_id_by_decision_id(&self, decision_id: u32) -> Result<Option<u128>> {
        let arena = Bump::new();
        let rtxn = self
            .storage
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read transaction: {e}"))?;

        self.lookup_node_by_decision_id(&rtxn, decision_id, &arena)
    }

    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use tempfile::TempDir;

    #[allow(dead_code)]
    fn create_test_decision(id: u32, title: &str) -> Decision {
        Decision {
            metadata: DecisionMetadata {
                id,
                uuid: Some(format!("hx-{id:06x}")),
                title: title.to_string(),
                status: Status::Proposed,
                date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
                deciders: vec!["Alice".to_string()],
                tags: vec!["test".to_string()],
                content_hash: None,
                git_commit: None,
                supersedes: None,
                superseded_by: None,
                amends: None,
                depends_on: None,
                related_to: None,
            },
            body: format!("Body of {title}"),
            file_path: PathBuf::from(format!(
                ".decisions/{id:03}-{}.md",
                title.to_lowercase().replace(' ', "-")
            )),
            content_hash: format!("hash-{id}"),
            embedding: None,
        }
    }

    #[test]
    fn test_normalize_path() {
        let repo_root = Path::new("/repo");

        assert_eq!(
            HelixDecisionBackend::normalize_path(
                repo_root,
                Path::new("/repo/.decisions/001-test.md")
            ),
            ".decisions/001-test.md"
        );

        assert_eq!(
            HelixDecisionBackend::normalize_path(
                repo_root,
                Path::new("/repo/./foo/../.decisions/001-test.md")
            ),
            ".decisions/001-test.md"
        );
    }

    #[test]
    fn test_get_identity() {
        let mut metadata = DecisionMetadata {
            id: 1,
            uuid: None,
            title: "Test".to_string(),
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
        };

        let (uuid, id) = HelixDecisionBackend::get_identity(&metadata);
        assert!(uuid.is_none());
        assert_eq!(id, 1);

        metadata.uuid = Some("hx-abc123".to_string());
        let (uuid, id) = HelixDecisionBackend::get_identity(&metadata);
        assert_eq!(uuid, Some("hx-abc123".to_string()));
        assert_eq!(id, 1);
    }

    #[test]
    fn test_backend_creation() {
        let temp = TempDir::new().unwrap();
        let backend = HelixDecisionBackend::new(temp.path());
        assert!(backend.is_ok());
    }

    #[test]
    fn test_manifest_persistence() {
        let temp = TempDir::new().unwrap();

        {
            let mut backend = HelixDecisionBackend::new(temp.path()).unwrap();

            let entry = ManifestEntry::new(
                PathBuf::from(".decisions/001-test.md"),
                1704067200,
                1024,
                "hash123".to_string(),
                1,
                None,
                EMBEDDING_MODEL,
            );
            backend.manifest_mut().upsert(entry);
            backend.commit_manifest().unwrap();
        }

        {
            let backend = HelixDecisionBackend::new(temp.path()).unwrap();
            assert!(
                backend
                    .manifest()
                    .contains(&PathBuf::from(".decisions/001-test.md"))
            );
        }
    }

    #[test]
    fn test_sync_stats_default() {
        let stats = SyncStats::default();
        assert_eq!(stats.scanned, 0);
        assert_eq!(stats.added, 0);
        assert_eq!(stats.modified, 0);
        assert_eq!(stats.deleted, 0);
        assert_eq!(stats.renamed, 0);
        assert_eq!(stats.unchanged, 0);
        assert_eq!(stats.errors, 0);
        assert_eq!(stats.duration_ms, 0);
    }
}
