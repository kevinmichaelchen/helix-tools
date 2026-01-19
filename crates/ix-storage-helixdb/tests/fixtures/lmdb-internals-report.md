# LMDB Internals: Memory-Mapped B+Tree Design

**Technical Report | Systems Engineering Audience**

## Context

LMDB (Lightning Memory-Mapped Database) is an embedded key-value store that achieves exceptional read performance through its unique combination of memory-mapping, B+tree indexing, and copy-on-write semantics [1]. Unlike LSM-tree databases, LMDB provides instant reads without compaction overhead, making it ideal for read-heavy workloads with moderate write requirements.

This report examines LMDB's core architecture with focus on memory-mapping mechanics, concurrency model, and operational considerations for production deployments.

## Core Mechanics

### Memory-Mapped Architecture

LMDB memory-maps the entire database file directly into the process address space, enabling the operating system to handle caching transparently [1][2]. Reads bypass application-level buffering entirely—the CPU accesses database pages directly from the OS page cache. This eliminates serialization overhead and enables zero-copy access patterns.

The B+tree structure uses fixed-size pages (typically 4KB, matching OS page size) that remain **immutable after commit** [1][3]. Each transaction sees a consistent snapshot via root pointers stored in a metadata page. Writers create new page versions through copy-on-write (CoW), leaving existing pages intact for concurrent readers.

### Single-Writer Multi-Reader (SWMR) Model

LMDB enforces strict concurrency semantics:

- **Single writer**: One transaction holds exclusive write access at any time. The writer briefly acquires a global lock only during commit to atomically update the metadata page with the new B+tree root pointer [3][4].
- **Multiple readers**: Concurrent read transactions bind to historical tree roots via MVCC. Readers require no locks—they simply pin a transaction ID that references an immutable snapshot [1][4].

The **reader lock table** in LMDB's lock file tracks active reader transaction IDs [3][4]. Writers cannot reclaim pages still referenced by active readers. This mechanism prevents use-after-free scenarios but introduces a critical operational risk: stale readers can block page reclamation indefinitely.

### Copy-on-Write Semantics

Write operations proceed as follows:

1. Traverse the memory-mapped B+tree to locate target leaf/branch nodes
2. Allocate new pages and copy modified nodes (plus ancestor chain to root)
3. During commit, flush changes via `msync` or `fdatasync`
4. Atomically swap the root pointer in the metadata page [1][3]

Only modified paths (O(log N) pages) are copied per transaction. Garbage collection occurs when readers advance past old snapshots, allowing page reuse.

## Operational Risks

### Map Size Misconfiguration

The `MDB_MAPSIZE` environment parameter defines the maximum database size. This value must be set before opening the environment and cannot be increased while the database is open [4]. Undersizing leads to `MDB_MAP_FULL` errors under write load; oversizing on 32-bit systems wastes address space.

**Recommendation**: Set map size to 2x expected peak data size to accommodate CoW temporary space. Monitor free page count via `mdb_env_stat`.

### Reader Lock Table Exhaustion

Long-running read transactions or leaked cursors prevent page reclamation. In extreme cases, the database file grows unbounded while available free pages approach zero [3][4]. This failure mode is insidious—the database remains functional but disk usage spirals.

**Recommendation**: Implement transaction timeouts. Monitor `ms_last_pgno - ms_map_pages` ratio from environment stats.

### Memory Pressure and Page Faults

Memory-mapping does not guarantee pages remain resident. Under memory pressure, the OS may evict database pages, causing page faults on subsequent access. This manifests as latency spikes during reads [1][2].

**Recommendation**: Use `mlock` for latency-critical deployments. Monitor page fault rates via `/proc/[pid]/stat`.

## Recommended Practices

1. **Size the map conservatively**: Use 2-4x current data size with headroom for CoW operations
2. **Bound transaction lifetimes**: Implement explicit timeouts for read transactions
3. **Monitor free page ratio**: Alert when free pages drop below 20% of total
4. **Use `MDB_NORDAHEAD`**: Disable OS read-ahead for random access patterns
5. **Avoid nested transactions**: They complicate cursor management and increase leak risk

## Sources

[1] https://www.youtube.com/watch?v=6wV6dSQfz4c - LMDB: Lightning Memory-Mapped Database Technical Overview
[2] https://dl.acm.org/doi/full/10.1145/3719656 - Memory-Mapped Database Performance Analysis
[3] https://pkg.go.dev/go.etcd.io/bbolt - bbolt Go package documentation (LMDB-derived)
[4] https://moskud.xyz/posts/2025/02/27/mdbx1.html - MDBX Technical Deep Dive
[5] https://github.com/ankur-anand/unisondb - UnisonDB: LMDB-based storage engine
