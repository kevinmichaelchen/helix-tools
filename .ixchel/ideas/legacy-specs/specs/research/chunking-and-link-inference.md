# Research Report: Chunk-Level Indexing and Relationship Inference

This report summarizes research on the architectural ideas introduced in commit
`4b3c3bc` — specifically chunk-level vector indexing and relationship inference
pipelines for knowledge graphs.

## Executive Summary

The Ixchel architecture's approach to chunking and link inference aligns with
current best practices in the RAG and knowledge graph domains. Key validations:

- **Chunk-level indexing** with centroid vectors addresses the precision vs
  context trade-off
- **Four-stage suggestion pipelines** (retrieve → filter → rerank → materialize)
  follow established patterns
- **Confidence-tagged edges** for suggested vs confirmed links is a clean,
  well-supported distinction

## 1. Chunk-Level Vector Indexing

### The Core Trade-Off

Traditional vector-only RAG systems face an inherent contradiction: small chunks
enable precise semantic matching but fragment context, while large chunks
preserve coherence but dilute semantic focus[ragflow]. This tension directly
impacts retrieval quality for complex queries requiring multi-hop reasoning.

### Ixchel's Approach

The architecture addresses this with:

| Component                         | Purpose                                   |
| --------------------------------- | ----------------------------------------- |
| ~512 token chunks                 | Fine-grained semantic matching            |
| Heading-based boundaries          | Preserve semantic coherence               |
| Centroid vectors per node         | Coarse recall and document-level matching |
| Chunk IDs on nodes (`vector_ids`) | Section-level recall and scoring          |

### Research Validation

**Structure-aware chunking** using document structure (headings, ASTs) prevents
semantically coupled content from being split across chunks, improving embedding
quality[arxiv-ast]. This principle extends naturally to heading-based strategies.

**GraphRAG integration** demonstrates that combining chunk-level vectors with
knowledge graphs provides approximately 7.4% absolute accuracy improvement over
vector-only approaches[video-search]. The graph enables cross-chunk context and
multi-hop reasoning that similarity search alone cannot achieve[graphrag].

**Cross-node aggregation** via relationship paths achieves superior robustness on
complex queries, even when related information is dispersed across different
nodes[oreateai-rag].

### HNSW Configuration

The specified HNSW parameters are within recommended ranges:

```rust
HnswConfig { m: 16, ef_construction: 200, ef_search: 64 }
```

Current best practices (2025-2026)[arxiv-hnsw][mariadb-vector]:

- **M (graph degree)**: 16-32 for balanced accuracy/performance
- **ef_search**: 100-200 for higher recall at throughput cost
- **Distance metric**: Cosine similarity for normalized embeddings

## 2. Relationship Inference Pipeline

### Ixchel's Four-Stage Pipeline

1. **Chunk retrieval**: Vector search for top-k candidate pairs across entity
   types
2. **Pair filtering**: Heuristic filters (type-specific patterns, keyword
   matching)
3. **Rerank/classify**: Cross-encoder scoring with relation labels + confidence
4. **Materialize suggestions**: Attach edges with `confidence`; require
   confirmation to promote

### Research Validation

**Two-stage retrieval** is standard practice: bi-encoder retrieval (fast, ~50-100
candidates) followed by cross-encoder reranking for precision[arxiv-reranker].
Key models include:

| Model             | Type             | Notes                                 |
| ----------------- | ---------------- | ------------------------------------- |
| BGE-reranker      | Cross-encoder    | High-precision reranking              |
| ColBERT           | Late interaction | Token-level matching, efficient       |
| ModernBERT (2024) | Cross-encoder    | Strong NanoBEIR performance           |
| RWKV rerankers    | State-based      | Efficient alternative to transformers |

**Link prediction confidence scoring** approaches[pmc-link-prediction][emnlp-calibrator]:

- **Embedding-based scores**: Similarity metrics (dot product of head-tail
  embeddings)
- **Probability calibration**: Post-hoc methods adjust raw outputs for
  well-calibrated probabilities
- **Hybrid evaluation**: Combine topological and multimodal features with
  threshold filtering

One study demonstrated 19.8% graph size reduction by filtering low-confidence
edges using median alignment score thresholds[pmc-link-prediction].

### Confidence Semantics

The pattern of setting `confidence` on auto-mined links while confirmed edges
omit it provides clear semantics:

```
Suggested edge:  (A)-[IMPLEMENTS {confidence: 0.82}]->(B)
Confirmed edge:  (A)-[IMPLEMENTS]->(B)
```

This aligns with knowledge graph embedding research where calibrated confidence
enables reliable filtering and user review workflows[emnlp-calibrator].

### Chunk Span Provenance

Storing which section/paragraph drove a suggested link enables:

- Explainable suggestions for user/agent review
- Debugging of false positives
- Incremental re-evaluation when source chunks change

## 3. Industry Trends (2025-2026)

### RAG Evolution

RAG systems are evolving toward "Context Engines" with hybrid indexing
(full-text, vector, and tensor) and text-first tensor reranking for better
semantic recall[ragflow-2025]. AutoRAG approaches automate pipeline optimization,
treating rerankers as part of a searchable configuration space[autorag].

### Knowledge Graph + Vector Hybrid

The combination of knowledge graphs with vector retrieval is increasingly
standard for production systems requiring:

- Multi-hop reasoning
- Constraint-based queries
- Cross-document relationship tracking

GraphRAG demonstrates the pattern: after chunking and vectorization, extract
entities and relationships, cluster into communities, and generate
summaries[graphrag].

## 4. Recommendations

### Near-Term Enhancements

1. **Reranker/classifier selection**: ColBERT-style late interaction models are
   retrieval-focused; use them for retrieval ablations only. For relation
   labeling, prefer cross-encoders unless investing in a custom late-interaction
   classifier. Run a small ablation before committing infra changes:
   - BGE cross-encoder (baseline, recommended for relation labeling)
   - ColBERT-v2 (retrieval stage only)
   - Lightweight ModernBERT cross-encoder
   - Measure latency and precision on a sampled edge-label dataset

2. **Per-relation confidence calibration**: Calibrate confidence scores per
   relation type using a held-out labeled set:
   - Start with temperature scaling or Dirichlet calibration
   - Track reliability curves (predicted vs actual accuracy)
   - Persist calibrator parameters alongside model version so confidence
     semantics remain stable across deploys

3. **Adaptive HNSW search with guardrails**:
   - Start with lower ef, increase until k qualified results found[arxiv-hnsw]
   - Add latency guardrail (e.g., p99 < 50ms) and log when falling back to
     higher ef
   - M parameter tuning: higher M for smaller collections or high-accuracy
     slices; lower M for very large collections prioritizing speed/memory

### Provenance-Backed Invalidation

When source chunks change, suggestion validity degrades. Implement automatic
invalidation based on three triggers:

- **Content hash change**: Detect chunk modifications via `content_hash`
  comparison during sync
- **Model version change**: Invalidate when embedding model version changes
  (stored alongside vectors)
- **TTL expiry**: Time-based invalidation for suggestions that haven't been
  confirmed within a threshold

On invalidation:

- Downgrade or remove suggestions that cite changed chunks
- Cascade re-embedding and re-evaluation for impacted nodes
- This preserves confidence semantics over time and bounds re-eval cost

Track **chunk drift** (how often source chunks change) as a leading indicator of
suggestion invalidation volume.

### Evaluation Harness

Define a small, versioned eval harness (MT-RAIG/RAGAS/ARES style) to gate
changes to chunking, retrieval params, rerankers, and calibrators:

- ~200-500 labeled pairs (relation type + ground truth label)
- Run on every config change before deploy
- Track regression on precision, recall, and latency SLOs
- Version the eval set alongside pipeline config

This de-risks all proposed enhancements by catching regressions early.

### Metrics to Track

| Metric                 | Purpose                                          |
| ---------------------- | ------------------------------------------------ |
| Suggestion precision@k | Quality of top-k suggested edges                 |
| True edge recall       | Coverage of known-good edges (requires gold set) |
| Confirmation rate      | % of suggestions promoted to canonical           |
| Chunk recall           | Whether relevant chunks appear in candidates     |
| Confidence calibration | Predicted vs actual accuracy correlation         |
| Chunk drift            | Rate of source chunk changes (invalidation cost) |
| Latency p50/p99        | Search and rerank latency SLOs                   |

## References

[arxiv-ast]: https://arxiv.org/html/2601.08773v1
[arxiv-hnsw]: https://arxiv.org/html/2601.01291v2
[arxiv-reranker]: https://arxiv.org/html/2601.07861v1
[autorag]: https://ai.gopubby.com/autorag-the-end-of-guesswork-in-retrieval-augmented-generation-cc9ac0ad578c
[emnlp-calibrator]: https://aclanthology.org/2025.emnlp-main.1522/
[graphrag]: https://www.intelligentmachines.blog/post/graphrag-explained-boosting-rag-performance-with-knowledge-graphs
[mariadb-vector]: https://severalnines.com/blog/introduction-to-mariadb-vector-search/
[oreateai-rag]: https://www.oreateai.com/blog/a-comparative-study-of-rag-retrievalaugmented-generation-and-kg-knowledge-graph-technologies/fe3580f238aa09853a00da600a3f9325
[pmc-link-prediction]: https://pmc.ncbi.nlm.nih.gov/articles/PMC12791124/
[ragflow]: https://ragflow.io/basics/what-is-rag
[ragflow-2025]: https://ragflow.io/blog/rag-review-2025-from-rag-to-context
[video-search]: https://thedataguy.pro/blog/2025/12/production-video-search-infrastructure/
