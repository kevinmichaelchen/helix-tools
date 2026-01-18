# HELIX-EMBEDDINGS AGENTS

**Parent:** See `../../AGENTS.md` and `../AGENTS.md` for shared context.

## Overview

Pluggable embedding infrastructure with config-driven provider/model selection.
Provides the `Embedder` API used by other tools. Supports multiple backends
via feature flags: fastembed (ONNX/CPU) and candle (Metal/CUDA).

## Architecture

```
┌─────────────────────────────────────────┐
│              Embedder                    │
│  (public API, uses Box<dyn Provider>)   │
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│       EmbeddingProvider trait            │
│  embed() / embed_batch() / dimension()  │
└─────────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        ▼                       ▼
┌───────────────┐       ┌───────────────┐
│ FastEmbed     │       │ Candle        │
│ Provider      │       │ Provider      │
│ (ONNX/CPU)    │       │ (Metal/CUDA)  │
└───────────────┘       └───────────────┘
```

## Providers

| Provider | Feature | GPU | Models |
|----------|---------|-----|--------|
| `fastembed` | `fastembed` (default) | CPU only | ONNX-exported models |
| `candle` | `candle` | Metal/CUDA | Any HuggingFace BERT |

Enable GPU acceleration:
- `candle` + `metal` → Apple Silicon GPU
- `candle` + `cuda` → NVIDIA GPU

## Structure

```
shared/helix-embeddings/
├── src/lib.rs             # Embedder, traits, both providers
├── docs/PROVIDER_COMPARISON.md  # Provider evaluation
└── specs/                 # requirements/design
```

## Code Map

| Symbol | Type | Role |
|--------|------|------|
| `EmbeddingProvider` | Trait | Abstract interface for any embedding backend |
| `Embedder` | Struct | Public API, wraps `Box<dyn EmbeddingProvider>` |
| `FastEmbedProvider` | Struct | ONNX-based via fastembed-rs (feature-gated) |
| `CandleProvider` | Struct | Hugging Face Candle (feature-gated) |
| `provider_from_config()` | Fn | Factory: config → provider |

## Where To Look

| Task | Location |
|------|----------|
| Add new provider | Implement `EmbeddingProvider` trait, update `provider_from_config()` |
| Change fastembed behavior | `FastEmbedProvider` impl (behind `#[cfg(feature = "fastembed")]`) |
| Change candle behavior | `CandleProvider` impl (behind `#[cfg(feature = "candle")]`) |
| Config options | `helix-config` crate (`EmbeddingConfig` struct) |

## Feature Flags

```toml
[features]
default = ["fastembed"]
fastembed = ["dep:fastembed"]
candle = ["dep:candle-core", "dep:candle-nn", ...]
metal = ["candle-core/metal", ...]  # Apple Silicon
cuda = ["candle-core/cuda", ...]    # NVIDIA
```

## Configuration

```toml
[embedding]
provider = "candle"  # or "fastembed"
model = "sentence-transformers/all-MiniLM-L6-v2"
batch_size = 32
```

## Anti-Patterns

| Don't | Why |
|-------|-----|
| Call fastembed/candle directly | Use `Embedder` API for consistency |
| Mix models in same index | Embeddings are incompatible across models |
| Skip dimension validation | Mismatched dimensions cause silent failures |
| Use candle without GPU feature | Falls back to CPU, slower than fastembed |
