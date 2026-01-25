//! SurrealDB-backed storage adapter for Ixchel.
//!
//! This crate provides a [`SurrealDbIndex`] that implements the [`IndexBackend`] trait
//! using `SurrealDB`'s embedded mode with `RocksDB` or `SurrealKV` storage.

mod manifest;
mod schema;
mod types;

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use manifest::{ManifestEntry, SyncAction, SyncManifest};

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
    /// Database connection, lazily initialized on first use.
    /// Uses `Mutex` for interior mutability so read operations can open the DB.
    db: Mutex<Option<Surreal<Db>>>,
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
            db: Mutex::new(None), // Opened lazily
            runtime,
            embedder,
            engine,
        })
    }

    /// Ensure the database is open, opening it lazily if needed.
    ///
    /// This allows read operations (`search`, `health_check`) to work on an existing
    /// database without requiring `sync()` to be called first on the same instance.
    fn ensure_db_open(&self) -> Result<()> {
        let mut db_guard = self
            .db
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {e}"))?;

        if db_guard.is_some() {
            return Ok(());
        }

        // Only open if the database path exists (has been synced before)
        if !self.db_path.exists() {
            anyhow::bail!(
                "Database not found at {}. Run `sync` first to create it.",
                self.db_path.display()
            );
        }

        let db = self
            .runtime
            .block_on(open_database(&self.db_path, &self.engine))?;
        *db_guard = Some(db);
        drop(db_guard);
        Ok(())
    }

    /// Get a reference to the database, ensuring it's open first.
    #[allow(clippy::significant_drop_tightening)]
    fn with_db<T, F>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&Surreal<Db>) -> Result<T>,
    {
        self.ensure_db_open()?;
        let db_guard = self
            .db
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {e}"))?;
        let db = db_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;
        f(db)
    }

    /// Query outgoing relationships from an entity.
    ///
    /// Returns the IDs of entities that `from_id` has the given relationship to.
    pub fn outgoing(&self, from_id: &str, rel: &str) -> Result<Vec<String>> {
        let label = rel.trim().to_ascii_uppercase();
        if label.is_empty() {
            return Ok(Vec::new());
        }

        // Clone to owned values for async block
        let from_id_owned = from_id.to_string();
        let label_owned = label;
        let runtime = self.runtime.clone();

        self.with_db(|db| {
            let result: Vec<types::NeighborResult> = runtime.block_on(async {
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
        })
    }

    /// Query incoming relationships to an entity.
    ///
    /// Returns the IDs of entities that have the given relationship to `to_id`.
    pub fn incoming(&self, to_id: &str, rel: &str) -> Result<Vec<String>> {
        let label = rel.trim().to_ascii_uppercase();
        if label.is_empty() {
            return Ok(Vec::new());
        }

        // Clone to owned values for async block
        let to_id_owned = to_id.to_string();
        let label_owned = label;
        let runtime = self.runtime.clone();

        self.with_db(|db| {
            let result: Vec<types::NeighborResult> = runtime.block_on(async {
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
        })
    }

    fn rebuild_database(&self) -> Result<()> {
        // Explicitly drop the old database connection to release the lock
        {
            let mut db_guard = self
                .db
                .lock()
                .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {e}"))?;
            if let Some(old_db) = db_guard.take() {
                drop(old_db);
            }
        }

        if self.db_path.exists() {
            std::fs::remove_dir_all(&self.db_path)
                .with_context(|| format!("Failed to remove {}", self.db_path.display()))?;
        }

        std::fs::create_dir_all(&self.db_path)
            .with_context(|| format!("Failed to create {}", self.db_path.display()))?;

        let new_db = self
            .runtime
            .block_on(open_database(&self.db_path, &self.engine))?;

        let mut db_guard = self
            .db
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {e}"))?;
        *db_guard = Some(new_db);
        drop(db_guard);
        Ok(())
    }

    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embedder
            .embed(text)
            .map_err(|e| anyhow::anyhow!("Embedding failed: {e}"))
    }

    fn insert_edges(
        &self,
        db: &Surreal<Db>,
        id_to_record_id: &BTreeMap<String, String>,
        records: Vec<PendingRelation>,
    ) -> Result<()> {
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

    /// Load the sync manifest from the database.
    fn load_manifest(&self, db: &Surreal<Db>) -> Result<SyncManifest> {
        let records: Vec<types::ManifestRecord> = self
            .runtime
            .block_on(async { db.query("SELECT * FROM sync_manifest").await?.take(0) })?;

        let entries = records
            .into_iter()
            .map(|r| {
                (
                    r.entity_id,
                    ManifestEntry {
                        content_hash: r.content_hash,
                        file_path: r.file_path,
                        #[allow(clippy::cast_sign_loss)]
                        last_synced: r.last_synced as u64,
                    },
                )
            })
            .collect();

        Ok(SyncManifest::from_entries(entries))
    }

    /// Save or update a manifest entry in the database.
    fn save_manifest_entry(
        &self,
        db: &Surreal<Db>,
        entity_id: &str,
        entry: &ManifestEntry,
    ) -> Result<()> {
        let entity_id_owned = entity_id.to_string();
        let content_hash = entry.content_hash.clone();
        let file_path = entry.file_path.clone();
        #[allow(clippy::cast_possible_wrap)]
        let last_synced = entry.last_synced as i64;

        self.runtime.block_on(async {
            // Use UPSERT to insert or update
            db.query(
                "UPSERT type::thing('sync_manifest', $entity_id) CONTENT {
                    entity_id: $entity_id,
                    content_hash: $content_hash,
                    file_path: $file_path,
                    last_synced: $last_synced
                }",
            )
            .bind(("entity_id", entity_id_owned))
            .bind(("content_hash", content_hash))
            .bind(("file_path", file_path))
            .bind(("last_synced", last_synced))
            .await?;
            Ok::<_, anyhow::Error>(())
        })
    }

    /// Delete an entity and its relationships from the database.
    fn delete_entity(&self, db: &Surreal<Db>, entity_id: &str) -> Result<()> {
        let entity_id_owned = entity_id.to_string();

        self.runtime.block_on(async {
            // Delete relationships first
            db.query(
                "DELETE relates WHERE in.entity_id = $entity_id OR out.entity_id = $entity_id",
            )
            .bind(("entity_id", entity_id_owned.clone()))
            .await?;

            // Delete the entity
            db.query("DELETE FROM entity WHERE entity_id = $entity_id")
                .bind(("entity_id", entity_id_owned))
                .await?;

            Ok::<_, anyhow::Error>(())
        })
    }

    /// Delete a manifest entry from the database.
    fn delete_manifest_entry(&self, db: &Surreal<Db>, entity_id: &str) -> Result<()> {
        let entity_id_owned = entity_id.to_string();

        self.runtime.block_on(async {
            db.query("DELETE type::thing('sync_manifest', $entity_id)")
                .bind(("entity_id", entity_id_owned))
                .await?;
            Ok::<_, anyhow::Error>(())
        })
    }

    /// Update an existing entity in the database.
    fn update_entity(&self, db: &Surreal<Db>, record: &EntityRecord) -> Result<()> {
        let entity_id = record.entity_id.clone();
        let record = record.clone();

        self.runtime.block_on(async {
            db.query(
                "UPDATE entity SET
                    kind = $kind,
                    title = $title,
                    status = $status,
                    file_path = $file_path,
                    content_hash = $content_hash,
                    tags = $tags,
                    body = $body,
                    embedding = $embedding
                WHERE entity_id = $entity_id",
            )
            .bind(("entity_id", entity_id))
            .bind(("kind", record.kind))
            .bind(("title", record.title))
            .bind(("status", record.status))
            .bind(("file_path", record.file_path))
            .bind(("content_hash", record.content_hash))
            .bind(("tags", record.tags))
            .bind(("body", record.body))
            .bind(("embedding", record.embedding))
            .await?;
            Ok::<_, anyhow::Error>(())
        })
    }

    /// Delete all relationships involving an entity.
    fn delete_entity_edges(&self, db: &Surreal<Db>, entity_id: &str) -> Result<()> {
        let entity_id_owned = entity_id.to_string();

        self.runtime.block_on(async {
            db.query(
                "DELETE relates WHERE in.entity_id = $entity_id OR out.entity_id = $entity_id",
            )
            .bind(("entity_id", entity_id_owned))
            .await?;
            Ok::<_, anyhow::Error>(())
        })
    }
}

impl IndexBackend for SurrealDbIndex {
    /// Sync entities from the filesystem to the database.
    ///
    /// Uses incremental sync by default: compares content hashes against
    /// a stored manifest and only updates changed entities.
    #[allow(clippy::significant_drop_tightening, clippy::too_many_lines)]
    fn sync(&mut self, repo: &IxchelRepo) -> Result<SyncStats> {
        // Check if database exists - if not, do a full rebuild
        let db_exists = self.db_path.exists();

        if db_exists {
            // Open existing database
            self.ensure_db_open()?;
        } else {
            self.rebuild_database()?;
        }

        // Lock the database for the duration of sync
        let db_guard = self
            .db
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to acquire database lock: {e}"))?;
        let db = db_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        // Initialize/update schema with embedding dimension
        let dimension = self.embedder.dimension();
        self.runtime.block_on(async {
            db.query(schema::SCHEMA_INIT).await?;
            db.query(schema::hnsw_index_query(dimension)).await
        })?;

        // Load existing manifest for incremental sync
        let mut manifest = if db_exists {
            self.load_manifest(db).unwrap_or_default()
        } else {
            SyncManifest::new()
        };

        let mut stats = SyncStats::default();
        let mut pending_relations: Vec<PendingRelation> = Vec::new();
        let mut id_to_record_id: BTreeMap<String, String> = BTreeMap::new();
        let mut seen_entity_ids: HashSet<String> = HashSet::new();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

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

            seen_entity_ids.insert(id.clone());
            let content_hash = blake3::hash(raw.as_bytes()).to_hex().to_string();
            let normalized_path = normalize_path(&self.repo_root, &entity_path);

            // Determine action based on manifest (checks both content hash and file path)
            let action = manifest.action_for(&id, &content_hash, &normalized_path);

            match action {
                SyncAction::Skip => {
                    // Entity unchanged - edges are already correct, no need to re-insert
                    stats.unchanged += 1;
                    // Still track the ID for relationship target resolution
                    id_to_record_id.insert(id.clone(), id.clone());
                    continue;
                }
                SyncAction::Insert | SyncAction::Update => {
                    // Need to process this entity
                }
            }

            let kind = get_string(&doc.frontmatter, "type")
                .and_then(|t| t.parse::<EntityKind>().ok())
                .or_else(|| kind_from_id(&id))
                .unwrap_or(EntityKind::Report);

            let title = get_string(&doc.frontmatter, "title").unwrap_or_default();
            let tags = get_string_list(&doc.frontmatter, "tags");
            let entity_status = get_string(&doc.frontmatter, "status").unwrap_or_default();

            let embedding_text = build_embedding_text(&title, &doc.body, &tags, kind);
            let embedding = self.embed(&embedding_text)?;

            let record = EntityRecord {
                record_id: None,
                entity_id: id.clone(),
                kind: kind.as_str().to_string(),
                title: title.clone(),
                status: entity_status,
                file_path: normalized_path.clone(),
                content_hash: content_hash.clone(),
                tags,
                body: doc.body.clone(),
                embedding,
            };

            match action {
                SyncAction::Insert => {
                    // Insert new entity
                    let record_id_key = id.clone();
                    self.runtime.block_on(async {
                        db.query("CREATE type::thing('entity', $record_id) CONTENT $content")
                            .bind(("record_id", record_id_key.clone()))
                            .bind(("content", record))
                            .await?;
                        Ok::<_, anyhow::Error>(())
                    })?;
                    stats.added += 1;
                }
                SyncAction::Update => {
                    // Delete old edges first, then update entity
                    self.delete_entity_edges(db, &id)?;
                    self.update_entity(db, &record)?;
                    stats.modified += 1;
                }
                SyncAction::Skip => unreachable!(),
            }

            // Update manifest entry
            let manifest_entry = ManifestEntry {
                content_hash,
                file_path: normalized_path,
                last_synced: now,
            };
            self.save_manifest_entry(db, &id, &manifest_entry)?;
            manifest.insert(id.clone(), manifest_entry);

            id_to_record_id.insert(id.clone(), id.clone());
            pending_relations.push(PendingRelation {
                from_record_id: id,
                rels: extract_relationships(&doc.frontmatter),
            });
        }

        // Find and delete entities that no longer exist on disk
        let manifest_ids: Vec<String> = manifest.entity_ids().cloned().collect();
        for entity_id in manifest_ids {
            if !seen_entity_ids.contains(&entity_id) {
                // Entity was deleted from disk
                self.delete_entity(db, &entity_id)?;
                self.delete_manifest_entry(db, &entity_id)?;
                stats.deleted += 1;
            }
        }

        // Insert edges only for new and modified entities
        // Unchanged entities keep their existing edges (skipped above)
        // For Update, edges were deleted in delete_entity_edges before update
        // For Insert, no old edges exist
        self.insert_edges(db, &id_to_record_id, pending_relations)?;

        Ok(stats)
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
        let embedding = self.embed(query)?;
        let runtime = self.runtime.clone();

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

        self.with_db(|db| {
            let results: Vec<SearchResult> = runtime.block_on(async {
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
        })
    }

    fn health_check(&self) -> Result<()> {
        let runtime = self.runtime.clone();

        self.with_db(|db| {
            runtime.block_on(async {
                let _: Vec<serde_json::Value> =
                    db.query("SELECT * FROM entity LIMIT 1").await?.take(0)?;
                Ok(())
            })
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
