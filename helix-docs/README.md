# helix-docs

Global documentation cache for AI-assisted development. Fetch, cache, and semantically index documentation from GitHub repos—shared across all your projects.

## Status

**Phase 0 - Project Foundation** (scaffolded)

The core architecture, traits, and CLI structure are defined. Implementation is pending.

## Quick Start (Planned)

```bash
# Add a documentation source (cached globally)
helix-docs add https://github.com/facebook/react --docs docs

# Fetch and index documentation
helix-docs ingest --embed

# Search with hybrid BM25 + semantic
helix-docs search --library facebook/react "hooks state management"

# Get specific document content
helix-docs get --library facebook/react docs/hooks.md
```

## Features (Planned)

- **Global Cache** - Docs cached once, shared across all projects
- **GitHub Integration** - Fetch docs from any public or private repo
- **Semantic Search** - Find docs by meaning, not just keywords
- **Hybrid Search** - BM25 + vector search with RRF fusion
- **Dependency Detection** - Auto-detect project dependencies from manifests
- **MCP Server** - Expose as tools for AI coding assistants
- **Pluggable Storage** - HelixDB backend with trait-based abstraction

## Why Global Cache?

Documentation like `facebook/react` is the same regardless of which project uses it. Global caching means:
- **Fetch once** - No redundant API calls across projects
- **Store once** - No duplicate storage per project
- **Share embeddings** - Vector embeddings computed once, reused everywhere

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    CLI Layer (helix-docs)               │
└────────────────────────────┬────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────┐
│                  Application Services                    │
│  SourceService | IngestionService | SearchService       │
└────────────────────────────┬────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────┐
│                      Domain Layer                        │
│  Source | Document | Chunk | SearchResult | Library     │
└────────────────────────────┬────────────────────────────┘
                             │
┌────────────────────────────┴────────────────────────────┐
│                   Port Traits (Interfaces)               │
│  SourceRepository | DocumentRepository | SearchIndex    │
│  FetchClient | EmbeddingGenerator                       │
└────────────────────────────┬────────────────────────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
┌───────┴───────┐  ┌─────────┴─────────┐  ┌──────┴───────┐
│ HelixDB       │  │ GitHub Client     │  │ Fastembed    │
│ Adapter       │  │                   │  │ Adapter      │
└───────────────┘  └───────────────────┘  └──────────────┘
```

## Storage

```
~/.helix/
├── config/
│   ├── config.toml           # Shared config (GitHub token, etc.)
│   └── helix-docs.toml       # Global helix-docs settings
│
└── data/
    └── docs/                 # Global documentation cache
        ├── helix-docs.db/    # HelixDB data
        └── schema.hx
```

## Configuration

### Global (`~/.helix/config/config.toml`)

```toml
[github]
token = "ghp_xxx"  # Or use GITHUB_TOKEN env var
```

### Global helix-docs (`~/.helix/config/helix-docs.toml`)

```toml
[ingest]
concurrency = 5
extensions = ["md", "mdx", "txt", "rst"]

[search]
default_mode = "hybrid"
default_limit = 10

[freshness]
stale_days = 7
use_etag = true
```

### Project (`.helix/helix-docs.toml`) - Optional

```toml
[search]
# Limit searches to specific libraries for this project
preferred_libraries = ["facebook/react", "vercel/next.js"]
```

## CLI Reference

| Command | Description |
|---------|-------------|
| `helix-docs add <url>` | Add a documentation source |
| `helix-docs source list` | List configured sources |
| `helix-docs source remove <id>` | Remove a source |
| `helix-docs ingest` | Fetch and index all sources |
| `helix-docs ingest --embed` | Include vector embeddings |
| `helix-docs search <query>` | Search documentation |
| `helix-docs library <name>` | Find libraries by name |
| `helix-docs get <path>` | Retrieve document content |
| `helix-docs status` | Show cache statistics |
| `helix-docs detect` | Detect project dependencies |
| `helix-docs mcp` | Start MCP server |

## Specifications

- [specs/requirements.md][requirements] - User stories and acceptance criteria (EARS notation)
- [specs/design.md][design] - Architecture, traits, and data model
- [specs/tasks.md][tasks] - Implementation roadmap with 7 phases

## Inspiration

Inspired by [librarian][librarian] - a TypeScript-based documentation tool. helix-docs reimplements the concept in Rust with HelixDB as the native backend.

## License

MIT

<!-- Links -->
[requirements]: ./specs/requirements.md
[design]: ./specs/design.md
[tasks]: ./specs/tasks.md
[librarian]: https://github.com/iannuttall/librarian
