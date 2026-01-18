# Embedding Provider Comparison

**Document:** PROVIDER_COMPARISON.md
**Status:** Draft (2026-01-17)
**Purpose:** Evaluate embedding backends for helix-embeddings pluggable provider architecture

## Executive Summary

| Provider      | Integration | GPU Support | Rust Native  | Recommended For                |
| ------------- | ----------- | ----------- | ------------ | ------------------------------ |
| **fastembed** | ✅ Current  | CPU only    | ✅ Yes       | Default, simple deployments    |
| **Candle**    | 🔧 Planned  | Metal/CUDA  | ✅ Yes       | Pure Rust, GPU acceleration    |
| **llama.cpp** | 🔧 Planned  | Metal/CUDA  | Via bindings | GGUF models, memory efficiency |
| **Ollama**    | 🔧 Planned  | Metal/CUDA  | HTTP API     | Easy setup, model management   |
| **vLLM**      | 🔧 Planned  | CUDA/Metal* | HTTP API     | High-throughput, production    |
| **MLX**       | 🔧 Planned  | Metal only  | Via bindings | Apple Silicon optimization     |

*vLLM Metal support via vllm-metal plugin

---

## Provider Deep Dive

### 1. FastEmbed (Current Implementation)

**Backend:** ONNX Runtime
**Crate:** `fastembed` (Qdrant)

#### Pros

- ✅ Already implemented in helix-embeddings
- ✅ Pure Rust with minimal dependencies
- ✅ Good model selection (BGE, MiniLM, etc.)
- ✅ No external server required
- ✅ Cross-platform (Linux, macOS, Windows)

#### Cons

- ❌ CPU-only inference
- ❌ No GPU acceleration
- ❌ Limited to ONNX-exported models

#### Performance

| Model             | Dimensions | Memory | Latency (CPU) |
| ----------------- | ---------- | ------ | ------------- |
| bge-small-en-v1.5 | 384        | ~150MB | 50-100ms      |
| all-MiniLM-L6-v2  | 384        | ~100MB | 40-80ms       |
| bge-base-en-v1.5  | 768        | ~400MB | 100-200ms     |

#### Best For

- Development and prototyping
- CPU-only environments
- Simple deployments without GPU

---

### 2. Candle (HuggingFace)

**Backend:** Pure Rust ML framework
**Crate:** `candle-core`, `candle-transformers`

#### Pros

- ✅ Pure Rust, no C++ dependencies
- ✅ Metal support for Apple Silicon
- ✅ CUDA support for NVIDIA GPUs
- ✅ Direct HuggingFace model loading
- ✅ Memory efficient

#### Cons

- ❌ Smaller ecosystem than PyTorch
- ❌ Some models need manual porting
- ❌ Less mature than ONNX Runtime

#### Implementation Notes

```rust
// Potential Candle provider structure
use candle_core::{Device, Tensor};
use candle_transformers::models::bert::BertModel;

struct CandleProvider {
    model: BertModel,
    device: Device, // Cpu, Cuda(0), Metal
}
```

#### Best For

- GPU-accelerated inference
- Pure Rust codebases
- Apple Silicon optimization

---

### 3. llama.cpp

**Backend:** C++ with Rust bindings
**Crate:** `llama-cpp-rs`

#### Pros

- ✅ Excellent Metal support on Apple Silicon
- ✅ GGUF quantized models (memory efficient)
- ✅ Battle-tested, widely used
- ✅ Supports embedding models

#### Cons

- ❌ C++ dependency (not pure Rust)
- ❌ Primarily designed for LLMs, not embeddings
- ❌ Build complexity (BLAS, Metal, CUDA flags)

#### Performance on Apple Silicon

| Setting    | Recommendation           |
| ---------- | ------------------------ |
| Threads    | -1 (auto, all cores)     |
| KV Cache   | f16 → q8_0 if OOM        |
| Batch Size | 2048 logical, 512 uBatch |

#### Rust Integration

```rust
use llama_cpp_rs::{Model, Context};

let model = Model::load("bge-small.gguf")?;
let mut ctx = model.context(512, 512)?;
ctx.set_embedding_input("text")?;
ctx.compute_embedding()?;
let embedding: Vec<f32> = ctx.embedding();
```

#### Best For

- GGUF quantized models
- Memory-constrained environments
- Mixed LLM + embedding workloads

---

### 4. Ollama

**Backend:** Go server wrapping llama.cpp
**Integration:** HTTP API (OpenAI-compatible)

#### Pros

- ✅ Dead simple setup (`ollama pull`, `ollama serve`)
- ✅ Model management built-in
- ✅ OpenAI-compatible `/v1/embeddings` endpoint
- ✅ Automatic GPU detection (Metal, CUDA)

#### Cons

- ❌ Requires running server
- ❌ HTTP overhead vs in-process
- ❌ Not a Rust library (external dependency)

#### Performance

- Retrieval + generation: **< 500ms** on consumer hardware
- Recommended model: `nomic-embed-text` (~340M params)

#### API Example

```bash
curl http://localhost:11434/api/embeddings \
  -d '{"model": "nomic-embed-text", "prompt": "Hello world"}'
```

#### Best For

- Quick prototyping
- Teams already using Ollama
- Multi-model deployments

---

### 5. vLLM

**Backend:** Python server with PagedAttention
**Integration:** HTTP API (OpenAI-compatible)

#### Pros

- ✅ **2-4x throughput** vs naive serving (PagedAttention)
- ✅ High concurrency handling
- ✅ Production-grade batching
- ✅ Supports embedding models (BGE, etc.)

#### Cons

- ❌ Python dependency
- ❌ Requires running server
- ❌ Heavier than Ollama
- ❌ Primarily GPU-focused (CPU mode slower)

#### vLLM-Metal (Apple Silicon)

- Plugin enabling vLLM on Apple Silicon via MLX backend
- Zero-copy operations on unified memory
- Full vLLM API compatibility

#### Usage

```bash
# Start embedding server
vllm serve BAAI/bge-large-en-v1.5 --port 8000

# Or direct Python
from vllm import LLM
llm = LLM(model="BAAI/bge-large-en-v1.5", task="embed")
output = llm.encode("text")
```

#### Best For

- High-throughput production
- GPU clusters
- Concurrent request handling

---

### 6. MLX (Apple)

**Backend:** Apple's ML framework for Apple Silicon
**Integration:** Python/Swift, Rust via bindings

#### Pros

- ✅ **Best performance on Apple Silicon**
- ✅ Optimized for unified memory
- ✅ Higher tok/s than llama.cpp on M-series
- ✅ Lower memory usage

#### Cons

- ❌ Apple Silicon only (no Linux/Windows)
- ❌ Python-first (Rust bindings less mature)
- ❌ Smaller model ecosystem

#### Performance vs Alternatives

| Framework | Apple Silicon Performance           |
| --------- | ----------------------------------- |
| MLX       | Best (optimized for unified memory) |
| llama.cpp | Good (Metal backend)                |
| Ollama    | Good (uses llama.cpp)               |
| ONNX      | CPU only                            |

#### Best For

- Apple Silicon deployments
- Maximum performance on Mac
- Memory-constrained Mac workflows

---

## Comparison Matrix

### By Use Case

| Use Case               | Recommended Provider | Rationale             |
| ---------------------- | -------------------- | --------------------- |
| **Development**        | fastembed            | Simple, no setup      |
| **macOS Production**   | MLX or Candle        | GPU acceleration      |
| **Linux GPU Server**   | vLLM                 | High throughput       |
| **Memory Constrained** | llama.cpp (GGUF)     | Quantized models      |
| **Multi-model**        | Ollama               | Easy model management |
| **Pure Rust**          | Candle               | No C++ deps           |

### By Hardware

| Hardware              | Best Provider  | Alternative           |
| --------------------- | -------------- | --------------------- |
| **Apple M1/M2/M3/M4** | MLX            | Candle (Metal)        |
| **NVIDIA GPU**        | vLLM           | Candle (CUDA)         |
| **CPU Only**          | fastembed      | llama.cpp (quantized) |
| **Low Memory (<8GB)** | llama.cpp (Q4) | fastembed (MiniLM)    |

### Feature Matrix

| Feature   | fastembed | Candle | llama.cpp | Ollama | vLLM | MLX |
| --------- | --------- | ------ | --------- | ------ | ---- | --- |
| Pure Rust | ✅        | ✅     | ❌        | ❌     | ❌   | ❌  |
| No Server | ✅        | ✅     | ✅        | ❌     | ❌   | ✅  |
| Metal GPU | ❌        | ✅     | ✅        | ✅     | ✅*  | ✅  |
| CUDA GPU  | ❌        | ✅     | ✅        | ✅     | ✅   | ❌  |
| Quantized | ❌        | ⚠️      | ✅        | ✅     | ⚠️    | ✅  |
| Batching  | ✅        | ✅     | ✅        | ❌     | ✅   | ✅  |

*via vllm-metal plugin

---

## Memory Footprint (Common Models)

| Model             | Params | Dims | FP32 Size | Quantized (Q4) |
| ----------------- | ------ | ---- | --------- | -------------- |
| all-MiniLM-L6-v2  | 22.3M  | 384  | ~89 MB    | ~25 MB         |
| bge-small-en-v1.5 | 33.5M  | 384  | ~134 MB   | ~35 MB         |
| bge-base-en-v1.5  | 109M   | 768  | ~436 MB   | ~110 MB        |
| nomic-embed-text  | 137M   | 768  | ~548 MB   | ~140 MB        |

---

## Implementation Priority

Based on helix-tools requirements (offline-first, Rust-native, Apple Silicon support):

### Phase 1: Current

1. **fastembed** ✅ — Already implemented, good default

### Phase 2: High Priority

2. **Candle** — Pure Rust with Metal support
3. **Ollama** — Easy to implement (HTTP client), wide adoption

### Phase 3: Future

4. **llama.cpp** — For GGUF model support
5. **vLLM** — For high-throughput production
6. **MLX** — If Rust bindings mature

---

## Recommendation

**For helix-tools specifically:**

1. **Keep fastembed as default** — Works everywhere, no GPU required
2. **Add Candle next** — Pure Rust, Metal support, aligns with project philosophy
3. **Add Ollama** — Easy win, many users already have it running

This gives users:

- CPU fallback (fastembed)
- GPU acceleration (Candle)
- Flexibility (Ollama for any model they want)

---

## References

- [fastembed-rs](https://github.com/qdrant/fastembed-rs)
- [Candle](https://github.com/huggingface/candle)
- [llama-cpp-rs](https://github.com/rustformers/llama-cpp-rs)
- [Ollama](https://ollama.com)
- [vLLM](https://github.com/vllm-project/vllm)
- [vLLM-Metal](https://github.com/vllm-project/vllm-metal)
- [MLX](https://github.com/ml-explore/mlx)
- [Open Source Embedding Models Benchmark](https://research.aimultiple.com/open-source-embedding-models/)
