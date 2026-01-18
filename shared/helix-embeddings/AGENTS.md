# HELIX-EMBEDDINGS AGENTS

**Parent:** See `../../AGENTS.md` and `../AGENTS.md` for shared context.

## Overview

Pluggable embedding infrastructure with config-driven provider/model selection.
Provides the `Embedder` API used by other tools. Currently supports fastembed
(ONNX-based), but architecture allows adding non-ONNX backends.

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
│ FastEmbed     │       │ Future:       │
│ Provider      │       │ Candle, Ollama│
│ (ONNX)        │       │ OpenAI, etc.  │
└───────────────┘       └───────────────┘
```

## Structure

```
shared/helix-embeddings/
├── src/lib.rs             # Embedder, EmbeddingProvider trait, FastEmbedProvider
└── specs/                 # requirements/design (outdated re: provider arch)
```

## Code Map

| Symbol                          | Type   | Line | Role                                            |
| ------------------------------- | ------ | ---- | ----------------------------------------------- |
| `EmbeddingProvider`             | Trait  | 38   | Abstract interface for any embedding backend    |
| `Embedder`                      | Struct | 49   | Public API, wraps `Box<dyn EmbeddingProvider>`  |
| `FastEmbedProvider`             | Struct | 99   | ONNX-based implementation via fastembed-rs      |
| `provider_from_config()`        | Fn     | 196  | Factory: config → provider (only fastembed now) |
| `fastembed_model_from_string()` | Fn     | 204  | Flexible model name parsing                     |

## Where To Look

| Task                      | Location                                                                   |
| ------------------------- | -------------------------------------------------------------------------- |
| Add new provider          | Implement `EmbeddingProvider` trait, update `provider_from_config()`       |
| Change fastembed behavior | `FastEmbedProvider` impl (line 106-194)                                    |
| Add supported model       | Update `fastembed_model_from_string()` or use fastembed's built-in parsing |
| Config options            | `helix-config` crate (`EmbeddingConfig` struct)                            |

## Adding a New Provider

1. Implement `EmbeddingProvider` trait:
   ```rust
   struct MyProvider { /* ... */ }

   impl EmbeddingProvider for MyProvider {
       fn embed(&self, text: &str) -> Result<Vec<f32>>;
       fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
       fn dimension(&self) -> usize;
       fn model_name(&self) -> &str;
       fn provider_name(&self) -> &'static str;
   }
   ```

2. Update `provider_from_config()` to handle new provider name

3. Update `EmbeddingConfig` in helix-config if new fields needed

## Current Limitations

- Only `fastembed` provider implemented (ONNX models)
- No GPU support (CPU-only via fastembed)
- No remote API providers (OpenAI, Ollama, etc.)

## Anti-Patterns

| Don't                     | Why                                         |
| ------------------------- | ------------------------------------------- |
| Call fastembed directly   | Use `Embedder` API for consistency          |
| Mix models in same index  | Embeddings are incompatible across models   |
| Skip dimension validation | Mismatched dimensions cause silent failures |
