# helix-embeddings

Shared embedding infrastructure for helix-tools with pluggable providers.

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
assert_eq!(embedding.len(), embedder.dimension());

// Embed batch (more efficient)
let embeddings = embedder.embed_batch(&[
    "First document",
    "Second document",
])?;
```

## Providers

Enable providers via Cargo features:

| Provider      | Feature               | GPU Support | Notes                                 |
| ------------- | --------------------- | ----------- | ------------------------------------- |
| **fastembed** | `fastembed` (default) | CPU only    | ONNX Runtime, fast startup            |
| **candle**    | `candle`              | Metal/CUDA  | Hugging Face models, GPU acceleration |

### Feature Flags

```toml
# Default (fastembed only)
helix-embeddings = { path = "..." }

# Candle with Metal (macOS)
helix-embeddings = { path = "...", features = ["candle", "metal"] }

# Candle with CUDA (Linux/Windows)
helix-embeddings = { path = "...", features = ["candle", "cuda"] }

# Both providers
helix-embeddings = { path = "...", features = ["fastembed", "candle"] }
```

## Configuration

Configure via `~/.helix/config/config.toml`:

```toml
[embedding]
provider = "fastembed"             # or "candle"
model = "BAAI/bge-small-en-v1.5"   # HuggingFace model ID
batch_size = 32                    # Batch size for embed_batch
dimension = 384                    # Optional (auto-detected)
```

## Supported Models

### FastEmbed (ONNX)

- `BAAI/bge-small-en-v1.5` (default, 384 dimensions)
- `BAAI/bge-base-en-v1.5` (768 dimensions)
- `sentence-transformers/all-MiniLM-L6-v2` (384 dimensions)

See [fastembed docs](https://docs.rs/fastembed) for full model list.

### Candle

Any BERT-based model from Hugging Face Hub:

| Model                                    | Params | Dims | Notes                                   |
| ---------------------------------------- | ------ | ---- | --------------------------------------- |
| `sentence-transformers/all-MiniLM-L6-v2` | 22M    | 384  | Fastest, good quality                   |
| `BAAI/bge-small-en-v1.5`                 | 33M    | 384  | Better retrieval                        |
| `BAAI/bge-base-en-v1.5`                  | 109M   | 768  | Higher quality                          |
| `BAAI/bge-large-en-v1.5`                 | 335M   | 1024 | **Best quality** (recommended with GPU) |

For highest quality with Metal GPU acceleration:

```toml
[embedding]
provider = "candle"
model = "BAAI/bge-large-en-v1.5"
```

## Consumers

| Crate           | Use Case                           |
| --------------- | ---------------------------------- |
| helix-decisions | Semantic search over decisions     |
| hbd             | Semantic search over issues        |
| helix-docs      | Semantic search over documentation |

## License

MIT
