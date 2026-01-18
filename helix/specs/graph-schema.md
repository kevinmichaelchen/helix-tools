# Graph Schema Specification (Knowledge + Attribution)

HelixDB stores a **typed property graph** plus **chunk-level vectors**. The schema focuses on durable knowledge artifacts with lightweight agent/session attribution. Run logs and code-surface indexing are deferred.

## Node Labels

```
DECISION, ISSUE, IDEA, REPORT, SOURCE, CITATION
AGENT, SESSION
```

## Common Node Properties

| Property          | Type     | Required | Indexed | Notes                              |
| ----------------- | -------- | -------- | ------- | ---------------------------------- |
| `id`              | String   | Yes      | Yes*    | `{prefix}-{hex}`                   |
| `title`           | String   | Yes      | No      |                                    |
| `created_at`      | DateTime | Yes      | Yes     |                                    |
| `updated_at`      | DateTime | Yes      | Yes     |                                    |
| `created_by`      | String   | No       | Yes     | Agent/human handle                 |
| `created_by_type` | String   | No       | Yes     | `human                             |
| `agent_id`        | String   | No       | Yes     | For attribution                    |
| `session_id`      | String   | No       | Yes     | Groups related actions             |
| `tags`            | String[] | No       | Yes     |                                    |
| `external_ref`    | String   | No       | Yes     |                                    |
| `confidence`      | Float    | No       | Yes     | 0..1 for speculative items         |
| `content_hash`    | String   | No       | No      | SHA/BLAKE3 of canonical content    |
| `vector_ids`      | String[] | No       | Yes     | Chunk vectors associated with node |

`*` All IDs are unique across the graph.

## Node-Specific Properties

- **DECISION**: `status`, `date`, `deciders[]`
- **ISSUE**: `status`, `issue_type`, `priority`, `assignees[]`, `parent_id`, `estimated_minutes`, `closed_at`, `closed_reason`
- **IDEA**: `status`, `champion`, `effort`, `impact`
- **REPORT**: `status`, `report_type`, `period_start`, `period_end`, `incident_date`
- **SOURCE**: `source_type`, `url`, `authors[]`, `published_date`, `publisher`, `doi`, `isbn`, `archived_at`, `local_path`
- **CITATION**: `quote`, `page`, `timestamp`, `is_paraphrase`, `from_source`
- **AGENT**: `kind`, `model`, `vendor`, `capabilities[]`, `contact`
- **SESSION**: `scope`, `participants[]`, `intent`, `outcome`, `started_at`, `ended_at`
- **RUN/PLAN/PATCH/SNAPSHOT/FILE/SYMBOL/TEST**: Deferred (future extensions)

## Edge Types

| Type           | Description                            | Notes                                  |
| -------------- | -------------------------------------- | -------------------------------------- |
| `RELATES_TO`   | Generic association                    | Bidirectional semantics                |
| `BLOCKS`       | A must complete before B               | Strict, cycles disallowed              |
| `DEPENDS_ON`   | A requires B                           |                                        |
| `SUPERSEDES`   | A replaces B                           | Implies B is obsolete                  |
| `AMENDS`       | A modifies B                           |                                        |
| `EVOLVES_INTO` | A became B                             |                                        |
| `SPAWNS`       | A created B                            | Sessions/Decisions spawning work       |
| `IMPLEMENTS`   | A is implementation of B               | Issues → Decisions                     |
| `ADDRESSES`    | A responds to/resolves B               |                                        |
| `SUMMARIZES`   | A condenses B                          | Reports summarizing issues/decisions   |
| `CITES`        | A references B                         | Reports/Decisions → Sources/Citations  |
| `QUOTES`       | A directly excerpts B                  | Citation → Source                      |
| `SUPPORTS`     | Evidence for B                         | Citation/Source → Decision/Idea/Report |
| `CONTRADICTS`  | Evidence against B                     | Citation/Source → Decision/Idea/Report |
| `USED_IN`      | A is used in B                         | Citation → Report/Decision             |
| `RECOMMENDS`   | A proposes/requests B                  | Reports/Agents → Decisions             |
| `PLAN_FOR`     | Plan targeting B                       | Deferred                               |
| `EXECUTES`     | Agent executes Run                     | Deferred                               |
| `BELONGS_TO`   | Run/Patch/Snapshot inside Session      | Deferred                               |
| `PRODUCES`     | A emits/creates B                      | Deferred                               |
| `AFFECTS`      | Patch/Run/Issue touches code           | Deferred                               |
| `COVERS`       | Test covers File/Symbol                | Deferred                               |
| `FAILS_ON`     | Test failing on Patch/Run              | Deferred                               |
| `OBSERVES`     | Report records observation of B        | → Session                              |
| `DUPLICATE_OF` | A is duplicate of B                    | Symmetric, no implied direction        |
| `DERIVES_FROM` | A derived/port of B                    |                                        |
| `CLAIMS`       | Agent reserves ownership of B          | → Issue                                |
| `RESERVES`     | Issue reserves File/Symbol             | Deferred                               |
| `OWNED_BY`     | Resource belongs to Agent/Plan/Team    | Deferred                               |
| `TESTED_BY`    | Patch/Decision/Issue validated by Test | Deferred                               |
| `BASED_ON`     | Run/Plan derived from knowledge        | Deferred                               |

## Edge Properties

| Property           | Type     | Description                                  |
| ------------------ | -------- | -------------------------------------------- |
| `created_at`       | DateTime | When the relationship was created            |
| `created_by`       | String   | Who created it                               |
| `confidence`       | Float    | 0..1 strength (useful for speculative links) |
| `lease_expires_at` | DateTime | For `CLAIMS` edges to prevent deadlocks      |
| `note`             | String   | Optional annotation                          |

## Validity Rules (Strict Mode)

Rules are evaluated `from --edge--> to`. Wildcards (`*`) represent any label.

- `RELATES_TO`: `* -> *`
- `BLOCKS`/`DEPENDS_ON`: Issue → Issue
- `SUPERSEDES`/`AMENDS`: Decision → Decision
- `EVOLVES_INTO`: Idea → Decision|Issue
- `SPAWNS`: Decision|Session → Issue
- `IMPLEMENTS`: Issue → Decision
- `ADDRESSES`: Decision|Report → Issue|Idea
- `SUMMARIZES`: Report → Issue|Decision|Session
- `CITES`/`QUOTES`: Report|Decision → Source|Citation; Citation → Source (quotes)
- `SUPPORTS`/`CONTRADICTS`: Citation|Source → Decision|Idea|Report
- `USED_IN`: Citation → Report|Decision
- `RECOMMENDS`: Report|Agent → Decision|Issue
- `PLAN_FOR`, `EXECUTES`, `BELONGS_TO`, `PRODUCES`, `AFFECTS`, `COVERS`, `FAILS_ON`, `RESERVES`, `OWNED_BY`, `TESTED_BY`, `BASED_ON`: Deferred
- `OBSERVES`: Report → Session|Issue
- `DUPLICATE_OF`/`DERIVES_FROM`: within same family (Issue/Idea/Source)
- `CLAIMS`: Agent → Issue

Permissive/custom modes can widen these sets (see `extensibility.md`).

## Indices

- **Unique**: `id`
- **Secondary**: `uuid`, `vector_ids`, `status`, `created_by`, `tags`, `assignees`, `priority`, `parent_id`, `agent_id`, `session_id`, `report_type`, `issue_type`
- **Graph**: outgoing/incoming adjacency lists, plus `CLAIMS` reverse indices for collision detection

## Vector Index & Chunking

```rust
VectorConfig {
    dimensions: 384,
    distance_metric: Cosine,
    hnsw: HnswConfig { m: 16, ef_construction: 200, ef_search: 64 },
    chunking: ChunkingConfig {
        max_tokens: 512,
        overlap: 64,
    },
}
```

- Each chunk stores `vector_id`, `node_id`, `span` (`start_line`, `end_line` or heading path).
- Centroid vectors per node are stored for coarse recall; rerank on chunk payload.

## Query Patterns (Key Paths for Agents)

- **Find available work without collisions**:
  ```
  MATCH (i:ISSUE {status:'open'})
  WHERE NOT EXISTS { MATCH (:AGENT)-[c:CLAIMS]->(i) WHERE c.lease_expires_at > now() }
  RETURN i LIMIT 50
  ```

- **Find evidence for/against a decision**:
  ```
  MATCH (c:CITATION)-[rel:SUPPORTS|CONTRADICTS]->(d:DECISION {id:$id})
  RETURN c, type(rel) AS stance, rel.confidence
  ORDER BY rel.confidence DESC
  ```

## Data Integrity

- Unique IDs across all labels.
- Cycle detection on `BLOCKS` and `SUPERSEDES`.
- Lease expiry enforcement on `CLAIMS`.
- `confidence` must be provided for auto-created links from agents.
