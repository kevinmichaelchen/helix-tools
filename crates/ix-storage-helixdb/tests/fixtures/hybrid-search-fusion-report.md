# Hybrid Search: BM25 and Vector Fusion Strategies

**Technical Report | Systems Engineering Audience**

## Context

Hybrid search combines lexical retrieval (BM25) with semantic retrieval (dense vectors) to capture both exact keyword matches and conceptual similarity [1][2]. Neither approach alone covers all query types: BM25 excels at specific terminology and proper nouns; vector search captures paraphrases and semantic intent.

This report examines score normalization challenges, fusion algorithms, and re-ranking strategies for production hybrid search systems.

## Core Mechanics

### BM25 Scoring

BM25 (Best Matching 25) computes relevance as a weighted sum over query terms [3][4]:

```
Score(q,d) = Σ IDF(t) × TF(t,d)
```

Where TF incorporates saturation and document length normalization:

```
TF(t,d) = freq(t,d) / (freq(t,d) + k1 × (1 - b + b × |d|/avgdl))
```

Key parameters:

- **k1** (typically 1.2–2.0): Controls term frequency saturation
- **b** (typically 0.75): Controls document length normalization [3][5][6]

BM25 scores are **unbounded and corpus-dependent**—a score of 15.3 in one corpus is not comparable to 15.3 in another [1][3].

### Vector Scoring

Dense embedding models produce fixed-dimension vectors, with similarity computed via cosine distance, dot product, or Euclidean distance [1][2]. Cosine similarity produces scores in [-1, 1], typically normalized to [0, 1] for ranking purposes.

Vector search captures semantic relationships but may miss exact terminology. A query for "PostgreSQL" might retrieve documents about "MySQL" or "relational databases" while missing documents that mention "PostgreSQL" only in passing.

### The Normalization Problem

Fusing BM25 and vector scores requires reconciling incompatible scales [1][3][8]:

| Metric            | BM25                         | Vector (Cosine)      |
| ----------------- | ---------------------------- | -------------------- |
| Range             | [0, ∞)                       | [-1, 1]              |
| Distribution      | Long-tailed, query-dependent | Approximately normal |
| Corpus dependence | High                         | Model-dependent      |

Direct weighted averaging (`0.5 × bm25 + 0.5 × vector`) produces unstable results because BM25 scores vary wildly across queries.

### Normalization Strategies

**Min-Max Scaling**

Normalize each score type to [0, 1] within the result set:

```
normalized = (score - min) / (max - min)
```

Pros: Simple, bounded output
Cons: Sensitive to outliers; requires computing min/max per query [8]

**Z-Score Normalization**

Standardize to zero mean, unit variance:

```
normalized = (score - mean) / stddev
```

Pros: Robust to outliers
Cons: Can produce negative scores; requires corpus statistics [8]

**Reciprocal Rank Fusion (RRF)**

Bypass raw scores entirely by fusing ranks [2][5][6]:

```
RRF(d) = Σ 1 / (k + rank(d))
```

Where k is a smoothing constant (typically 60). Documents appearing in multiple result sets receive boosted scores.

Pros: No normalization needed; stable across corpora
Cons: Discards score magnitude information [2][5][7]

## Operational Risks

### Keyword-Semantic Drift

When vector and BM25 retrievers return disjoint result sets, fusion algorithms must choose between them. Without careful weighting, one modality may dominate inappropriately [1][2].

**Symptom**: Queries for specific product codes or error messages return semantically similar but incorrect results.

**Mitigation**: Boost BM25 weight for queries containing rare terms or exact-match patterns.

### Score Distribution Mismatch

BM25 scores for different queries can differ by orders of magnitude. Min-max normalization on a query returning scores [0.1, 0.2, 0.3] produces the same [0, 0.5, 1] as a query returning [2.0, 10.0, 18.0] [3][5].

**Mitigation**: Use RRF for general-purpose systems; use learned fusion weights for high-stakes applications.

### Latency Multiplication

Executing both BM25 and vector queries doubles retrieval latency. Re-ranking adds additional overhead [1][2].

**Mitigation**: Execute retrievers in parallel. Use efficient re-rankers (cross-encoders add 50-100ms; lightweight learned rankers add 5-10ms).

## Recommended Practices

1. **Default to RRF with k=60**: Most robust general-purpose fusion [2][5][6]
2. **Execute retrievers in parallel**: Network latency often dominates compute time
3. **Over-retrieve before fusion**: Fetch top-100 from each retriever, fuse to top-20
4. **Monitor per-modality precision**: Track which retriever contributes to final results
5. **Consider learned fusion for high-value queries**: Train lightweight rankers on click data

## Sources

[1] https://superlinked.com/vectorhub/articles/optimizing-rag-with-hybrid-search-reranking
[2] https://dev.to/kuldeep_paul/advanced-rag-from-naive-retrieval-to-hybrid-search-and-re-ranking-4km3
[3] https://www.geeksforgeeks.org/nlp/what-is-bm25-best-matching-25-algorithm/
[4] https://www.sourcely.net/resources/bm25-and-its-role-in-document-relevance-scoring
[5] https://www.tigerdata.com/blog/introducing-pg_textsearch-true-bm25-ranking-hybrid-retrieval-postgres
[6] https://weaviate.io/blog/hybrid-search-explained
[7] https://learn.microsoft.com/en-us/azure/search/semantic-search-overview
[8] https://milvus.io/ai-quick-reference/how-do-hybrid-approaches-combine-fulltext-and-vector-search
