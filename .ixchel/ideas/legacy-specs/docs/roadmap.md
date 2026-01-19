# Implementation Roadmap

This document outlines the phased implementation plan for the unified Ixchel
knowledge system.

Note: this roadmap is a historical blueprint. The implemented system is now
**Ixchel** (crates `ix-*`, canonical dir `.ixchel/`). For the current backlog,
prefer each crate’s `specs/tasks.md`.

## Current State

### Existing Assets

| Crate              | Status      | Capabilities                                                 |
| ------------------ | ----------- | ------------------------------------------------------------ |
| `hbd`              | ✅ Complete | Git-backed issue tracker, dependency graphs, cycle detection |
| `ixchel-decisions` | Removed     | Replaced by `.ixchel/decisions/` (git-first Markdown)        |
| `ix-embeddings`    | ✅ Complete | Pluggable embedding providers (fastembed, candle)            |
| `helix-db`         | ✅ Complete | Graph-vector database with HNSW                              |
| `ix-config`        | ✅ Complete | Configuration management                                     |

### What We Can Reuse

From `hbd`:

- Markdown + YAML frontmatter parsing
- ID generation (BLAKE3 hash-based)
- Dependency graph algorithms (cycle detection, blocker trees)
- CLI structure (clap)
- File-based storage patterns

From `ixchel-decisions`:

- Semantic search implementation
- Delta detection for incremental sync
- Manifest-based change tracking
- Git hook installation
- HelixDB integration patterns

---

## Phase 0: Foundation (Week 1)

### Goal

Set up the `ixchel-core` crate with the unified entity abstraction.

### Tasks

#### 0.1 Create Crate Structure

```
ixchel/
├── Cargo.toml
└── src/
    └── main.rs          # Placeholder

ixchel-core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── entity/
    │   └── mod.rs       # Entity trait
    └── types.rs         # Common types
```

#### 0.2 Define Entity Trait

```rust
pub trait Entity: Sized + Send + Sync {
    const PREFIX: &'static str;
    const DIRECTORY: &'static str;

    fn id(&self) -> &str;
    fn title(&self) -> &str;
    fn metadata(&self) -> &EntityMetadata;

    fn to_markdown(&self) -> String;
    fn from_markdown(content: &str, path: PathBuf) -> Result<Self>;

    fn embedding_text(&self) -> String;
    fn validate(&self) -> Result<()>;
}
```

#### 0.3 Define Common Types

```rust
pub struct EntityMetadata { ... }
pub struct Relationship { ... }
pub enum RelationType { ... }
pub enum EntityType { Decision, Issue, Idea, Report, Source, Citation }
```

#### 0.4 Port Decision Entity

Copy and adapt from `ixchel-decisions`:

- `Decision` struct
- `DecisionMetadata`
- `DecisionStatus`
- Markdown parsing/serialization

### Deliverable

`ixchel-core` crate that can represent Decision entities.

---

## Phase 1: Entity Types (Week 2)

### Goal

Implement all six entity types with full markdown support.

### Tasks

#### 1.1 Issue Entity

Port from `hbd`:

- `Issue` struct with all fields
- Status, priority, type enums
- Comment support
- Parent/child relationships

#### 1.2 Idea Entity

New implementation:

- `Idea` struct
- Status lifecycle (draft → proposed → evolved)
- Effort/impact fields

#### 1.3 Report Entity

New implementation:

- `Report` struct
- Report types (postmortem, rfc, retrospective)
- Period fields for retrospectives

#### 1.4 Source Entity

New implementation:

- `Source` struct
- Source types (paper, article, etc.)
- Bibliographic fields (DOI, ISBN, authors)

#### 1.5 Citation Entity

New implementation:

- `Citation` struct
- Quote storage
- Page/timestamp references

#### 1.6 Shared Markdown Module

Unified parsing:

```rust
pub fn parse_entity<E: Entity>(content: &str, path: PathBuf) -> Result<E>;
pub fn serialize_entity<E: Entity>(entity: &E) -> String;
```

### Deliverable

All six entity types implemented with markdown round-tripping.

---

## Phase 2: Storage Layer (Week 3)

### Goal

Implement unified file and graph storage.

### Tasks

#### 2.1 File Storage

Unified file operations:

```rust
pub struct FileStorage {
    root: PathBuf,
}

impl FileStorage {
    pub fn read<E: Entity>(&self, id: &str) -> Result<E>;
    pub fn write<E: Entity>(&self, entity: &E) -> Result<()>;
    pub fn delete<E: Entity>(&self, id: &str) -> Result<()>;
    pub fn list<E: Entity>(&self) -> Result<Vec<E>>;
}
```

#### 2.2 Graph Storage

HelixDB integration:

```rust
pub struct GraphStorage {
    db: HelixGraphStorage,
}

impl GraphStorage {
    pub fn upsert_node<E: Entity>(&self, entity: &E) -> Result<NodeId>;
    pub fn add_edge(&self, from: &str, to: &str, rel: RelationType) -> Result<()>;
    pub fn query_outgoing(&self, id: &str) -> Result<Vec<(RelationType, NodeId)>>;
    pub fn query_incoming(&self, id: &str) -> Result<Vec<(RelationType, NodeId)>>;
}
```

#### 2.3 Vector Storage

Embedding integration:

```rust
pub struct VectorStorage {
    embedder: Embedder,
    hnsw: HNSW,
}

impl VectorStorage {
    pub fn index<E: Entity>(&self, entity: &E) -> Result<VectorId>;
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<(EntityId, f32)>>;
}
```

#### 2.4 Unified Storage

Coordinate all three:

```rust
pub struct UnifiedStorage {
    files: FileStorage,
    graph: GraphStorage,
    vectors: VectorStorage,
}

impl StorageBackend for UnifiedStorage {
    async fn create<E: Entity>(&self, entity: &E) -> Result<()> {
        self.files.write(entity)?;
        let node_id = self.graph.upsert_node(entity)?;
        let vector_id = self.vectors.index(entity)?;
        // Link vector to node
        Ok(())
    }
}
```

### Deliverable

Working storage layer that maintains file ↔ graph ↔ vector consistency.

---

## Phase 3: CLI (Week 4)

### Goal

Implement the unified `ixchel` CLI.

### Tasks

#### 3.1 CLI Skeleton

```rust
#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long)]
    json: bool,

    #[arg(long)]
    agent: Option<String>,

    #[arg(long)]
    session: Option<String>,
}
```

#### 3.2 Create Commands

```bash
ixchel create decision "Title"
ixchel create issue "Title"
# ... all entity types
```

#### 3.3 CRUD Commands

```bash
ixchel show <id>
ixchel list <type>
ixchel update <id>
ixchel delete <id>
```

#### 3.4 Search Command

```bash
ixchel search "query" --types decision,issue --limit 10
```

#### 3.5 Graph Command

```bash
ixchel graph <id> --depth 3 --format tree
```

#### 3.6 Link/Unlink Commands

```bash
ixchel link dec-42 spawns iss-17
ixchel unlink dec-42 spawns iss-17
```

#### 3.7 Context Command

```bash
ixchel context <id> --depth 2 --format markdown
```

#### 3.8 Maintenance Commands

```bash
ixchel init
ixchel sync
ixchel check
ixchel health
```

### Deliverable

Fully functional CLI with all commands.

---

## Phase 4: Sync Engine (Week 5)

### Goal

Implement incremental synchronization between files and database.

### Tasks

#### 4.1 Manifest

Track indexed state:

```rust
pub struct Manifest {
    pub version: u32,
    pub entries: HashMap<String, ManifestEntry>,
}

pub struct ManifestEntry {
    pub entity_id: String,
    pub file_path: PathBuf,
    pub mtime: SystemTime,
    pub size: u64,
    pub content_hash: String,
    pub node_id: NodeId,
    pub vector_id: VectorId,
}
```

#### 4.2 Delta Detection

Port from `ixchel-decisions`:

```rust
pub struct Delta {
    pub to_add: Vec<PathBuf>,
    pub to_modify: Vec<PathBuf>,
    pub to_remove: Vec<String>,
    pub renamed: Vec<(String, PathBuf)>,
}

pub fn detect_delta(files: &[FileInfo], manifest: &Manifest) -> Delta;
```

#### 4.3 Sync Execution

```rust
pub async fn sync(storage: &UnifiedStorage) -> Result<SyncReport> {
    let delta = detect_delta(...)?;

    for path in delta.to_add {
        let entity = parse_file(&path)?;
        storage.create(&entity)?;
    }

    for path in delta.to_modify {
        let entity = parse_file(&path)?;
        storage.update(&entity)?;
    }

    for id in delta.to_remove {
        storage.delete(&id)?;
    }

    Ok(SyncReport { ... })
}
```

#### 4.4 File Watcher (Optional Daemon)

```rust
pub async fn watch(storage: &UnifiedStorage) -> Result<()> {
    let (tx, rx) = mpsc::channel();
    let watcher = notify::recommended_watcher(tx)?;
    watcher.watch(".ixchel/", RecursiveMode::Recursive)?;

    while let Ok(event) = rx.recv() {
        if let Some(delta) = process_event(event) {
            sync_delta(storage, delta)?;
        }
    }
}
```

### Deliverable

Incremental sync that only re-processes changed files.

---

## Phase 5: Search & Graph Algorithms (Week 6)

### Goal

Implement semantic search and graph traversal algorithms.

### Tasks

#### 5.1 Semantic Search

```rust
pub struct SearchQuery {
    pub text: String,
    pub entity_types: Vec<EntityType>,
    pub filters: Filters,
    pub limit: usize,
}

pub fn search(storage: &UnifiedStorage, query: &SearchQuery) -> Result<Vec<SearchResult>> {
    // 1. Embed query
    let embedding = storage.vectors.embed(&query.text)?;

    // 2. Vector search (over-fetch for filtering)
    let candidates = storage.vectors.search(&embedding, query.limit * 3)?;

    // 3. Resolve to entities
    let entities = candidates
        .iter()
        .filter_map(|(id, score)| {
            let entity = storage.read_any(id).ok()?;
            if query.filters.matches(&entity) && query.entity_types.contains(&entity.type_()) {
                Some(SearchResult { entity, score: *score })
            } else {
                None
            }
        })
        .take(query.limit)
        .collect();

    Ok(entities)
}
```

#### 5.2 Graph Traversal

```rust
pub fn traverse(
    storage: &UnifiedStorage,
    start_id: &str,
    query: &TraverseQuery,
) -> Result<Graph> {
    let mut graph = Graph::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    queue.push_back((start_id.to_string(), 0));

    while let Some((id, depth)) = queue.pop_front() {
        if depth > query.max_depth || !visited.insert(id.clone()) {
            continue;
        }

        let node = storage.read_any(&id)?;
        graph.add_node(node);

        let edges = match query.direction {
            Direction::Outgoing => storage.graph.outgoing(&id)?,
            Direction::Incoming => storage.graph.incoming(&id)?,
            Direction::Both => {
                let mut e = storage.graph.outgoing(&id)?;
                e.extend(storage.graph.incoming(&id)?);
                e
            }
        };

        for (rel_type, target_id) in edges {
            if query.relationship_types.is_empty() || query.relationship_types.contains(&rel_type) {
                graph.add_edge(&id, &target_id, rel_type);
                queue.push_back((target_id, depth + 1));
            }
        }
    }

    Ok(graph)
}
```

#### 5.3 Cycle Detection

Port from `hbd`:

```rust
pub fn find_cycles(storage: &UnifiedStorage, edge_type: RelationType) -> Vec<Vec<String>> {
    // Tarjan's SCC algorithm
    ...
}

pub fn would_create_cycle(storage: &UnifiedStorage, from: &str, to: &str) -> bool {
    // BFS from `to` to see if we can reach `from`
    ...
}
```

#### 5.4 Critical Path

```rust
pub fn find_critical_path(storage: &UnifiedStorage, target_id: &str) -> Vec<String> {
    // DFS to find longest blocking chain
    ...
}
```

### Deliverable

Full search and graph query capabilities.

---

## Phase 6: Context Generation (Week 7)

### Goal

Implement AI context generation.

### Tasks

#### 6.1 Context Builder

```rust
pub struct ContextBuilder {
    storage: Arc<UnifiedStorage>,
    max_tokens: usize,
    format: ContextFormat,
}

impl ContextBuilder {
    pub fn build(&self, id: &str, depth: usize) -> Result<String> {
        let entity = self.storage.read_any(id)?;
        let graph = self.storage.traverse(id, depth)?;

        match self.format {
            ContextFormat::Markdown => self.render_markdown(&entity, &graph),
            ContextFormat::Xml => self.render_xml(&entity, &graph),
            ContextFormat::Json => self.render_json(&entity, &graph),
        }
    }
}
```

#### 6.2 Token Budgeting

```rust
fn budget_tokens(&self, entity: &Entity, graph: &Graph) -> TokenBudget {
    let total = self.max_tokens;
    let entity_tokens = estimate_tokens(&entity.body()) + 200; // metadata

    let remaining = total - entity_tokens;
    let related_count = graph.nodes.len() - 1;

    if related_count == 0 {
        return TokenBudget { entity: entity_tokens, per_related: 0 };
    }

    TokenBudget {
        entity: entity_tokens,
        per_related: remaining / related_count,
    }
}
```

#### 6.3 Markdown Rendering

```rust
fn render_markdown(&self, entity: &Entity, graph: &Graph) -> String {
    let mut out = String::new();

    // Header
    writeln!(out, "# Context for {}: {}", entity.id(), entity.title());
    writeln!(out);

    // Main entity
    writeln!(out, "## This {}", entity.type_name());
    writeln!(out, "{}", entity.body());
    writeln!(out);

    // Group related by relationship type
    for (rel_type, related) in graph.group_by_relationship() {
        writeln!(out, "## {}", rel_type.display_name());
        for node in related {
            writeln!(out, "### {}: {}", node.id(), node.title());
            writeln!(out, "{}", truncate(&node.body(), self.per_related_tokens));
        }
    }

    out
}
```

### Deliverable

Working `ixchel context` command with multiple output formats.

---

## Phase 7: Hooks & Validation (Week 8)

### Goal

Implement git hooks and entity validation.

### Tasks

#### 7.1 Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

ixchel check --staged
if [ $? -ne 0 ]; then
    echo "Ixchel validation failed. Commit aborted."
    exit 1
fi
```

#### 7.2 Immutability Enforcement

```rust
pub fn validate_immutability(old: &Decision, new: &Decision) -> Result<()> {
    if old.status == DecisionStatus::Accepted {
        if old.title != new.title {
            return Err(Error::ImmutableField("title"));
        }
        if old.body != new.body {
            return Err(Error::ImmutableField("body"));
        }
        // Only status can change to superseded/deprecated
        match new.status {
            DecisionStatus::Superseded | DecisionStatus::Deprecated => Ok(()),
            _ if new.status == old.status => Ok(()),
            _ => Err(Error::InvalidStatusTransition),
        }
    } else {
        Ok(())
    }
}
```

#### 7.3 Relationship Validation

```rust
pub fn validate_relationship(from: &Entity, to: &Entity, rel: RelationType) -> Result<()> {
    // Check validity matrix
    if !VALIDITY_MATRIX.allows(from.type_(), to.type_(), rel) {
        return Err(Error::InvalidRelationship { ... });
    }

    // Check for self-loops
    if from.id() == to.id() {
        return Err(Error::SelfLoop);
    }

    // Check for cycles (blocking relationships only)
    if rel.is_blocking() && would_create_cycle(from.id(), to.id()) {
        return Err(Error::CycleDetected { ... });
    }

    Ok(())
}
```

#### 7.4 Health Checks

```rust
pub fn health_report(storage: &UnifiedStorage) -> HealthReport {
    HealthReport {
        entity_counts: count_by_type_and_status(storage),
        stale_entities: find_stale(storage, Duration::days(90)),
        orphaned_entities: find_orphans(storage),
        decisions_without_sources: find_unsourced_decisions(storage),
        blocked_too_long: find_long_blocked(storage, Duration::days(60)),
        cycles: find_all_cycles(storage),
    }
}
```

### Deliverable

Robust validation and git hook integration.

---

## Phase 8: TUI (Week 9-10)

### Goal

Implement the terminal user interface.

### Tasks

#### 8.1 App Structure

```rust
pub struct App {
    view: View,
    storage: Arc<UnifiedStorage>,
    state: AppState,
}

pub enum View {
    Browser,
    Detail(String),
    Search,
    Graph(String),
    Dashboard,
}
```

#### 8.2 Browser View

- Entity list with type filtering
- Status/priority filters
- Sorting options
- Keyboard navigation (vim-style)

#### 8.3 Detail View

- Full entity display
- Relationship sidebar
- Quick actions (edit, link, graph)

#### 8.4 Search View

- Search input
- Real-time results
- Filter toggles

#### 8.5 Graph View

- ASCII graph rendering
- Pan/zoom with keyboard
- Depth control

#### 8.6 Dashboard View

- Entity counts
- Health warnings
- Recent activity

### Deliverable

Fully functional TUI with all views.

---

## Phase 9: MCP Server (Week 11)

### Goal

Implement MCP server for AI tool integration.

### Tasks

#### 9.1 Server Setup

```rust
#[tokio::main]
async fn main() {
    let storage = UnifiedStorage::open(".ixchel")?;
    let server = McpServer::new(storage);
    server.serve().await?;
}
```

#### 9.2 Tool Implementations

- `ixchel_search` — Semantic search
- `ixchel_show` — Entity details
- `ixchel_list` — List with filters
- `ixchel_graph` — Relationship traversal
- `ixchel_create` — Create entity
- `ixchel_link` — Add relationship
- `ixchel_context` — Generate context

#### 9.3 Claude Code Integration

Configuration for MCP settings.

### Deliverable

Working MCP server that Claude Code can use.

---

## Phase 10: Migration & Polish (Week 12)

### Goal

Migration tools and final polish.

### Tasks

#### 10.1 hbd Migration

```bash
ixchel migrate hbd --source .tickets --target .ixchel/issues
```

- Convert `.tickets/*.md` to `.ixchel/issues/*.md`
- Preserve IDs (or remap with prefix)
- Convert relationships

#### 10.2 ixchel-decisions Migration

```bash
ixchel migrate ixchel-decisions --source .decisions --target .ixchel/decisions
```

- Convert `.decisions/*.md` to `.ixchel/decisions/*.md`
- Preserve IDs
- Update relationship references

#### 10.3 Documentation

- Complete README
- CLI help text
- Man pages (optional)
- Examples repository

#### 10.4 Testing

- Unit tests for all entities
- Integration tests for storage
- CLI integration tests
- TUI snapshot tests

#### 10.5 Release

- Cargo publish `ixchel-core`
- Cargo publish `ixchel-cli`
- GitHub releases
- Homebrew formula

### Deliverable

Production-ready v1.0 release.

---

## Timeline Summary

| Phase | Week | Focus                     |
| ----- | ---- | ------------------------- |
| 0     | 1    | Foundation & Entity trait |
| 1     | 2    | All entity types          |
| 2     | 3    | Storage layer             |
| 3     | 4    | CLI commands              |
| 4     | 5    | Sync engine               |
| 5     | 6    | Search & graph algorithms |
| 6     | 7    | Context generation        |
| 7     | 8    | Hooks & validation        |
| 8     | 9-10 | TUI                       |
| 9     | 11   | MCP server                |
| 10    | 12   | Migration & polish        |

**Total: 12 weeks to v1.0**

---

## Post-v1.0 Roadmap

### v1.1: Enhanced Search

- Hybrid search (vector + keyword)
- Faceted search
- Search within graph subsets

### v1.2: Collaboration

- Conflict detection on merge
- Branch comparison views
- PR integration (show changed entities)

### v1.3: Automation

- Scheduled health reports
- Auto-linking suggestions
- Stale entity notifications

### v1.4: Plugins

- Custom entity types via TOML
- Custom relationship types
- Hook system for extensions

### v2.0: Multi-Repo

- Cross-repository references
- Federated search
- Shared source libraries
