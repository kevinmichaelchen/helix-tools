# helix-embeddings

Shared embedding infrastructure for helix-tools using fastembed.

## Why

Multiple helix-tools need semantic embeddings:
- **helix-decisions** — Embed decisions for semantic search
- **hbd** — Embed issues for similarity search (planned)
- **helix-docs** — Embed documentation chunks

This crate provides a unified `Embedder` so each tool doesn't reinvent embedding logic.

## Usage

```rust
use helix_embeddings::Embedder;

// Create embedder (uses config from ~/.helix/config/config.toml)
let embedder = Embedder::new()?;

// Embed single text
let embedding = embedder.embed("How to handle authentication?")?;
assert_eq!(embedding.len(), 384); // Default model dimension

// Embed batch (more efficient)
let embeddings = embedder.embed_batch(&[
    "First document",
    "Second document",
])?;
```

## Configuration

Configure via `~/.helix/config/config.toml`:

```toml
[embedding]
model = "BAAI/bge-small-en-v1.5"  # Default model
batch_size = 32                    # Batch size for embed_batch
```

## Supported Models

Uses fastembed which supports:
- `BAAI/bge-small-en-v1.5` (default, 384 dimensions)
- `BAAI/bge-base-en-v1.5` (768 dimensions)
- `sentence-transformers/all-MiniLM-L6-v2` (384 dimensions)

See [fastembed docs](https://docs.rs/fastembed) for full model list.

## Consumers

| Crate | Use Case |
|-------|----------|
| helix-decisions | Semantic search over decisions |
| hbd | Semantic search over issues |
| helix-docs | Semantic search over documentation |

## License

MIT
