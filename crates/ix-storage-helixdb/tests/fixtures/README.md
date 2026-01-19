# ix-storage-helixdb e2e fixtures

Realistic report-style fixtures for the ix-storage-helixdb e2e sync/search test.
These are intentionally long-form to exercise embeddings and semantic search.

## Files

- `lmdb-internals-report.md`: LMDB architecture, MVCC, CoW, SWMR
- `lmdb-mapsize-incident-review.md`: map size incident postmortem
- `lmdb-vs-rocksdb-comparison.md`: comparison report for LMDB vs RocksDB
- `hnsw-parameter-tuning-report.md`: HNSW tuning tradeoffs
- `hybrid-search-fusion-report.md`: BM25 + vector fusion strategies
- `embedding-lifecycle-report.md`: cold starts, dimension mismatch, migrations

## Suggested queries

- `reader lock table` (LMDB internals)
- `map size` (LMDB incident review)
- `efConstruction` (HNSW report)
- `reciprocal rank fusion` (hybrid search report)
