# Embedding Model Performance on Apple Silicon

**Target Hardware:** Apple M1 Pro, 16GB unified memory
**Use Case:** Indexing issue/ticket text for semantic search

## TL;DR

| Model                 | Latency/Ticket | Throughput      | 1,000 Tickets |
| --------------------- | -------------- | --------------- | ------------- |
| all-MiniLM-L6-v2      | **0.2-1ms**    | 1,000-5,000/sec | **< 1 sec**   |
| bge-small-en-v1.5     | 1-2ms          | 500-1,000/sec   | 1-2 sec       |
| bge-base-en-v1.5      | 3-5ms          | 200-300/sec     | 3-5 sec       |
| nomic-embed-text-v1.5 | 5-10ms         | 100-200/sec     | 5-10 sec      |

**Bottom line:** For indexing issues, even the slowest model processes your entire
backlog in seconds. Model choice should prioritize **quality**, not speed.

---

## Understanding the Numbers

### What is an "Average Ticket"?

| Metric      | Typical Value | Notes                        |
| ----------- | ------------- | ---------------------------- |
| Title       | 50-100 chars  | "Fix authentication bug"     |
| Description | 200-500 chars | Bug report with steps        |
| Total       | 300-600 chars | ~75-150 words                |
| **Tokens**  | **50-150**    | ~0.75 words per token (BERT) |

Most embedding models have a 512-token context window. A typical issue uses
only **10-30%** of that capacity, meaning inference is fast.[^1]

### What Affects Speed?

1. **Model size** - Fewer parameters = faster inference
2. **Sequence length** - Shorter text = less computation
3. **Backend** - ONNX (CPU) vs Candle (Metal GPU) vs MLX
4. **Batch size** - Batching amortizes overhead

---

## Benchmark Data

### FastEmbed (ONNX Runtime, CPU)

FastEmbed uses ONNX Runtime optimized for CPU inference. On Apple Silicon,
this runs on the high-performance CPU cores.

| Model             | Params | Dims | Throughput    | Latency |
| ----------------- | ------ | ---- | ------------- | ------- |
| all-MiniLM-L6-v2  | 22M    | 384  | ~5,000 sent/s | ~0.2ms  |
| bge-small-en-v1.5 | 33M    | 384  | ~2,500 sent/s | ~0.4ms  |
| bge-base-en-v1.5  | 109M   | 768  | ~800 sent/s   | ~1.2ms  |

**Source:** Rust ONNX Runtime achieves 3-5x higher throughput than Python
equivalents with 80-90% less memory.[^2]

### Candle (Metal GPU)

Candle leverages Apple's Metal API for GPU acceleration on M-series chips.

| Operation  | Metal (M1 Pro) | CPU     | Speedup |
| ---------- | -------------- | ------- | ------- |
| Softmax    | 41.5 µs        | 216 µs  | 5.2x    |
| Layer Norm | 45.8 µs        | 116 µs  | 2.5x    |
| RMS Norm   | 25.0 µs        | 60.4 µs | 2.4x    |

**Embedding batch scaling:** Near constant-time performance where batch
1 to 100 increases latency by only 13% (3.9ms to 4.4ms).[^3]

### MLX (Apple Native)

MLX is Apple's framework optimized for unified memory architecture.

| Framework  | Relative Speed | Notes                          |
| ---------- | -------------- | ------------------------------ |
| MLX        | Fastest        | Fused Metal kernels, zero-copy |
| Candle     | ~2-6x slower   | Multiple kernel launches       |
| ONNX (CPU) | Varies         | No GPU, but highly optimized   |

MLX achieves 30-50% faster inference than alternatives like Ollama due to
direct Metal Performance Shaders mapping.[^4]

---

## Real-World Scenarios

### Scenario 1: Index 100 Issues (Small Project)

```
Model: all-MiniLM-L6-v2 (fastembed)
Average ticket: 100 tokens
Latency: 0.2ms per ticket

Total time: 100 × 0.2ms = 20ms
```

**Result:** Imperceptible. Faster than a network round-trip.

### Scenario 2: Index 10,000 Issues (Large Backlog)

```
Model: bge-small-en-v1.5 (fastembed)
Average ticket: 100 tokens
Latency: 0.4ms per ticket

Total time: 10,000 × 0.4ms = 4 seconds
With batching (32): ~2 seconds
```

**Result:** A few seconds. Background task while you grab coffee.

### Scenario 3: Real-Time Search Query

```
Query: "authentication timeout errors" (~5 tokens)
Model: all-MiniLM-L6-v2

Embedding latency: < 1ms
Vector search (10k vectors): < 5ms
Total: < 10ms
```

**Result:** Instant. Users won't notice any delay.

---

## Recommendations

### For helix-tools (hbd, helix-decisions)

| Provider    | Model             | When to Use                     |
| ----------- | ----------------- | ------------------------------- |
| `fastembed` | all-MiniLM-L6-v2  | Default. Fast, good quality     |
| `fastembed` | bge-small-en-v1.5 | Better retrieval, still fast    |
| `candle`    | bge-small-en-v1.5 | GPU acceleration, balanced      |
| `candle`    | bge-large-en-v1.5 | **Best quality** (requires GPU) |

### Model Selection Guide

```
Speed Priority:     all-MiniLM-L6-v2 (22M params, 384 dims)
Balanced:           bge-small-en-v1.5 (33M params, 384 dims)
Quality Priority:   bge-large-en-v1.5 (335M params, 1024 dims) ← Recommended with GPU
```

### Memory Considerations (16GB M1 Pro)

| Model             | Memory  | Headroom                      |
| ----------------- | ------- | ----------------------------- |
| all-MiniLM-L6-v2  | ~89 MB  | Plenty for large indices      |
| bge-small-en-v1.5 | ~134 MB | Comfortable                   |
| bge-base-en-v1.5  | ~436 MB | Fine for typical workloads    |
| bge-large-en-v1.5 | ~1.3 GB | Comfortable with 16GB         |
| nomic-embed-text  | ~548 MB | Works, but watch for pressure |

---

## Comparison: Why Not Larger Models?

| Model                   | Params | Quality (MTEB)  | Speed   | Verdict              |
| ----------------------- | ------ | --------------- | ------- | -------------------- |
| all-MiniLM-L6-v2        | 22M    | Good (~63)      | Fastest | Best for dev         |
| bge-small-en-v1.5       | 33M    | Better (~63)    | Fast    | Good balance         |
| bge-base-en-v1.5        | 109M   | Great (~64)     | Medium  | Quality without GPU  |
| bge-large-en-v1.5       | 335M   | Excellent (~65) | Slower  | **Best with GPU**    |
| llama-embed-nemotron-8b | 7.5B   | Best (~70)      | Slow    | Overkill for tickets |

For issue/ticket search, **bge-large-en-v1.5 with GPU** hits the sweet spot:
high quality retrieval while staying within your 10-second budget for 10k tickets.
Models beyond 500M params offer diminishing returns for short-form technical text.[^5]

---

## Glossary

| Term       | Meaning                                              |
| ---------- | ---------------------------------------------------- |
| Latency    | Time to process one request                          |
| Throughput | Requests processed per second                        |
| Token      | ~0.75 words (subword unit for BERT models)           |
| MTEB       | Massive Text Embedding Benchmark (quality metric)    |
| Metal      | Apple's GPU API for M-series chips                   |
| ONNX       | Open Neural Network Exchange (portable model format) |

---

## References

[^1]: [Semantic Search Architecture][hakia] - Typical document tokenization patterns

[^2]: [Building Sentence Transformers in Rust][rust-st] - ONNX Runtime vs Python performance comparison

[^3]: [metal-candle Benchmarks][metal-candle] - Apple Silicon GPU acceleration metrics

[^4]: [LLM Inference Speed Comparison][mlx-bench] - MLX vs Ollama on Apple Silicon

[^5]: [MTEB Leaderboard][mteb] - Embedding model quality benchmarks

[hakia]: https://www.hakia.com/tech-insights/how-semantic-search-works/
[rust-st]: https://dev.to/mayu2008/building-sentence-transformers-in-rust-a-practical-guide-with-burn-onnx-runtime-and-candle-281k
[metal-candle]: https://github.com/GarthDB/metal-candle/blob/main/BENCHMARKS.md
[mlx-bench]: https://singhajit.com/llm-inference-speed-comparison/
[mteb]: https://huggingface.co/spaces/mteb/leaderboard
