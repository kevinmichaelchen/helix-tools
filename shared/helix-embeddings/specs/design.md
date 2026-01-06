# helix-embeddings: Design Specification

**Document:** design.md  
**Status:** Active (2026-01-06)  
**Author:** Kevin Chen

## Overview

helix-embeddings provides shared embedding infrastructure for helix-tools using fastembed. It generates semantic embeddings for text, enabling similarity search across decisions, issues, and documentation.

## Design Goals

1. **Unified API** — Single `Embedder` struct for all tools
2. **Offline-first** — CPU-only inference, no external APIs
3. **Configurable** — Model selection via helix-config
4. **Efficient** — Batch processing for throughput
5. **Minimal surface** — Simple API, hide fastembed complexity

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     Consumer Tools                       │
│      helix-decisions    hbd    helix-docs               │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                   helix-embeddings                       │
│  • Embedder::new() → Result<Self>                       │
│  • embed(&str) → Result<Vec<f32>>                       │
│  • embed_batch(&[&str]) → Result<Vec<Vec<f32>>>         │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                     fastembed-rs                         │
│  • ONNX Runtime inference                               │
│  • Model caching in ~/.cache/fastembed/                 │
│  • CPU-only (no GPU required)                           │
└─────────────────────────────────────────────────────────┘
```

## API Design

### Core API

```rust
use helix_embeddings::{Embedder, EmbedderConfig};

/// Create embedder with default config
let embedder = Embedder::new()?;

/// Create embedder with custom config
let config = EmbedderConfig {
    model: "BAAI/bge-base-en-v1.5".to_string(),
    batch_size: 64,
    normalize: true,
};
let embedder = Embedder::with_config(config)?;

/// Embed single text
let embedding: Vec<f32> = embedder.embed("How to handle authentication?")?;

/// Embed batch (more efficient)
let embeddings: Vec<Vec<f32>> = embedder.embed_batch(&[
    "First document",
    "Second document",
    "Third document",
])?;

/// Get embedding dimension
let dim = embedder.dimension();  // e.g., 384
```

### Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct EmbedderConfig {
    /// Model identifier (HuggingFace format)
    #[serde(default = "default_model")]
    pub model: String,
    
    /// Batch size for embed_batch
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    
    /// Normalize embeddings to unit length
    #[serde(default = "default_normalize")]
    pub normalize: bool,
}

fn default_model() -> String {
    "BAAI/bge-small-en-v1.5".to_string()
}

fn default_batch_size() -> usize {
    32
}

fn default_normalize() -> bool {
    true
}
```

### Integration with helix-config

```rust
impl Embedder {
    /// Create embedder using helix-config settings
    pub fn new() -> Result<Self, EmbedderError> {
        let shared = helix_config::load_shared_config()?;
        let config = EmbedderConfig {
            model: shared.embedding.model,
            batch_size: shared.embedding.batch_size,
            normalize: true,
        };
        Self::with_config(config)
    }
}
```

## Model Selection

### Supported Models

| Model | Dimensions | Size | Speed | Quality | Use Case |
|-------|------------|------|-------|---------|----------|
| `BAAI/bge-small-en-v1.5` | 384 | ~30MB | Fast | Good | **Default** — balanced |
| `BAAI/bge-base-en-v1.5` | 768 | ~110MB | Medium | Better | Higher quality needs |
| `sentence-transformers/all-MiniLM-L6-v2` | 384 | ~22MB | Fastest | Acceptable | Resource-constrained |

### Model Selection Guide

```
Decision tree:
1. Resource-constrained environment? → all-MiniLM-L6-v2
2. Need highest quality? → bge-base-en-v1.5
3. Default (balanced) → bge-small-en-v1.5
```

### Why These Models?

1. **English-focused** — helix-tools targets English codebases
2. **Open weights** — No API keys or licensing issues
3. **ONNX support** — Works with fastembed's runtime
4. **Proven quality** — Top performers on MTEB benchmark

## Performance Targets

### Single Embedding

| Operation | Target | Notes |
|-----------|--------|-------|
| First embed (cold) | < 2s | Model loading |
| Subsequent embed | 50-100ms | Inference only |
| Short text (< 100 chars) | ~50ms | Typical queries |
| Long text (> 1000 chars) | ~100ms | Full documents |

### Batch Embedding

| Batch Size | Target | Throughput |
|------------|--------|------------|
| 10 docs | < 500ms | 20 docs/sec |
| 32 docs | < 1.5s | 21 docs/sec |
| 100 docs | < 4s | 25 docs/sec |

**Note:** Batch processing amortizes model overhead and is significantly more efficient than individual calls.

### Memory Usage

| Model | RAM (loaded) | Peak (inference) |
|-------|--------------|------------------|
| bge-small | ~150MB | ~300MB |
| bge-base | ~400MB | ~700MB |
| all-MiniLM | ~100MB | ~200MB |

## Integration with HelixDB

helix-embeddings generates vectors; HelixDB stores and searches them.

```
┌──────────────┐     embed()     ┌──────────────┐
│   Document   │ ───────────────▶│   Vec<f32>   │
│   (text)     │                 │  (384 dims)  │
└──────────────┘                 └──────────────┘
                                        │
                                        ▼
                                 ┌──────────────┐
                                 │   HelixDB    │
                                 │  HNSW Index  │
                                 └──────────────┘
```

### Workflow

1. **Indexing:** `embed_batch()` → store vectors in HelixDB HNSW
2. **Querying:** `embed()` query → search HelixDB for nearest neighbors
3. **Results:** HelixDB returns node IDs → fetch full documents

### Important: Embedding Consistency

**All embeddings for a tool MUST use the same model.** Mixing models produces incompatible vectors.

```rust
// CORRECT: Same model for index and query
let embedder = Embedder::new()?;  // Uses configured model
let doc_embedding = embedder.embed(&document)?;
let query_embedding = embedder.embed(&query)?;

// WRONG: Different models
let index_embedder = Embedder::with_config(bge_small)?;
let query_embedder = Embedder::with_config(bge_base)?;  // Incompatible!
```

**Migration:** If you change models, you must re-embed all documents.

## Error Handling

```rust
#[derive(Debug, Error)]
pub enum EmbedderError {
    #[error("Model not found: {model}")]
    ModelNotFound { model: String },
    
    #[error("Model download failed: {source}")]
    DownloadFailed {
        #[source]
        source: std::io::Error,
    },
    
    #[error("Inference failed: {message}")]
    InferenceFailed { message: String },
    
    #[error("Config error: {source}")]
    ConfigError {
        #[from]
        source: helix_config::ConfigError,
    },
}
```

## Caching

### Model Cache

fastembed caches downloaded models in `~/.cache/fastembed/`:

```
~/.cache/fastembed/
├── BAAI--bge-small-en-v1.5/
│   ├── model.onnx
│   ├── tokenizer.json
│   └── config.json
└── sentence-transformers--all-MiniLM-L6-v2/
    └── ...
```

**First run:** Downloads model (~30MB for default)
**Subsequent runs:** Uses cached model (instant startup)

### Embedding Cache

helix-embeddings does NOT cache embeddings. That's the consumer's responsibility:
- helix-decisions: Stores in HelixDB (or helix-storage JSON)
- hbd: Stores in HelixDB
- helix-docs: Stores in HelixDB

## Thread Safety

`Embedder` is `Send + Sync` and can be shared across threads:

```rust
let embedder = Arc::new(Embedder::new()?);

// Multiple threads can call embed concurrently
let handles: Vec<_> = texts.into_iter().map(|text| {
    let e = Arc::clone(&embedder);
    thread::spawn(move || e.embed(&text))
}).collect();
```

**Note:** Internal model is NOT parallel. Concurrent calls are serialized via internal mutex. For throughput, use `embed_batch()` instead of parallel `embed()` calls.

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[ignore]  // Requires model download
    fn test_embed_single() {
        let embedder = Embedder::new().unwrap();
        let embedding = embedder.embed("test").unwrap();
        assert_eq!(embedding.len(), 384);
    }
    
    #[test]
    #[ignore]  // Requires model download
    fn test_embed_batch() {
        let embedder = Embedder::new().unwrap();
        let embeddings = embedder.embed_batch(&["a", "b", "c"]).unwrap();
        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|e| e.len() == 384));
    }
    
    #[test]
    #[ignore]  // Requires model download
    fn test_similarity() {
        let embedder = Embedder::new().unwrap();
        let e1 = embedder.embed("database migration").unwrap();
        let e2 = embedder.embed("database schema changes").unwrap();
        let e3 = embedder.embed("cooking recipes").unwrap();
        
        let sim_12 = cosine_similarity(&e1, &e2);
        let sim_13 = cosine_similarity(&e1, &e3);
        
        assert!(sim_12 > sim_13);  // Related texts more similar
    }
}
```

### Integration Tests

Tests are `#[ignore]` by default because they require model download (~30MB).

Run with: `cargo test --ignored -p helix-embeddings`

## Consumers

| Tool | Use Case | Batch Size | Model |
|------|----------|------------|-------|
| helix-decisions | Decision search | 10-50 | bge-small (default) |
| hbd | Issue search | 10-100 | bge-small (default) |
| helix-docs | Doc chunk search | 100-1000 | bge-small or bge-base |

---

## See Also

- [requirements.md](./requirements.md) — Requirements specification
- [helix-config/specs/design.md](../helix-config/specs/design.md) — Configuration
- [fastembed docs](https://docs.rs/fastembed) — Underlying library
