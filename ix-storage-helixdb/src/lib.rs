use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bumpalo::Bump;
use heed3::{RoTxn, RwTxn};
use helix_db::helix_engine::storage_core::storage_methods::StorageMethods;
use helix_db::helix_engine::storage_core::{HelixGraphStorage, version_info::VersionInfo};
use helix_db::helix_engine::traversal_core::config::{Config, GraphConfig, VectorConfig};
use helix_db::helix_engine::types::SecondaryIndex;
use helix_db::helix_engine::vector_core::hnsw::HNSW;
use helix_db::protocol::value::Value;
use helix_db::utils::items::{Edge, Node};
use helix_db::utils::label_hash::hash_label;
use helix_db::utils::properties::ImmutablePropertiesMap;
use ix_core::entity::{EntityKind, kind_from_id};
use ix_core::index::{IndexBackend, SearchHit, SyncStats};
use ix_core::markdown::{get_string, get_string_list, parse_markdown};
use ix_core::repo::IxchelRepo;
use ix_embeddings::Embedder;
use ix_helixdb_ops as graph_ops;
use serde_yaml::Value as YamlValue;
use uuid::Uuid;

const NODE_LABEL: &str = "IXCHEL_ENTITY";
const METADATA_KEYS: &[&str] = &[
    "id",
    "type",
    "title",
    "status",
    "date",
    "created_at",
    "updated_at",
    "created_by",
    "tags",
];

pub struct HelixDbIndex {
    repo_root: PathBuf,
    db_path: PathBuf,
    storage: Option<HelixGraphStorage>,
    embedder: Embedder,
}

impl HelixDbIndex {
    pub fn open(repo: &IxchelRepo) -> Result<Self> {
        let embedder = Embedder::with_config(&repo.config.embedding)
            .map_err(|e| anyhow::anyhow!("Failed to initialize embedder: {e}"))?;
        Self::open_with_embedder(repo, embedder)
    }

    pub fn open_with_embedder(repo: &IxchelRepo, embedder: Embedder) -> Result<Self> {
        let repo_root = repo.paths.repo_root().to_path_buf();
        let db_path = repo
            .paths
            .ixchel_dir()
            .join(PathBuf::from(&repo.config.storage.path));

        std::fs::create_dir_all(&db_path)
            .with_context(|| format!("Failed to create {}", db_path.display()))?;

        let storage = Some(open_storage(&db_path)?);

        Ok(Self {
            repo_root,
            db_path,
            storage,
            embedder,
        })
    }

    pub fn outgoing(&self, from_id: &str, rel: &str) -> Result<Vec<String>> {
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;

        let rtxn = storage
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read transaction: {e}"))?;

        let Some(from_node) = lookup_node_by_entity_id(storage, &rtxn, from_id)? else {
            return Ok(Vec::new());
        };

        let label = rel.trim().to_ascii_uppercase();
        if label.is_empty() {
            return Ok(Vec::new());
        }

        let label_hash = hash_label(&label, None);
        let neighbors = graph_ops::outgoing_neighbors(storage, &rtxn, from_node, &label_hash)
            .map_err(|e| anyhow::anyhow!("Failed to read outgoing neighbors: {e}"))?;

        let arena = Bump::new();
        let mut out = neighbors
            .into_iter()
            .filter_map(|node_id| node_entity_id(storage, &rtxn, node_id, &arena).transpose())
            .collect::<Result<Vec<_>>>()?;

        out.sort();
        out.dedup();
        Ok(out)
    }

    pub fn incoming(&self, to_id: &str, rel: &str) -> Result<Vec<String>> {
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;

        let rtxn = storage
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read transaction: {e}"))?;

        let Some(to_node) = lookup_node_by_entity_id(storage, &rtxn, to_id)? else {
            return Ok(Vec::new());
        };

        let label = rel.trim().to_ascii_uppercase();
        if label.is_empty() {
            return Ok(Vec::new());
        }

        let label_hash = hash_label(&label, None);
        let neighbors = graph_ops::incoming_neighbors(storage, &rtxn, to_node, &label_hash)
            .map_err(|e| anyhow::anyhow!("Failed to read incoming neighbors: {e}"))?;

        let arena = Bump::new();
        let mut out = neighbors
            .into_iter()
            .filter_map(|node_id| node_entity_id(storage, &rtxn, node_id, &arena).transpose())
            .collect::<Result<Vec<_>>>()?;

        out.sort();
        out.dedup();
        Ok(out)
    }

    fn rebuild_storage(&mut self) -> Result<()> {
        self.storage.take();
        if self.db_path.exists() {
            std::fs::remove_dir_all(&self.db_path)
                .with_context(|| format!("Failed to remove {}", self.db_path.display()))?;
        }
        std::fs::create_dir_all(&self.db_path)
            .with_context(|| format!("Failed to create {}", self.db_path.display()))?;

        self.storage = Some(open_storage(&self.db_path)?);
        Ok(())
    }

    fn begin_write_txn(&self) -> Result<RwTxn<'_>> {
        self.storage
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?
            .graph_env
            .write_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start write transaction: {e}"))
    }

    fn commit_txn(wtxn: RwTxn<'_>) -> Result<()> {
        wtxn.commit()
            .map_err(|e| anyhow::anyhow!("Failed to commit transaction: {e}"))
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embedder
            .embed(text)
            .map_err(|e| anyhow::anyhow!("Embedding failed: {e}"))
    }

    fn insert_edges(
        &self,
        wtxn: &mut RwTxn<'_>,
        id_to_node: &BTreeMap<String, u128>,
        records: Vec<EntityRecord>,
    ) -> Result<()> {
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;

        for record in records {
            for (rel, to_ids) in record.rels {
                let label = rel.to_ascii_uppercase();
                for to_id in to_ids {
                    let Some(to_node) = id_to_node.get(&to_id).copied() else {
                        continue;
                    };

                    let arena = Bump::new();
                    let edge_label = arena.alloc_str(&label);
                    let edge = Edge {
                        id: Uuid::new_v4().as_u128(),
                        label: edge_label,
                        version: 1,
                        from_node: record.from_node,
                        to_node,
                        properties: None,
                    };

                    graph_ops::put_edge(storage, wtxn, &edge)
                        .map_err(|e| anyhow::anyhow!("Failed to store edge: {e}"))?;
                }
            }
        }

        Ok(())
    }
}

impl IndexBackend for HelixDbIndex {
    fn sync(&mut self, repo: &IxchelRepo) -> Result<SyncStats> {
        self.rebuild_storage()?;

        let mut stats = SyncStats::default();
        let mut records: Vec<EntityRecord> = Vec::new();
        let mut id_to_node: BTreeMap<String, u128> = BTreeMap::new();

        let mut wtxn = self.begin_write_txn()?;
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;

        for entity_path in iter_entity_paths(repo)? {
            stats.scanned += 1;

            let raw = std::fs::read_to_string(&entity_path)
                .with_context(|| format!("Failed to read {}", entity_path.display()))?;
            let doc = parse_markdown(&entity_path, &raw)?;

            let id = get_string(&doc.frontmatter, "id")
                .or_else(|| {
                    entity_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(std::string::ToString::to_string)
                })
                .unwrap_or_default();
            if id.trim().is_empty() {
                continue;
            }

            let kind = get_string(&doc.frontmatter, "type")
                .and_then(|t| t.parse::<EntityKind>().ok())
                .or_else(|| kind_from_id(&id))
                .unwrap_or(EntityKind::Report);

            let title = get_string(&doc.frontmatter, "title").unwrap_or_default();

            let tags = get_string_list(&doc.frontmatter, "tags");
            let tags_json = serde_json::to_string(&tags).unwrap_or_default();

            let entity_status = get_string(&doc.frontmatter, "status").unwrap_or_default();

            let content_hash = blake3::hash(raw.as_bytes()).to_hex().to_string();
            let normalized_path = normalize_path(&self.repo_root, &entity_path);

            let embedding_text = build_embedding_text(&title, &doc.body, &tags, kind);
            let embedding = self.embed(&embedding_text)?;

            let node_id = Uuid::new_v4().as_u128();
            let vector_id = insert_vector(storage, &mut wtxn, &embedding)?;

            let arena = Bump::new();
            let label = arena.alloc_str(NODE_LABEL);

            let mut props: Vec<(&str, Value)> = Vec::with_capacity(10);
            props.push((arena.alloc_str("id"), Value::String(id.clone())));
            props.push((
                arena.alloc_str("kind"),
                Value::String(kind.as_str().to_string()),
            ));
            props.push((arena.alloc_str("title"), Value::String(title.clone())));
            props.push((arena.alloc_str("status"), Value::String(entity_status)));
            props.push((arena.alloc_str("file_path"), Value::String(normalized_path)));
            props.push((arena.alloc_str("content_hash"), Value::String(content_hash)));
            props.push((
                arena.alloc_str("vector_id"),
                Value::String(vector_id.to_string()),
            ));
            props.push((arena.alloc_str("tags"), Value::String(tags_json)));
            props.push((arena.alloc_str("body"), Value::String(doc.body.clone())));

            let properties = ImmutablePropertiesMap::new(props.len(), props.into_iter(), &arena);
            let node = Node {
                id: node_id,
                label,
                version: 1,
                properties: Some(properties),
            };

            graph_ops::put_node(storage, &mut wtxn, &node)
                .map_err(|e| anyhow::anyhow!("Failed to store node: {e}"))?;
            graph_ops::update_secondary_indices(storage, &mut wtxn, &node)
                .map_err(|e| anyhow::anyhow!("Failed to update secondary index: {e}"))?;

            id_to_node.insert(id.clone(), node_id);
            records.push(EntityRecord {
                from_node: node_id,
                rels: extract_relationships(&doc.frontmatter),
            });

            stats.added += 1;
        }

        self.insert_edges(&mut wtxn, &id_to_node, records)?;

        Self::commit_txn(wtxn)?;
        Ok(stats)
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;
        let embedding = self.embed(query)?;
        let query_f64: Vec<f64> = embedding.iter().map(|&x| f64::from(x)).collect();

        let arena = Bump::new();
        let rtxn = storage
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read transaction: {e}"))?;
        let label = arena.alloc_str(NODE_LABEL);

        let vector_results =
            storage
                .vectors
                .search::<fn(
                    &helix_db::helix_engine::vector_core::vector::HVector<'_>,
                    &RoTxn<'_>,
                ) -> bool>(&rtxn, &query_f64, limit, label, None, false, &arena)
                .map_err(|e| anyhow::anyhow!("Vector search failed: {e:?}"))?;

        let mut hits = Vec::new();

        for hvector in vector_results {
            let vector_id = hvector.id;
            let distance = hvector.get_distance();
            #[allow(clippy::cast_possible_truncation)]
            let score = (1.0 / (1.0 + distance)) as f32;

            let Some(node_id) = lookup_node_by_vector_id(storage, &rtxn, vector_id)? else {
                continue;
            };

            let node = storage
                .get_node(&rtxn, &node_id, &arena)
                .map_err(|e| anyhow::anyhow!("Failed to get node: {e:?}"))?;

            let id = node
                .get_property("id")
                .and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            let title = node
                .get_property("title")
                .and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default();

            let kind = node.get_property("kind").and_then(|v| match v {
                Value::String(s) => s.parse::<EntityKind>().ok(),
                _ => None,
            });

            hits.push(SearchHit {
                score,
                id,
                kind,
                title,
            });
        }

        hits.sort_by(|a, b| b.score.total_cmp(&a.score));
        Ok(hits)
    }

    fn health_check(&self) -> Result<()> {
        let _rtxn = self
            .storage
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?
            .graph_env
            .read_txn()
            .map_err(|e| anyhow::anyhow!("Failed to start read transaction: {e}"))?;
        Ok(())
    }
}

#[derive(Debug)]
struct EntityRecord {
    from_node: u128,
    rels: Vec<(String, Vec<String>)>,
}

fn open_storage(db_path: &Path) -> Result<HelixGraphStorage> {
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

    let path = db_path.to_string_lossy().to_string();
    HelixGraphStorage::new(&path, config, VersionInfo::default())
        .map_err(|e| anyhow::anyhow!("Failed to create storage: {e:?}"))
}

fn normalize_path(repo_root: &Path, file_path: &Path) -> String {
    file_path
        .strip_prefix(repo_root)
        .unwrap_or(file_path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn iter_entity_paths(repo: &IxchelRepo) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();

    for kind in [
        EntityKind::Decision,
        EntityKind::Issue,
        EntityKind::Idea,
        EntityKind::Report,
        EntityKind::Source,
        EntityKind::Citation,
        EntityKind::Agent,
        EntityKind::Session,
    ] {
        let dir = repo.paths.kind_dir(kind);
        if !dir.exists() {
            continue;
        }

        for entry in
            std::fs::read_dir(&dir).with_context(|| format!("Failed to read {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            out.push(path);
        }
    }

    out.sort();
    Ok(out)
}

fn insert_vector<'a>(
    storage: &'a HelixGraphStorage,
    wtxn: &mut RwTxn<'a>,
    embedding: &[f32],
) -> Result<u128> {
    let arena = Bump::new();
    let label = arena.alloc_str(NODE_LABEL);

    let embedding_f64: Vec<f64> = embedding.iter().map(|&x| f64::from(x)).collect();
    let vector =
        storage
            .vectors
            .insert::<fn(
                &helix_db::helix_engine::vector_core::vector::HVector<'_>,
                &RoTxn<'_>,
            ) -> bool>(wtxn, label, &embedding_f64, None, &arena)
            .map_err(|e| anyhow::anyhow!("Failed to insert vector: {e:?}"))?;

    Ok(vector.id)
}

fn lookup_node_by_vector_id(
    storage: &HelixGraphStorage,
    rtxn: &RoTxn<'_>,
    vector_id: u128,
) -> Result<Option<u128>> {
    let key = Value::String(vector_id.to_string());
    graph_ops::lookup_secondary_index(storage, rtxn, "vector_id", &key)
        .map_err(|e| anyhow::anyhow!("Failed to lookup vector_id: {e}"))
}

fn lookup_node_by_entity_id(
    storage: &HelixGraphStorage,
    rtxn: &RoTxn<'_>,
    entity_id: &str,
) -> Result<Option<u128>> {
    let key = Value::String(entity_id.to_string());
    graph_ops::lookup_secondary_index(storage, rtxn, "id", &key)
        .map_err(|e| anyhow::anyhow!("Failed to lookup id: {e}"))
}

fn node_entity_id(
    storage: &HelixGraphStorage,
    rtxn: &RoTxn<'_>,
    node_id: u128,
    arena: &Bump,
) -> Result<Option<String>> {
    let node = storage
        .get_node(rtxn, &node_id, arena)
        .map_err(|e| anyhow::anyhow!("Failed to get node: {e:?}"))?;

    Ok(node.get_property("id").and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        _ => None,
    }))
}

fn build_embedding_text(title: &str, body: &str, tags: &[String], kind: EntityKind) -> String {
    let tags_str = if tags.is_empty() {
        String::new()
    } else {
        tags.join(", ")
    };

    format!(
        "{title}\n\n{body}\n\nTags: {tags_str}\nType: {}\n",
        kind.as_str()
    )
}

fn extract_relationships(frontmatter: &serde_yaml::Mapping) -> Vec<(String, Vec<String>)> {
    let mut rels = Vec::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();

    for (key, value) in frontmatter {
        let YamlValue::String(key) = key else {
            continue;
        };

        if METADATA_KEYS.contains(&key.as_str()) {
            continue;
        }

        let targets = match value {
            YamlValue::Sequence(seq) => seq
                .iter()
                .filter_map(|v| match v {
                    YamlValue::String(s) => Some(s.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            YamlValue::String(s) => vec![s.clone()],
            _ => Vec::new(),
        };

        let targets = targets
            .into_iter()
            .filter(|t| ix_core::entity::looks_like_entity_id(t))
            .collect::<Vec<_>>();

        if targets.is_empty() {
            continue;
        }

        let key = key.trim().to_string();
        if key.is_empty() || !seen.insert(key.clone()) {
            continue;
        }

        rels.push((key, targets));
    }

    rels
}
