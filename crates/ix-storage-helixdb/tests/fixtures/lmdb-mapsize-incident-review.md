# Incident Review: LMDB Map Size Exhaustion

**Internal Incident Summary | INC-2024-0847**

## Background

On 2024-11-14, the ixchel-graph service experienced write failures across three production nodes after deploying release v2.8.3. The service uses LMDB for local graph storage with a preconfigured map size of 10GB. The incident lasted 47 minutes and affected approximately 12,000 graph mutation requests.

The root cause was identified as map size exhaustion caused by increased write amplification from a new graph traversal feature, combined with stale reader transactions preventing page reclamation.

## Symptoms

**14:23 UTC** - Alerting triggered on `lmdb_write_errors` exceeding threshold (>10/min)

**14:25 UTC** - On-call engineer observed MDB_MAP_FULL errors in service logs:

```
mdb_put failed: MDB_MAP_FULL: Environment mapsize limit reached
```

**14:28 UTC** - Service health checks began failing; load balancer removed affected nodes from rotation

**14:31 UTC** - Customer reports of failed graph updates began arriving in support queue

Initial investigation focused on recent deployment changes. Release v2.8.3 introduced batch graph traversal, which increased write transaction sizes by approximately 3x.

## Root Cause

Two factors combined to exhaust the LMDB map size:

**1. Increased Write Amplification**

The new batch traversal feature modified more nodes per transaction than the previous implementation. Each modification triggers copy-on-write page allocation, temporarily doubling storage requirements for modified B+tree paths [1]. With larger transactions, peak space usage exceeded the 10GB limit despite steady-state data size remaining under 6GB.

**2. Stale Reader Transactions**

A connection pool bug introduced in v2.8.0 caused read transactions to remain open indefinitely under certain error conditions [3]. LMDB cannot reclaim pages still referenced by active readers, even if those readers are idle. Over 72 hours of operation, approximately 2GB of pages became unreclaimable due to 340+ stale reader transactions.

The combination—larger write transactions requiring more temporary space, plus reduced free page pool from stale readers—triggered map exhaustion during peak traffic.

## Fix

**Immediate mitigation (14:52 UTC)**:

- Restarted affected nodes to clear stale reader transactions
- Increased map size to 20GB via environment variable override

**Permanent fixes (deployed v2.8.4)**:

1. Patched connection pool to enforce transaction timeouts (30s max) [4]
2. Added monitoring for `ms_readers` count and oldest reader age
3. Increased default map size to 4x steady-state data size
4. Implemented free page ratio alerting (threshold: <25%)

## Lessons Learned

### What went well

- Alerting fired within 2 minutes of first failure
- Runbook for LMDB errors existed and was accurate
- Rolling restart with increased map size resolved the immediate issue

### What could improve

1. **Pre-deployment load testing did not simulate sustained operation**. The stale reader issue only manifested after ~72 hours of accumulated leaks. Future releases affecting LMDB should include extended soak tests.

2. **Map size was set based on current data size, not peak operational requirements**. LMDB's copy-on-write behavior means temporary space usage can exceed steady-state by 2-3x during large transactions [1][4]. Sizing guidelines have been updated.

3. **Reader transaction lifecycle was not monitored**. The connection pool bug existed for three releases without detection. Added `lmdb_stale_readers` metric and alert.

### Action items

| Action                                               | Owner         | Status      |
| ---------------------------------------------------- | ------------- | ----------- |
| Add soak test to release pipeline                    | Platform      | Completed   |
| Document LMDB sizing formula in runbook              | Platform      | Completed   |
| Alert on oldest reader age >60s                      | Observability | Completed   |
| Audit other services using LMDB for similar patterns | Architecture  | In Progress |

## Sources

[1] https://www.youtube.com/watch?v=6wV6dSQfz4c - LMDB Technical Overview
[2] https://dl.acm.org/doi/full/10.1145/3719656 - Memory-Mapped Database Performance
[3] https://pkg.go.dev/go.etcd.io/bbolt - bbolt documentation (MVCC reader semantics)
[4] https://moskud.xyz/posts/2025/02/27/mdbx1.html - MDBX/LMDB Operational Guide
