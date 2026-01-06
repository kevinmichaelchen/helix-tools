# helix-decisions: Design Specification

**Document:** design.md  
**Status:** In Progress (2026-01-05)  
**Author:** Kevin Chen

## Design Philosophy

Decisions are not isolated documents—they form a **decision graph**:
- Decision 005 **supersedes** Decision 002
- Decision 007 **amends** Decision 003  
- Decision 004 **relates to** Decision 006
- Decision 008 **depends on** Decision 001

This graph structure enables powerful queries beyond simple search:
- "What's the current decision?" → Follow supersedes chain to leaf
- "Why was this changed?" → Traverse back through supersedes history
- "What else is affected?" → Find related and dependent decisions

## Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                helix-decisions CLI                   │
│  • search <query>     - Semantic vector search       │
│  • chain <id>         - Show supersedes chain        │
│  • related <id>       - Find related decisions       │
└──────────────────────────┬──────────────────────────┘
                           │
┌──────────────────────────▼──────────────────────────┐
│                  DecisionSearcher                    │
│  • sync()    - Delta index decisions + relationships │
│  • search()  - Vector similarity + graph context     │
│  • chain()   - Traverse supersedes edges             │
│  • related() - Find connected decisions              │
└──────────────────────────┬──────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         │                 │                 │
┌────────▼─────┐   ┌───────▼───────┐   ┌─────▼──────┐
│   Loader     │   │   Embedder    │   │  Storage   │
│  (YAML/MD)   │   │  (fastembed)  │   │ (HelixDB)  │
│              │   │               │   │            │
│ • Parse      │   │ • Local embed │   │ • Nodes    │
│   decisions  │   │ • 384-dim     │   │ • Vectors  │
│ • Extract    │   │ • MiniLM-L6   │   │ • Edges    │
│   relations  │   │               │   │            │
└──────────────┘   └───────────────┘   └────────────┘
```

## Graph Schema

### Node: Decision

Decisions are stored as graph nodes with properties and vector embeddings.

```
┌─────────────────────────────────────────┐
│ Node: Decision                          │
├─────────────────────────────────────────┤
│ id: u128 (HelixDB internal)             │
│ label: "decision"                       │
├─────────────────────────────────────────┤
│ Properties:                             │
│   decision_id: u32     # Local number   │
│   uuid: String         # Global hash ID │
│   title: String                         │
│   status: String       # enum as string │
│   date: String         # ISO 8601       │
│   deciders: [String]                    │
│   tags: [String]                        │
│   file_path: String                     │
│   content_hash: String # for delta      │
│   git_commit: String   # immutability   │
│   body: String         # markdown text  │
├─────────────────────────────────────────┤
│ Vector: 384-dim embedding of body       │
└─────────────────────────────────────────┘
```

## ID Scheme

### Local ID (`id`)
- Sequential integer (1, 2, 3...)
- Human-readable and easy to reference
- Unique within a single repository
- Used in filenames: `003-database-migration.md`

### Global UUID (`uuid`)
- Optional hash-based identifier via helix-id
- Format: `hx-xxxxxx` (6 hex chars from Blake3 hash)
- Safe for distributed collaboration across branches
- Generated from decision content or random UUID
- Prevents merge conflicts when multiple developers create decisions

### Why Both?
- `id`: For humans ("see decision 5")
- `uuid`: For machines and cross-repo references
- Local IDs can conflict across branches; UUIDs cannot

## Immutability Model

### Soft Immutability via Git

Decisions become immutable once accepted:

1. **content_hash**: SHA-256 of decision content at acceptance
2. **git_commit**: Git commit hash when status changed to `accepted`

### Amendment Pattern

Instead of modifying accepted decisions:
- Create new decision with `amends: [original_id]`
- Original remains unchanged for audit trail
- Search returns both, with amendment relationship visible

### Supersession Pattern

When a decision is replaced entirely:
- Create new decision with `supersedes: [old_id]`
- Set old decision status to `superseded`
- Graph traversal shows evolution chain

### Edges: Relationships

```
┌──────────────────────────────────────────────────────────┐
│ Edge Types                                                │
├───────────────┬──────────────────────────────────────────┤
│ SUPERSEDES    │ Decision A supersedes Decision B         │
│               │ Direction: A → B                          │
│               │ Inverse: B.superseded_by = A              │
├───────────────┼──────────────────────────────────────────┤
│ AMENDS        │ Decision A modifies Decision B           │
│               │ Direction: A → B                          │
│               │ (B remains valid with amendments)         │
├───────────────┼──────────────────────────────────────────┤
│ DEPENDS_ON    │ Decision A requires Decision B           │
│               │ Direction: A → B                          │
│               │ (A assumes B is accepted)                 │
├───────────────┼──────────────────────────────────────────┤
│ RELATED_TO    │ Decision A and B are topically related   │
│               │ Direction: bidirectional (A ↔ B)         │
│               │ (informational link only)                 │
└───────────────┴──────────────────────────────────────────┘
```

### Example Graph

```
     001 (Database Choice)
         │
         │ SUPERSEDES
         ▼
     003 (PostgreSQL Selection)
         │
    ┌────┴────┐
    │         │
 AMENDS    RELATED_TO
    │         │
    ▼         ▼
   007       004
(Indexes) (Caching)
              │
          DEPENDS_ON
              │
              ▼
             006
        (Redis Choice)
```

## Module Design

### types.rs
```rust
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Decision status values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Proposed,
    Accepted,
    Superseded,
    Deprecated,
}

/// Relationship types between decisions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    /// This decision replaces another (makes it obsolete)
    Supersedes,
    /// This decision modifies another without replacing it
    Amends,
    /// This decision requires another's decision to be in effect
    DependsOn,
    /// This decision is topically related to another
    RelatedTo,
}

impl RelationType {
    /// Edge label for HelixDB storage
    pub fn as_edge_label(&self) -> &'static str {
        match self {
            Self::Supersedes => "SUPERSEDES",
            Self::Amends => "AMENDS",
            Self::DependsOn => "DEPENDS_ON",
            Self::RelatedTo => "RELATED_TO",
        }
    }
}

/// A relationship from this decision to another
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub relation_type: RelationType,
    pub target_id: u32,
}

/// Decision metadata from YAML frontmatter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMetadata {
    pub id: u32,
    #[serde(default)]
    pub uuid: Option<String>,  // Global hash-based ID
    pub title: String,
    pub status: Status,
    pub date: NaiveDate,
    #[serde(default)]
    pub deciders: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content_hash: Option<String>,  // For immutability proof
    #[serde(default)]
    pub git_commit: Option<String>,    // Commit when accepted
    
    // Relationship fields (all optional, can be single ID or array)
    #[serde(default)]
    pub supersedes: Option<OneOrMany<u32>>,
    #[serde(default)]
    pub superseded_by: Option<u32>,  // Inverse, usually auto-set
    #[serde(default)]
    pub amends: Option<OneOrMany<u32>>,
    #[serde(default)]
    pub depends_on: Option<OneOrMany<u32>>,
    #[serde(default)]
    pub related_to: Option<OneOrMany<u32>>,
}

/// Helper for YAML fields that can be single value or array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T: Clone> OneOrMany<T> {
    pub fn to_vec(&self) -> Vec<T> {
        match self {
            Self::One(v) => vec![v.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

impl DecisionMetadata {
    /// Extract all outgoing relationships from metadata
    pub fn relationships(&self) -> Vec<Relationship> {
        let mut rels = Vec::new();
        
        if let Some(ref ids) = self.supersedes {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::Supersedes,
                    target_id: id,
                });
            }
        }
        if let Some(ref ids) = self.amends {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::Amends,
                    target_id: id,
                });
            }
        }
        if let Some(ref ids) = self.depends_on {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::DependsOn,
                    target_id: id,
                });
            }
        }
        if let Some(ref ids) = self.related_to {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::RelatedTo,
                    target_id: id,
                });
            }
        }
        
        rels
    }
}

/// Full decision with body and computed fields
#[derive(Debug, Clone)]
pub struct Decision {
    pub metadata: DecisionMetadata,
    pub body: String,
    pub file_path: PathBuf,
    pub content_hash: String,
    pub embedding: Option<Vec<f32>>,
}

/// Search result with relevance score
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub id: u32,
    pub uuid: Option<String>,
    pub title: String,
    pub status: Status,
    pub score: f32,
    pub tags: Vec<String>,
    pub date: NaiveDate,
    pub deciders: Vec<String>,
    pub file_path: PathBuf,
    /// Related decisions found via graph traversal
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedDecision>,
}

/// Minimal info for related decision references
#[derive(Debug, Clone, Serialize)]
pub struct RelatedDecision {
    pub id: u32,
    pub title: String,
    pub relation: RelationType,
}

/// Search response envelope
#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub count: usize,
    pub results: Vec<SearchResult>,
}

/// Chain response for supersedes traversal
#[derive(Debug, Serialize)]
pub struct ChainResponse {
    pub root_id: u32,
    pub chain: Vec<ChainNode>,
}

#[derive(Debug, Serialize)]
pub struct ChainNode {
    pub id: u32,
    pub title: String,
    pub status: Status,
    pub date: NaiveDate,
    pub is_current: bool,  // true for leaf (not superseded)
}
```

### loader.rs
```rust
use crate::types::{Decision, DecisionMetadata};
use anyhow::Result;
use gray_matter::{engine::YAML, Matter};
use sha2::{Digest, Sha256};
use std::path::Path;

/// Load all decisions from a directory
pub fn load_decisions(dir: &Path) -> Result<Vec<Decision>> {
    let mut decisions = Vec::new();
    let matter = Matter::<YAML>::new();
    
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().map_or(false, |e| e == "md") {
            match load_decision(&path, &matter) {
                Ok(decision) => decisions.push(decision),
                Err(e) => eprintln!("Warning: Skipping {}: {}", path.display(), e),
            }
        }
    }
    
    Ok(decisions)
}

fn load_decision(path: &Path, matter: &Matter<YAML>) -> Result<Decision> {
    let content = std::fs::read_to_string(path)?;
    let parsed = matter.parse(&content);
    
    let metadata: DecisionMetadata = parsed
        .data
        .ok_or_else(|| anyhow::anyhow!("Missing frontmatter"))?
        .deserialize()?;
    
    let body = parsed.content;
    let content_hash = hash_content(&content);
    
    Ok(Decision {
        metadata,
        body,
        file_path: path.to_path_buf(),
        content_hash,
        embedding: None,
    })
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

### embeddings.rs
```rust
use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2))?;
        Ok(Self { model })
    }
    
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text], None)?;
        Ok(embeddings.into_iter().next().unwrap())
    }
    
    pub fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        self.model.embed(texts, None).map_err(Into::into)
    }
}
```

### storage.rs
```rust
use crate::types::{Decision, RelationType, Relationship, ChainNode};
use anyhow::Result;
use helix_db::helix_engine::{
    storage_core::HelixGraphStorage,
    traversal_core::config::Config,
    vector_core::hnsw::HNSW,
};
use std::collections::HashMap;
use std::path::Path;

/// Graph-vector storage for decisions using embedded HelixDB
pub struct DecisionStorage {
    storage: HelixGraphStorage,
    /// Map from decision ID (u32) to HelixDB node ID (u128)
    decision_id_map: HashMap<u32, u128>,
}

impl DecisionStorage {
    /// Open or create the HelixDB storage at ~/.helix/data/decisions/
    pub fn open() -> Result<Self> {
        let data_dir = dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot find home directory"))?
            .join(".helix/data/decisions");
        
        Self::open_at(&data_dir)
    }
    
    /// Open at a specific path (useful for testing)
    pub fn open_at(path: &Path) -> Result<Self> {
        std::fs::create_dir_all(path)?;
        
        let config = Config {
            vector_config: Some(VectorConfig {
                m: Some(16),
                ef_construction: Some(128),
                ef_search: Some(64),  // Smaller for local use
            }),
            graph_config: Some(GraphConfig {
                secondary_indices: Some(vec!["decision_id".to_string()]),
            }),
            db_max_size_gb: Some(1),  // 1GB is plenty for decisions
            bm25: Some(false),        // Don't need BM25 for decisions
            ..Default::default()
        };
        
        let version_info = VersionInfo::current();
        let storage = HelixGraphStorage::new(
            path.to_str().unwrap(),
            config,
            version_info,
        )?;
        
        Ok(Self {
            storage,
            decision_id_map: HashMap::new(),
        })
    }
    
    /// Index a decision as a node with vector embedding
    pub fn index_decision(&mut self, decision: &Decision) -> Result<u128> {
        let arena = bumpalo::Bump::new();
        let mut txn = self.storage.graph_env.write_txn()?;
        
        // Create properties map
        let properties = self.decision_to_properties(decision, &arena);
        
        // Insert vector (embedding)
        let embedding: Vec<f64> = decision.embedding
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Decision missing embedding"))?
            .iter()
            .map(|&f| f as f64)
            .collect();
        
        let vector = self.storage.vectors.insert::<fn(&_, &_) -> bool>(
            &mut txn,
            "decision",
            &embedding,
            Some(properties),
            &arena,
        )?;
        
        // Track mapping
        self.decision_id_map.insert(decision.metadata.id, vector.id);
        
        txn.commit()?;
        Ok(vector.id)
    }
    
    /// Create edges for decision relationships
    pub fn index_relationships(&mut self, decision: &Decision) -> Result<()> {
        let arena = bumpalo::Bump::new();
        let mut txn = self.storage.graph_env.write_txn()?;
        
        let from_node_id = self.decision_id_map
            .get(&decision.metadata.id)
            .ok_or_else(|| anyhow::anyhow!("Decision {} not indexed", decision.metadata.id))?;
        
        for rel in decision.metadata.relationships() {
            if let Some(&to_node_id) = self.decision_id_map.get(&rel.target_id) {
                // Create edge: from_decision --[RELATION]--> to_decision
                self.storage.add_edge(
                    &mut txn,
                    *from_node_id,
                    to_node_id,
                    rel.relation_type.as_edge_label(),
                    None,  // No edge properties
                    &arena,
                )?;
            }
            // Skip if target decision doesn't exist (might be external reference)
        }
        
        txn.commit()?;
        Ok(())
    }
    
    /// Search by vector similarity
    pub fn vector_search(&self, embedding: &[f32], limit: usize) -> Result<Vec<(u128, f32)>> {
        let arena = bumpalo::Bump::new();
        let txn = self.storage.graph_env.read_txn()?;
        
        let query: Vec<f64> = embedding.iter().map(|&f| f as f64).collect();
        
        let results = self.storage.vectors.search::<fn(&_, &_) -> bool>(
            &txn,
            &query,
            limit,
            "decision",
            None,
            false,
            &arena,
        )?;
        
        Ok(results
            .into_iter()
            .map(|v| (v.id, v.get_distance() as f32))
            .collect())
    }
    
    /// Traverse supersedes chain from a given decision
    pub fn get_supersedes_chain(&self, decision_id: u32) -> Result<Vec<ChainNode>> {
        let arena = bumpalo::Bump::new();
        let txn = self.storage.graph_env.read_txn()?;
        
        let mut chain = Vec::new();
        let mut current_id = self.decision_id_map.get(&decision_id).copied();
        
        while let Some(node_id) = current_id {
            let node = self.storage.get_node(&txn, &node_id, &arena)?;
            chain.push(self.node_to_chain_node(&node)?);
            
            // Follow SUPERSEDES edge (this decision supersedes which?)
            current_id = self.get_outgoing_edge(&txn, node_id, "SUPERSEDES", &arena)?;
        }
        
        // Mark the last one as current
        if let Some(last) = chain.last_mut() {
            last.is_current = true;
        }
        
        Ok(chain)
    }
    
    /// Find decisions related to a given decision (1-hop)
    pub fn get_related(&self, decision_id: u32) -> Result<Vec<(u128, RelationType)>> {
        let arena = bumpalo::Bump::new();
        let txn = self.storage.graph_env.read_txn()?;
        
        let node_id = self.decision_id_map
            .get(&decision_id)
            .ok_or_else(|| anyhow::anyhow!("Decision {} not found", decision_id))?;
        
        let mut related = Vec::new();
        
        // Check all relationship types (both directions)
        for rel_type in [
            RelationType::Supersedes,
            RelationType::Amends,
            RelationType::DependsOn,
            RelationType::RelatedTo,
        ] {
            // Outgoing
            for target in self.get_all_outgoing(&txn, *node_id, rel_type.as_edge_label(), &arena)? {
                related.push((target, rel_type));
            }
            // Incoming (for bidirectional relationships or reverse lookups)
            for source in self.get_all_incoming(&txn, *node_id, rel_type.as_edge_label(), &arena)? {
                related.push((source, rel_type));
            }
        }
        
        Ok(related)
    }
    
    /// Get stored content hashes for delta detection
    pub fn get_hashes(&self) -> Result<HashMap<String, String>> {
        let arena = bumpalo::Bump::new();
        let txn = self.storage.graph_env.read_txn()?;
        
        let mut hashes = HashMap::new();
        
        // Iterate all decision vectors and extract file_path -> content_hash
        let vectors = self.storage.vectors.get_all_vectors(&txn, None, &arena)?;
        
        for vector in vectors {
            if let Some(props) = &vector.properties {
                if let (Some(path), Some(hash)) = (
                    props.get("file_path").and_then(|v| v.as_str()),
                    props.get("content_hash").and_then(|v| v.as_str()),
                ) {
                    hashes.insert(path.to_string(), hash.to_string());
                }
            }
        }
        
        Ok(hashes)
    }
    
    /// Remove a decision and its edges
    pub fn remove_decision(&mut self, decision_id: u32) -> Result<()> {
        let arena = bumpalo::Bump::new();
        
        if let Some(node_id) = self.decision_id_map.remove(&decision_id) {
            let mut txn = self.storage.graph_env.write_txn()?;
            self.storage.drop_vector(&mut txn, &node_id)?;
            txn.commit()?;
        }
        
        Ok(())
    }
}
```

### delta.rs
```rust
use crate::types::Decision;
use std::collections::HashMap;

/// Delta detection result
pub struct DeltaResult {
    pub to_add: Vec<Decision>,
    pub to_remove: Vec<String>,
}

/// Compute delta between filesystem and indexed decisions
pub fn compute_delta(
    current_decisions: Vec<Decision>,
    stored_hashes: HashMap<String, String>,
) -> DeltaResult {
    let mut to_add = Vec::new();
    let mut to_remove = Vec::new();
    
    // Track which stored paths we've seen
    let mut seen_paths: std::collections::HashSet<String> = std::collections::HashSet::new();
    
    for decision in current_decisions {
        let path = decision.file_path.to_string_lossy().to_string();
        seen_paths.insert(path.clone());
        
        match stored_hashes.get(&path) {
            Some(stored_hash) if stored_hash == &decision.content_hash => {
                // No change, skip
            }
            _ => {
                // New or changed, need to re-index
                to_add.push(decision);
            }
        }
    }
    
    // Find deleted decisions
    for path in stored_hashes.keys() {
        if !seen_paths.contains(path) {
            to_remove.push(path.clone());
        }
    }
    
    DeltaResult { to_add, to_remove }
}
```

### searcher.rs
```rust
use crate::delta::compute_delta;
use crate::embeddings::Embedder;
use crate::loader::load_decisions;
use crate::storage::{DecisionStorage, HelixDBStorage};
use crate::types::{SearchResponse, SearchResult, Status};
use anyhow::Result;
use std::path::Path;

pub struct DecisionSearcher {
    storage: Box<dyn DecisionStorage>,
    embedder: Embedder,
}

impl DecisionSearcher {
    pub fn new() -> Result<Self> {
        let storage = Box::new(HelixDBStorage::open()?);
        let embedder = Embedder::new()?;
        Ok(Self { storage, embedder })
    }
    
    /// Load and sync decisions from directory
    pub fn sync(&mut self, dir: &Path) -> Result<()> {
        // Load current decisions
        let decisions = load_decisions(dir)?;
        
        // Get stored hashes
        let stored_hashes = self.storage.get_hashes()?;
        
        // Compute delta
        let delta = compute_delta(decisions, stored_hashes);
        
        // Remove deleted decisions
        if !delta.to_remove.is_empty() {
            self.storage.remove(delta.to_remove)?;
        }
        
        // Embed and index new/changed decisions
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
    
    /// Search for decisions matching query
    pub fn search(
        &self,
        query: &str,
        limit: usize,
        status_filter: Option<Status>,
        tags_filter: Option<Vec<String>>,
    ) -> Result<SearchResponse> {
        // Embed query
        let query_embedding = self.embedder.embed(query)?;
        
        // Search storage
        let results = self.storage.search(query_embedding, limit * 2)?;  // Over-fetch for filtering
        
        // Filter and convert
        let mut search_results: Vec<SearchResult> = results
            .into_iter()
            .filter(|(decision, _)| {
                // Status filter
                if let Some(ref status) = status_filter {
                    if &decision.metadata.status != status {
                        return false;
                    }
                }
                // Tags filter
                if let Some(ref tags) = tags_filter {
                    if !tags.iter().all(|t| decision.metadata.tags.contains(t)) {
                        return false;
                    }
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
            })
            .collect();
        
        search_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        Ok(SearchResponse {
            query: query.to_string(),
            count: search_results.len(),
            results: search_results,
        })
    }
}
```

## Data Flow

### First Invocation (Cold Start)
```
1. CLI parses args (query, options)
2. DecisionSearcher::new() opens HelixDB (creates if needed)
3. DecisionSearcher::sync(dir) loads all decisions from .decisions/
4. Delta check finds all decisions are new
5. Embed all decisions with fastembed (~2-5s)
6. Store decisions as vectors in HelixDB (with properties)
7. Create relationship edges from frontmatter
8. DecisionSearcher::search(query) embeds query
9. HelixDB vector search returns ranked results
10. Optionally enrich with graph context
11. CLI formats and outputs results
```

### Subsequent Invocations (Warm)
```
1. CLI parses args
2. DecisionSearcher::new() opens existing HelixDB
3. DecisionSearcher::sync(dir) loads current decisions
4. Delta check compares hashes to stored
5. Only re-embed changed decisions (usually 0)
6. Update edges for changed decisions
7. Remove deleted decisions and their edges
8. Search proceeds as normal (~100ms total)
```

### Graph Traversal (chain/related commands)
```
1. CLI parses args with decision ID
2. DecisionSearcher::new() opens existing HelixDB
3. No sync needed for read-only graph queries
4. Traverse edges from specified decision
5. Return connected decisions with relationship info
```

## Query Examples

### Semantic Search
```bash
# Find decisions about caching
helix-decisions search "caching strategy"

# With graph context (show related decisions)
helix-decisions search "caching strategy" --with-related

# Filter by status
helix-decisions search "database" --status accepted
```

### Graph Queries
```bash
# Show the evolution of a decision (supersedes chain)
helix-decisions chain 2
# Output: 002 → 005 → 008 (current)

# Find all decisions related to a specific one
helix-decisions related 5
# Output: 
#   supersedes: 002
#   amended_by: 007
#   related_to: 004, 006

# Find the current decision (follow supersedes to leaf)
helix-decisions chain 2
# Output: 008 (the leaf of the chain starting at 2)
```

### JSON Output (for agents)
```bash
helix-decisions search "authentication" --json
```
```json
{
  "query": "authentication",
  "count": 2,
  "results": [
    {
      "id": 4,
      "title": "JWT Authentication",
      "status": "accepted",
      "score": 0.89,
      "tags": ["auth", "security"],
      "date": "2026-01-03",
      "related": [
        {"id": 1, "title": "API Design", "relation": "depends_on"}
      ]
    }
  ]
}
```

## Storage Schema

### HelixDB Graph-Vector Structure

Decisions are stored as vectors with properties (combining node + vector storage):

```
Vector "decision" {
    id: u128                  // HelixDB internal ID
    label: "decision"         // Vector label for search
    embedding: [f64; 384]     // MiniLM-L6-v2 embedding
    
    properties: {
        decision_id: u32,      // Decision number (1, 2, 3...)
        uuid: String,          // Global hash-based ID
        title: String,
        status: String,        // "proposed"|"accepted"|"superseded"|"deprecated"
        date: String,          // ISO 8601
        deciders: String,      // JSON array as string
        tags: String,          // JSON array as string
        file_path: String,
        content_hash: String,
        git_commit: String,    // Commit hash for immutability
        body: String,          // Full markdown for display
    }
}

Edge "SUPERSEDES" {
    from: decision.id,
    to: decision.id,
}

Edge "AMENDS" {
    from: decision.id,
    to: decision.id,
}

Edge "DEPENDS_ON" {
    from: decision.id,
    to: decision.id,
}

Edge "RELATED_TO" {
    from: decision.id,
    to: decision.id,
    // Note: Store both directions for bidirectional lookup
}
```

### Index Location
```
~/.helix/data/decisions/
├── data.mdb         # LMDB data file
└── lock.mdb         # LMDB lock file
```

### Decision Frontmatter Format

```yaml
---
id: 5
uuid: hx-a1b2c3             # Optional: hash-based UUID for distributed safety
title: PostgreSQL for Primary Database
status: accepted
date: 2026-01-04
deciders:
  - Alice
  - Bob
tags:
  - database
  - infrastructure
content_hash: abc123...     # Optional: for immutability proof
git_commit: def456...       # Optional: commit when accepted

# Relationships (all optional, can be single ID or array)
supersedes: 2               # This decision replaces decision 2
amends: [3, 4]              # This decision modifies decisions 3 and 4
depends_on: 1               # This decision assumes decision 1 is accepted
related_to: [6, 7]          # Related but not dependent
---

# Context and Problem Statement
...
```

## Embedding Model

Using `fastembed` with `AllMiniLML6V2`:
- 384 dimensions
- ~50ms per embedding (CPU)
- Good semantic understanding
- Small model size (~90MB)

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| First sync | 2-5s | Embedding 100 decisions |
| Delta sync | < 50ms | Hash comparison |
| Query embed | 50-100ms | Single text |
| Vector search | < 50ms | HelixDB |
| Graph traversal | < 50ms | Chain/related |
| Total search | < 100ms | After first run |

## Error Handling

| Error | Behavior |
|-------|----------|
| Missing directory | Exit 2 with message |
| Malformed YAML | Warn, skip file |
| HelixDB error | Exit 2 with message |
| Embedding error | Exit 2 with message |
| No results | Exit 1, show "No results" |
