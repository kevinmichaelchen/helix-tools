use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use bumpalo::Bump;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use heed3::{RoTxn, RwTxn};
use helix_db::helix_engine::storage_core::storage_methods::StorageMethods;
use helix_db::helix_engine::storage_core::{HelixGraphStorage, version_info::VersionInfo};
use helix_db::helix_engine::traversal_core::config::{Config, GraphConfig, VectorConfig};
use helix_db::helix_engine::types::SecondaryIndex;
use helix_db::helix_engine::vector_core::hnsw::HNSW;
use helix_db::protocol::value::Value;
use helix_db::utils::items::{Edge, Node};
use helix_db::utils::properties::ImmutablePropertiesMap;
use helix_graph_ops as graph_ops;
use ix_core::entity::{EntityKind, kind_from_id};
use ix_core::index::{IndexBackend, SearchHit, SyncStats};
use ix_core::markdown::{get_string, get_string_list, parse_markdown};
use ix_core::repo::IxchelRepo;
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
    embedder: Mutex<TextEmbedding>,
}

impl HelixDbIndex {
    pub fn open(repo: &IxchelRepo) -> Result<Self> {
        let repo_root = repo.paths.repo_root().to_path_buf();
        let db_path = repo
            .paths
            .ixchel_dir()
            .join(PathBuf::from(&repo.config.storage.path));

        std::fs::create_dir_all(&db_path)
            .with_context(|| format!("Failed to create {}", db_path.display()))?;

        let storage = Some(open_storage(&db_path)?);
        let embedder = Mutex::new(open_embedder(&repo.config.embedding)?);

        Ok(Self {
            repo_root,
            db_path,
            storage,
            embedder,
        })
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

    fn commit_txn(&self, wtxn: RwTxn<'_>) -> Result<()> {
        wtxn.commit()
            .map_err(|e| anyhow::anyhow!("Failed to commit transaction: {e}"))
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let model = self
            .embedder
            .lock()
            .map_err(|_| anyhow::anyhow!("Embedding model lock poisoned"))?;
        let mut embeddings = model
            .embed(vec![text], None)
            .map_err(|e| anyhow::anyhow!("Embedding failed: {e}"))?;
        embeddings
            .pop()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))
    }
}

impl IndexBackend for HelixDbIndex {
    fn sync(&mut self, repo: &IxchelRepo) -> Result<SyncStats> {
        self.rebuild_storage()?;

        let mut stats = SyncStats::default();
        let mut records: Vec<EntityRecord> = Vec::new();
        let mut id_to_node: BTreeMap<String, u128> = BTreeMap::new();

        let mut wtxn = self.begin_write_txn()?;

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
                        .map(|s| s.to_string())
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

            let status = get_string(&doc.frontmatter, "status").unwrap_or_default();

            let content_hash = blake3::hash(raw.as_bytes()).to_hex().to_string();
            let normalized_path = normalize_path(&self.repo_root, &entity_path);

            let embedding_text = build_embedding_text(&title, &doc.body, &tags, kind);
            let embedding = self.embed(&embedding_text)?;

            let node_id = Uuid::new_v4().as_u128();
            let storage = self
                .storage
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;
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
            props.push((arena.alloc_str("status"), Value::String(status)));
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

            let storage = self
                .storage
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;

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

                    let storage = self
                        .storage
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Storage not initialized"))?;
                    graph_ops::put_edge(storage, &mut wtxn, &edge)
                        .map_err(|e| anyhow::anyhow!("Failed to store edge: {e}"))?;
                }
            }
        }

        self.commit_txn(wtxn)?;
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
            let distance = hvector.get_distance() as f32;
            let score = 1.0 / (1.0 + distance);

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

fn open_embedder(config: &ix_core::config::EmbeddingConfig) -> Result<TextEmbedding> {
    if config.provider.trim() != "fastembed" {
        anyhow::bail!("Unsupported embedding provider: {}", config.provider);
    }

    let model = fastembed_model_from_string(&config.model)?;

    TextEmbedding::try_new(InitOptions::new(model))
        .map_err(|e| anyhow::anyhow!("Failed to initialize embedding model: {e}"))
}

fn fastembed_model_from_string(model_name: &str) -> Result<EmbeddingModel> {
    let trimmed = model_name.trim();
    if trimmed.is_empty() {
        anyhow::bail!("Unsupported embedding model: {model_name}");
    }

    if let Ok(model) = trimmed.parse() {
        return Ok(model);
    }

    let needle = normalize_model_token(trimmed);
    let needle_suffix = normalize_model_token(trimmed.rsplit('/').next().unwrap_or(trimmed));

    for info in TextEmbedding::list_supported_models() {
        for candidate in model_identifiers(&info.model_code) {
            if candidate == needle || candidate == needle_suffix {
                return Ok(info.model);
            }
        }
    }

    anyhow::bail!("Unsupported embedding model: {model_name}")
}

fn model_identifiers(model_code: &str) -> Vec<String> {
    let normalized = normalize_model_token(model_code);
    let suffix = model_code.rsplit('/').next().unwrap_or(model_code);
    let suffix_normalized = normalize_model_token(suffix);

    let mut identifiers = vec![normalized, suffix_normalized];

    for value in [suffix.strip_suffix("-onnx"), suffix.strip_suffix("-onnx-q")]
        .into_iter()
        .flatten()
    {
        identifiers.push(normalize_model_token(value));
    }

    identifiers
}

fn normalize_model_token(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
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
