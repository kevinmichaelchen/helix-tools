# Helix Tools

A monorepo of AI-native developer tools powered by
[HelixDB](https://github.com/HelixDB/helix-db) - the graph-vector database built
for RAG and AI applications.

## Vision

Modern AI-assisted development needs persistent, structured memory. These tools
provide that memory layer using HelixDB's unique combination of:

- **Graph traversal** - Navigate relationships between entities
- **Vector search** - Find semantically similar content
- **BM25 text search** - Traditional keyword matching
- **Hybrid reranking** - Combine search methods intelligently

## Tools

| Tool              | Description                                          | Status            |
| ----------------- | ---------------------------------------------------- | ----------------- |
| **[hbd](./hbd/)** | Git-first issue tracker for AI-supervised workflows  | 🚧 In Development |
| **helix-docs**    | Local cache for fetched docs during agentic research | 📋 Planned        |
| **helix-map**     | Codebase structure cache for fast exploration        | 📋 Planned        |
| **helix-mail**    | Agent-to-agent messaging and coordination            | 📋 Planned        |

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLI Tools Layer                              │
│                                                                      │
│   hbd          helix-docs      helix-map       helix-mail           │
│   (issues)     (research)      (codebase)      (messaging)          │
│                                                                      │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
┌──────────────────────────────┴──────────────────────────────────────┐
│                        Shared Libraries                              │
│                                                                      │
│   helix-embed          helix-sync           helix-common            │
│   (fastembed)          (git ops)            (types, utils)          │
│                                                                      │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
┌──────────────────────────────┴──────────────────────────────────────┐
│                           HelixDB                                    │
│                                                                      │
│   Graph Engine    Vector Search    BM25 Index    LMDB Storage       │
│   (traversals)    (HNSW)           (text)        (persistence)      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Design Principles

### 1. Git-First

All tools store their data in git-friendly formats (Markdown, YAML, JSONL).
HelixDB acts as a fast query cache, not the source of truth.

### 2. Offline-First

Full functionality without network access. Local embeddings via `fastembed` (no
Ollama server required). Cloud APIs are optional fallbacks.

### 3. AI-Native

Every tool is designed for AI agent consumption:

- `--json` output on all commands
- Semantic search for context retrieval
- Structured data for LLM prompts
- Agent session tracking

### 4. UNIX Philosophy

Each tool does one thing well. They compose via standard interfaces
(stdin/stdout, files, git).

## Getting Started

### Prerequisites

- Rust 1.75+
- Git 2.0+
- HelixDB CLI (`helix`)

### Installation

```bash
# Clone the repo
git clone https://github.com/kevinmichaelchen/helix-tools.git
cd helix-tools

# Build all tools
cargo build --release

# Install to PATH
cargo install --path hbd
```

### Quick Start with hbd

```bash
# Initialize in your project
cd your-project
hbd init

# Create an issue
hbd create "Add user authentication" \
  --description "Implement JWT-based auth flow" \
  --type feature \
  --priority 1

# Find similar issues
hbd similar bd-a1b2

# Search with keywords + semantics
hbd search "authentication bug" --hybrid

# Check what's ready to work on
hbd ready
```

## Project Structure

```
helix-tools/
├── hbd/                    # Issue tracker
│   ├── specs/              # Kiro-style specifications
│   │   ├── requirements.md
│   │   ├── design.md
│   │   └── tasks.md
│   └── src/
│
├── shared/                 # Shared Rust crates
│   ├── helix-embed/        # Embedding utilities
│   └── helix-sync/         # Git sync utilities
│
├── Cargo.toml              # Workspace root
└── README.md
```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](./LICENSE) for details.

## Acknowledgments

- [HelixDB](https://github.com/HelixDB/helix-db) - The graph-vector database
  powering these tools
- [Beads](https://github.com/steveyegge/beads) - Inspiration for git-backed
  issue tracking
- [fastembed](https://github.com/Anush008/fastembed-rs) - Native Rust embeddings
