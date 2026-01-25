//! SurrealDB-backed storage adapter for Ixchel.
//!
//! This crate provides a [`SurrealDbIndex`] that implements the [`IndexBackend`] trait
//! using `SurrealDB`'s embedded mode with `RocksDB` or `SurrealKV` storage.

mod schema;
mod types;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use ix_core::entity::{EntityKind, kind_from_id};
use ix_core::index::{IndexBackend, SearchHit, SyncStats};
use ix_core::markdown::{get_string, get_string_list, parse_markdown};
use ix_core::repo::IxchelRepo;
use ix_embeddings::Embedder;
use serde_yaml::Value as YamlValue;
use surrealdb::Surreal;
use surrealdb::engine::local::{Db, RocksDb, SurrealKv};
use tokio::runtime::Runtime;

pub use types::{EntityRecord, SearchResult};

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

/// SurrealDB-backed index for Ixchel entities.
///
/// Uses `SurrealDB`'s embedded mode with either `RocksDB` or `SurrealKV` for persistence.
/// Supports graph relationships and HNSW vector similarity search.
pub struct SurrealDbIndex {
    repo_root: PathBuf,
    db_path: PathBuf,
    db: Option<Surreal<Db>>,
    runtime: Arc<Runtime>,
    embedder: Embedder,
    engine: String,
}

impl SurrealDbIndex {
    /// Open a `SurrealDB` index for the given repository.
    ///
    /// Uses the embedding configuration from the repository config.
    pub fn open(repo: &IxchelRepo) -> Result<Self> {
        let embedder = Embedder::with_config(&repo.config.embedding)
            .map_err(|e| anyhow::anyhow!("Failed to initialize embedder: {e}"))?;
        Self::open_with_embedder(repo, embedder)
    }

    /// Open a `SurrealDB` index with a custom embedder.
    ///
    /// Useful for testing with deterministic embedding providers.
    /// Note: The database is opened lazily on first use to avoid lock conflicts during sync.
    pub fn open_with_embedder(repo: &IxchelRepo, embedder: Embedder) -> Result<Self> {
        let repo_root = repo.paths.repo_root().to_path_buf();
        let db_path = repo
            .paths
            .ixchel_dir()
            .join(PathBuf::from(&repo.config.storage.path));

        let runtime =
            Arc::new(Runtime::new().context("Failed to create tokio runtime for SurrealDB")?);

        // Get engine from config, default to surrealkv (pure Rust, better locking behavior)
        let engine = repo
            .config
            .storage
            .engine
            .clone()
            .unwrap_or_else(|| "surrealkv".to_string());

        Ok(Self {
            repo_root,
            db_path,
            db: None, // Opened lazily
            runtime,
            embedder,
            engine,
        })
    }

    /// Query outgoing relationships from an entity.
    ///
    /// Returns the IDs of entities that `from_id` has the given relationship to.
    pub fn outgoing(&self, from_id: &str, rel: &str) -> Result<Vec<String>> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let label = rel.trim().to_ascii_uppercase();
        if label.is_empty() {
            return Ok(Vec::new());
        }

        // Clone to owned values for async block
        let from_id_owned = from_id.to_string();
        let label_owned = label;

        let result: Vec<types::NeighborResult> = self.runtime.block_on(async {
            db.query(
                "SELECT out.entity_id AS entity_id FROM relates WHERE in.entity_id = $from_id AND label = $label",
            )
            .bind(("from_id", from_id_owned))
            .bind(("label", label_owned))
            .await?
            .take(0)
        })?;

        let mut out: Vec<String> = result.into_iter().map(|r| r.entity_id).collect();
        out.sort();
        out.dedup();
        Ok(out)
    }

    /// Query incoming relationships to an entity.
    ///
    /// Returns the IDs of entities that have the given relationship to `to_id`.
    pub fn incoming(&self, to_id: &str, rel: &str) -> Result<Vec<String>> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let label = rel.trim().to_ascii_uppercase();
        if label.is_empty() {
            return Ok(Vec::new());
        }

        // Clone to owned values for async block
        let to_id_owned = to_id.to_string();
        let label_owned = label;

        let result: Vec<types::NeighborResult> = self.runtime.block_on(async {
            db.query(
                "SELECT in.entity_id AS entity_id FROM relates WHERE out.entity_id = $to_id AND label = $label",
            )
            .bind(("to_id", to_id_owned))
            .bind(("label", label_owned))
            .await?
            .take(0)
        })?;

        let mut out: Vec<String> = result.into_iter().map(|r| r.entity_id).collect();
        out.sort();
        out.dedup();
        Ok(out)
    }

    fn rebuild_database(&mut self) -> Result<()> {
        // Explicitly drop the old database connection to release the lock
        if let Some(old_db) = self.db.take() {
            drop(old_db);
        }

        if self.db_path.exists() {
            std::fs::remove_dir_all(&self.db_path)
                .with_context(|| format!("Failed to remove {}", self.db_path.display()))?;
        }

        std::fs::create_dir_all(&self.db_path)
            .with_context(|| format!("Failed to create {}", self.db_path.display()))?;

        self.db = Some(
            self.runtime
                .block_on(open_database(&self.db_path, &self.engine))?,
        );
        Ok(())
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embedder
            .embed(text)
            .map_err(|e| anyhow::anyhow!("Embedding failed: {e}"))
    }

    fn insert_edges(
        &self,
        id_to_record_id: &BTreeMap<String, String>,
        records: Vec<PendingRelation>,
    ) -> Result<()> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        for record in records {
            for (rel, to_ids) in record.rels {
                let label = rel.to_ascii_uppercase();
                for to_id in to_ids {
                    let Some(to_record_id) = id_to_record_id.get(&to_id) else {
                        continue;
                    };

                    // Clone values for async block
                    let from_record_id = record.from_record_id.clone();
                    let to_record_id = to_record_id.clone();
                    let label_owned = label.clone();

                    self.runtime.block_on(async {
                        db.query("RELATE $from->relates->$to SET label = $label")
                            .bind((
                                "from",
                                surrealdb::RecordId::from(("entity", from_record_id.as_str())),
                            ))
                            .bind((
                                "to",
                                surrealdb::RecordId::from(("entity", to_record_id.as_str())),
                            ))
                            .bind(("label", label_owned))
                            .await
                    })?;
                }
            }
        }

        Ok(())
    }
}

impl IndexBackend for SurrealDbIndex {
    fn sync(&mut self, repo: &IxchelRepo) -> Result<SyncStats> {
        self.rebuild_database()?;

        // Initialize schema with embedding dimension
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let dimension = self.embedder.dimension();
        self.runtime.block_on(async {
            db.query(schema::SCHEMA_INIT).await?;
            db.query(schema::hnsw_index_query(dimension)).await
        })?;

        let mut stats = SyncStats::default();
        let mut pending_relations: Vec<PendingRelation> = Vec::new();
        let mut id_to_record_id: BTreeMap<String, String> = BTreeMap::new();

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
            let entity_status = get_string(&doc.frontmatter, "status").unwrap_or_default();
            let content_hash = blake3::hash(raw.as_bytes()).to_hex().to_string();
            let normalized_path = normalize_path(&self.repo_root, &entity_path);

            let embedding_text = build_embedding_text(&title, &doc.body, &tags, kind);
            let embedding = self.embed(&embedding_text)?;

            // Use the entity ID as the record ID key for simplicity
            let record_id_key = id.clone();

            let record = EntityRecord {
                record_id: None,
                entity_id: id.clone(),
                kind: kind.as_str().to_string(),
                title: title.clone(),
                status: entity_status,
                file_path: normalized_path,
                content_hash,
                tags,
                body: doc.body.clone(),
                embedding,
            };

            // Insert entity using raw query to avoid record ID deserialization issues
            let record_id_key_owned = record_id_key.clone();
            self.runtime.block_on(async {
                db.query("CREATE type::thing('entity', $record_id) CONTENT $content")
                    .bind(("record_id", record_id_key_owned))
                    .bind(("content", record))
                    .await?;
                Ok::<_, anyhow::Error>(())
            })?;

            id_to_record_id.insert(id.clone(), record_id_key.clone());
            pending_relations.push(PendingRelation {
                from_record_id: record_id_key,
                rels: extract_relationships(&doc.frontmatter),
            });

            stats.added += 1;
        }

        self.insert_edges(&id_to_record_id, pending_relations)?;

        Ok(stats)
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let embedding = self.embed(query)?;

        // Use HNSW KNN search - results come back ordered by distance
        // The <|K,EF|> operator returns K nearest neighbors with ef_search=EF
        // Distance function (COSINE) is defined in the index, not the query
        // Note: KNN operator requires literal for K, so we format it into the query
        let ef_search = 64; // Higher = more accurate but slower
        let query_str = format!(
            "SELECT entity_id, kind, title, vector::distance::knn() AS distance \
             FROM entity \
             WHERE embedding <|{limit},{ef_search}|> $query_embedding \
             ORDER BY distance"
        );
        let results: Vec<SearchResult> = self.runtime.block_on(async {
            db.query(&query_str)
                .bind(("query_embedding", embedding))
                .await?
                .take(0)
        })?;

        // Convert distance to score (lower distance = higher score)
        let hits = results
            .into_iter()
            .map(|r| {
                // Cosine distance: 0 = identical, 2 = opposite
                // Convert to score: 1/(1+distance) gives ~1.0 for identical, ~0.33 for opposite
                #[allow(clippy::cast_possible_truncation)]
                let score = (1.0 / (1.0 + r.distance)) as f32;
                let kind = r.kind.and_then(|k| k.parse::<EntityKind>().ok());
                SearchHit {
                    score,
                    id: r.entity_id,
                    kind,
                    title: r.title,
                }
            })
            .collect();

        Ok(hits)
    }

    fn health_check(&self) -> Result<()> {
        let db = self
            .db
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        self.runtime.block_on(async {
            let _: Vec<serde_json::Value> =
                db.query("SELECT * FROM entity LIMIT 1").await?.take(0)?;
            Ok(())
        })
    }
}

#[derive(Debug)]
struct PendingRelation {
    from_record_id: String,
    rels: Vec<(String, Vec<String>)>,
}

async fn open_database(db_path: &Path, engine: &str) -> Result<Surreal<Db>> {
    let path = db_path.to_string_lossy().to_string();

    let db: Surreal<Db> = match engine.to_ascii_lowercase().as_str() {
        "rocksdb" => Surreal::new::<RocksDb>(&path).await?,
        "surrealkv" => Surreal::new::<SurrealKv>(&path).await?,
        e => anyhow::bail!("Unknown SurrealDB engine: {e}"),
    };

    // Select namespace and database
    db.use_ns("ixchel").use_db("main").await?;

    Ok(db)
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
