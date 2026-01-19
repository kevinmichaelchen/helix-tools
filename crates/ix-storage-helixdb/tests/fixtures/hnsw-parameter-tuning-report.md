# HNSW Parameter Tuning: Recall, Latency, and Memory Tradeoffs

**Technical Report | Systems Engineering Audience**

## Context

Hierarchical Navigable Small World (HNSW) graphs have become the dominant indexing structure for approximate nearest neighbor (ANN) search in production vector databases [1][2]. Unlike tree-based indexes, HNSW achieves sub-linear query complexity through greedy traversal of a multi-layer proximity graph.

This report examines the three critical HNSW parameters—M, efConstruction, and efSearch—and their impact on recall, latency, and memory consumption. Understanding these tradeoffs is essential for tuning vector search systems to meet application-specific SLAs.

## Core Mechanics

### Graph Structure

HNSW organizes vectors across multiple layers, with each layer containing progressively fewer nodes [1][6]. Layer 0 (bottom) contains all vectors; higher layers contain exponentially fewer, forming a coarse-to-fine navigation hierarchy. Queries begin at the top layer and greedily descend, narrowing the search region at each level.

Each vector maintains bidirectional connections to its nearest neighbors within each layer. The graph exhibits "small world" properties—most node pairs are reachable through short paths—enabling efficient greedy search [1][3].

### Parameter Definitions

| Parameter          | Description                                | Typical Range  |
| ------------------ | ------------------------------------------ | -------------- |
| **M**              | Maximum connections per node per layer     | 4–64 [2][4][7] |
| **efConstruction** | Candidate pool size during index building  | 64–500 [2][4]  |
| **efSearch**       | Candidate pool size during query execution | 16–512 [2][5]  |

### Tradeoff Analysis

**M (Max Connections)**

Higher M values create denser graphs with more navigation pathways [2][7]. This improves recall by reducing "dead end" scenarios where greedy search terminates in local minima. However, each additional edge increases memory consumption proportionally, and denser graphs slow both indexing and query traversal.

| M Value | Recall | Index Time | Query Time | Memory    |
| ------- | ------ | ---------- | ---------- | --------- |
| 8       | ~85%   | Low        | Fast       | Low       |
| 16      | ~92%   | Medium     | Medium     | Medium    |
| 32      | ~96%   | High       | Slower     | High      |
| 64      | ~98%   | Very High  | Slowest    | Very High |

**efConstruction**

This parameter controls how thoroughly HNSW searches for optimal neighbors during index construction [2][4][7]. Higher values produce better-connected graphs (higher recall) at the cost of significantly longer build times. Once the index is built, efConstruction has no runtime impact.

For datasets requiring high recall, values of 200–500 are common. For rapid iteration during development, 64–100 may suffice.

**efSearch**

Unlike efConstruction, efSearch is tunable per query without rebuilding the index [2][5][7]. Higher values expand the candidate pool during search, improving recall at the cost of query latency. This enables dynamic quality/speed tradeoffs based on query priority.

Production systems often expose efSearch as a query parameter, allowing applications to specify precision requirements per request.

## Operational Risks

### Memory Growth Under High M

Each node stores M×2 connections on average (bidirectional edges). For 1 billion vectors with M=32, edge storage alone exceeds 256GB [2]. This often dominates total index size, exceeding the vector data itself.

**Mitigation**: Use M=12-16 for large-scale deployments; increase efSearch to compensate for recall loss.

### Index Build Time Scaling

efConstruction=500 can result in build times 10-20x longer than efConstruction=100 [4][7]. For datasets exceeding 100M vectors, full rebuilds may require hours or days.

**Mitigation**: Use parallel index construction where supported. Consider incremental indexing strategies for append-heavy workloads.

### Recall Degradation Under Distribution Shift

HNSW graphs are optimized for the data distribution at build time. If query vectors differ significantly from indexed vectors (e.g., different embedding model), recall may degrade substantially [3].

**Mitigation**: Monitor recall metrics in production. Rebuild indexes when embedding models change.

## Recommended Practices

1. **Start with M=16, efConstruction=200, efSearch=100**: This configuration achieves 95%+ recall for most datasets [2][6]
2. **Tune efSearch first**: It's adjustable at runtime and provides the most direct recall/latency tradeoff
3. **Profile memory before scaling M**: Calculate expected edge storage before committing to high M values
4. **Benchmark with production query distribution**: Synthetic benchmarks may not reflect real recall characteristics
5. **Implement recall monitoring**: Track precision@k against ground truth samples in production

## Sources

[1] https://milvus.io/blog/understand-hierarchical-navigable-small-worlds-hnsw-for-vector-search.md
[2] https://redis.io/blog/how-hnsw-algorithms-can-improve-search/
[3] https://www.emergentmind.com/topics/hierarchical-navigable-small-world-hnsw-graph
[4] https://www.elastic.co/search-labs/blog/hnsw-graph
[5] https://www.youtube.com/watch?v=fCUy1DZZspM - HNSW Parameter Tuning Deep Dive
[6] https://en.wikipedia.org/wiki/Hierarchical_navigable_small_world
[7] https://qdrant.tech/course/essentials/day-2/what-is-hnsw/
[8] https://developer.ibm.com/tutorials/awb-enhancing-retrieval-hnsw-rag/
