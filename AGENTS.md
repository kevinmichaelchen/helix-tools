# HELIX-TOOLS KNOWLEDGE BASE

**Generated:** 2026-01-03  
**Commit:** 5560198  
**Branch:** main

## OVERVIEW

Rust monorepo of AI-native developer tools powered by HelixDB. Primary tool is `hbd` - a git-first issue tracker storing issues as Markdown in `.tickets/`. Includes a Fumadocs documentation site.

## STRUCTURE

```
helix-tools/
├── hbd/                    # Issue tracker CLI (Rust binary)
│   ├── src/                # 8 modules: main, lib, types, storage, markdown, id, error, db
│   └── specs/              # Kiro-style specs (requirements.md, design.md, tasks.md)
├── docs/                   # Next.js Fumadocs site (see docs/AGENTS.md)
├── .tickets/               # Issue storage (created by hbd init)
├── .helix/                 # hbd config (config.toml)
└── .github/workflows/      # CI (Rust) + docs deployment
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| Add CLI command | `hbd/src/main.rs` | Clap derive, Commands enum at line 19 |
| Add issue field | `hbd/src/types.rs` | Issue struct, update markdown.rs for serialization |
| Modify Markdown format | `hbd/src/markdown.rs` | YAML frontmatter + gray_matter |
| Change ID format | `hbd/src/id.rs` | Blake3 hash of UUID, 6-char prefix |
| File I/O operations | `hbd/src/storage.rs` | TicketStore, .tickets/ management |
| HelixDB integration | `hbd/src/db.rs` | ALL METHODS ARE TODO! Placeholder only |
| Kiro specifications | `hbd/specs/` | requirements.md, design.md, tasks.md |
| Documentation content | `docs/content/docs/` | MDX files with YAML frontmatter |

## CODE MAP (hbd)

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `Cli` | Struct | main.rs:7 | Root CLI args (--json flag) |
| `Commands` | Enum | main.rs:19 | 25+ subcommands |
| `Issue` | Struct | types.rs | Core data type with all fields |
| `Status` | Enum | types.rs | open, in_progress, blocked, closed |
| `Priority` | Enum | types.rs | 0-4 (critical to backlog) |
| `IssueType` | Enum | types.rs | bug, feature, task, epic, chore |
| `Dependency` | Struct | types.rs | Blocking relationships |
| `TicketStore` | Struct | storage.rs | File-based persistence |
| `generate_id()` | Function | id.rs | Blake3(UUID) → bd-xxxxxx |
| `HbdError` | Enum | error.rs | Structured errors with exit codes |

## CONVENTIONS

### Rust
- **Edition 2024** (future edition)
- **Strict clippy**: correctness=deny, pedantic/style/perf=warn
- **Formatting**: 100 char width, Unix newlines (rustfmt.toml)
- **Error handling**: thiserror for types, anyhow for propagation
- **Async**: tokio runtime (full features)
- **Git ops**: gix (pure Rust, not libgit2)

### Project-Specific
- **Issue IDs**: `bd-` prefix + 6 hex chars from Blake3 hash
- **Storage**: Markdown files with YAML frontmatter in `.tickets/`
- **Status values**: open, in_progress, blocked, closed (spec has 6, impl has 4)
- **Dependency types**: blocks, related, waits_for (spec has 4, impl has 3)
- **Agent tracking**: `--agent` and `--session` flags on mutating commands

## ANTI-PATTERNS

| Pattern | Why Forbidden |
|---------|---------------|
| Direct HelixDB calls | Use `db.rs` methods (currently todo!) |
| New status/type values | Keep aligned with implementation notes in specs |
| Cloud API calls | Offline-only by design (config.offline_only=true) |
| Skipping content_hash | Required for sync conflict detection |
| Modifying .tickets/ without TicketStore | Breaks consistency guarantees |

## UNIQUE STYLES

- **Hash-based IDs**: Prevent conflicts across branches without coordination
- **Markdown as source of truth**: HelixDB is query cache, not primary storage
- **Dual architecture docs**: design.md shows both "Target" and "Current" architecture
- **Kiro specs**: EARS notation for requirements, sequence diagrams for design

## COMMANDS

```bash
# Development
cargo build --all-features
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check

# Run hbd
cargo run -p hbd -- --help
cargo run -p hbd -- init
cargo run -p hbd -- create "Title" --type bug

# Documentation
cd docs && bun install && bun run dev   # Dev server on :3000
cd docs && bun run build                # Static export
```

## DEPENDENCIES (Key)

| Crate | Purpose |
|-------|---------|
| clap | CLI parsing with derive |
| serde + serde_yaml | Frontmatter serialization |
| gray_matter | YAML frontmatter extraction |
| pulldown-cmark | Markdown parsing/rendering |
| blake3 + uuid | Hash-based ID generation |
| gix | Git operations (pure Rust) |
| chrono | Timestamps |
| fastembed | Local embeddings (planned) |

## NOTES

- **HelixDB integration incomplete**: All `db.rs` methods are `todo!()` - Phase 4/5 planned
- **Semantic search not implemented**: fastembed dependency exists but unused
- **No daemon mode yet**: Direct file access only, no file watching
- **Beads inspiration**: Similar concepts (hash IDs, deps, agents) but Markdown storage
- **Reference repos**:
  - HelixDB: `/Users/kevinchen/dev/github.com/HelixDB/helix-db`
  - Beads: `/Users/kevinchen/dev/github.com/steveyegge/beads`
