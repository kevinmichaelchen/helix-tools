# LMDB vs RocksDB: Architectural Comparison

**Technical Report | Systems Engineering Audience**

## Context

Selecting an embedded key-value store requires understanding fundamental architectural tradeoffs. LMDB and RocksDB represent opposing design philosophies: LMDB uses a B+tree with memory-mapping; RocksDB uses a Log-Structured Merge Tree (LSM) with explicit buffering [2][5]. Neither is universally superior—the optimal choice depends on workload characteristics.

This report compares these databases across write amplification, compaction behavior, memory footprint, and failure recovery to inform technology selection.

## Core Mechanics

### LMDB: B+Tree with Memory-Mapping

LMDB stores data in a B+tree structure, memory-mapped directly into the process address space [1][5]. Reads access pages through the OS page cache without application-level buffering. Writes use copy-on-write semantics: modified pages are written to new locations, preserving consistency without write-ahead logging.

Key characteristics:

- **Single-writer, multi-reader** concurrency model
- **No background compaction** processes
- **Zero-copy reads** via memory mapping
- **Instant crash recovery** (no WAL replay)

### RocksDB: LSM Tree with Write Buffering

RocksDB buffers writes in an in-memory memtable, periodically flushing to sorted string table (SST) files on disk [2]. Background compaction merges SST files across levels, maintaining read performance at the cost of write amplification.

Key characteristics:

- **Concurrent readers and writers**
- **Background compaction** threads
- **Configurable memory budgets** for caching
- **Write-ahead logging** for durability

## Comparison Analysis

### Write Performance and Amplification

**RocksDB** excels at sustained write throughput. Benchmarks show 100,000+ writes/second under heavy load, compared to LMDB's lower write throughput [3].

However, RocksDB's LSM design incurs **write amplification**—data may be rewritten multiple times during compaction. Typical amplification factors range from 10-30x, meaning 1GB of user writes may generate 10-30GB of disk I/O [2].

**LMDB** avoids compaction entirely. Write amplification is limited to copy-on-write overhead (typically 2-3x for modified paths). However, single-writer serialization limits concurrent write throughput [1].

| Metric              | LMDB                  | RocksDB             |
| ------------------- | --------------------- | ------------------- |
| Write throughput    | Lower (single-writer) | Higher (concurrent) |
| Write amplification | 2-3x                  | 10-30x              |
| Compaction overhead | None                  | Significant         |

**Recommendation**: Choose RocksDB for write-heavy workloads; choose LMDB when write amplification must be minimized (e.g., SSD wear concerns).

### Read Performance

**LMDB** achieves exceptional read latency through memory-mapping. Benchmarks show 0.39ms average read latency compared to RocksDB's 0.79ms [3]. The difference stems from LMDB's zero-copy access—reads don't traverse application-level caches.

**RocksDB** read performance depends heavily on configuration. Block cache sizing, bloom filter configuration, and compaction state all affect latency. Well-tuned RocksDB can approach LMDB read performance, but requires significant configuration effort [1][2].

| Metric                   | LMDB       | RocksDB             |
| ------------------------ | ---------- | ------------------- |
| Read latency (p50)       | 0.39ms     | 0.79ms              |
| Configuration complexity | Minimal    | Extensive           |
| Cache management         | OS-managed | Application-managed |

**Recommendation**: Choose LMDB for read-heavy workloads or when configuration simplicity is valued.

### Memory Footprint and Resource Usage

**LMDB** uses remarkably few system resources. A single environment requires only 1-2 file descriptors regardless of database size [1]. Memory usage is controlled by the OS page cache—no application-level memory budgeting required.

**RocksDB** can consume thousands of file descriptors for large databases (one per SST file) [1]. Memory usage includes write buffers, block cache, bloom filters, and compaction buffers—all requiring explicit sizing.

| Resource                 | LMDB          | RocksDB             |
| ------------------------ | ------------- | ------------------- |
| File descriptors         | 1-2           | Thousands           |
| Memory management        | OS page cache | Application-managed |
| Configuration parameters | ~5            | 100+                |

**Recommendation**: Choose LMDB when file descriptor limits matter (e.g., many concurrent databases); choose RocksDB when fine-grained memory control is required.

### Failure Recovery

**LMDB** recovers instantly from crashes. The copy-on-write design ensures the database is always in a consistent state—either the old root or the new root is valid. No log replay required [1].

**RocksDB** requires WAL replay on startup after unclean shutdown. Recovery time depends on WAL size and memtable configuration. Well-configured systems recover in seconds; misconfigured systems may take minutes [2].

Both databases provide ACID guarantees when properly configured.

### Compaction Behavior

**LMDB** has no compaction. Deleted data is reclaimed when transactions referencing old pages complete. This simplifies operations but means database files don't shrink automatically.

**RocksDB** compaction runs continuously in the background, consuming CPU and I/O bandwidth [2]. Compaction can cause latency spikes if it falls behind write rate. However, it maintains predictable read performance and reclaims space automatically.

| Aspect                 | LMDB                      | RocksDB                         |
| ---------------------- | ------------------------- | ------------------------------- |
| Background I/O         | None                      | Continuous                      |
| Space reclamation      | Manual (copy to new file) | Automatic                       |
| Latency predictability | High                      | Variable (compaction-dependent) |

## Decision Framework

| Workload Characteristic              | Recommended Choice |
| ------------------------------------ | ------------------ |
| Read-heavy (>90% reads)              | LMDB               |
| Write-heavy (>50% writes)            | RocksDB            |
| Latency-sensitive                    | LMDB               |
| Throughput-sensitive                 | RocksDB            |
| Simple operations                    | LMDB               |
| Complex queries (prefix scans, etc.) | RocksDB            |
| SSD wear concerns                    | LMDB               |
| Large datasets (>100GB)              | RocksDB            |
| Resource-constrained                 | LMDB               |

## Sources

[1] https://news.ycombinator.com/item?id=44836463 - LMDB vs RocksDB Discussion
[2] https://www.simplyblock.io/glossary/what-is-rocksdb/
[3] https://surrealdb.com/blog/beginning-our-benchmarking-journey
[4] https://docs.hypermode.com/badger/design
[5] https://www.explo.co/blog/embedded-sql-databases
[6] https://cpp.libhunt.com/lmdb-alternatives
[7] https://github.com/lmdbjava/benchmarks
