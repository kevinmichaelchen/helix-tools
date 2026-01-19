# Requirements

This document defines requirements for `ix-helixdb-ops`.

`ix-helixdb-ops` provides small, reusable helpers for common HelixDB graph
storage patterns used across Ixchel.

## 1. Storage Writes

### US-001: Write nodes

| ID       | Acceptance Criterion                                                             |
| -------- | -------------------------------------------------------------------------------- |
| AC-001.1 | WHEN `put_node(storage, wtxn, node)` is called THE SYSTEM SHALL persist the node |
| AC-001.2 | THE SYSTEM SHALL use HelixDB’s node key encoding for storage                     |

### US-002: Write edges

| ID       | Acceptance Criterion                                                             |
| -------- | -------------------------------------------------------------------------------- |
| AC-002.1 | WHEN `put_edge(storage, wtxn, edge)` is called THE SYSTEM SHALL persist the edge |
| AC-002.2 | THE SYSTEM SHALL update in-edge and out-edge adjacency indices                   |

## 2. Secondary Indices

### US-003: Update indices

| ID       | Acceptance Criterion                                                                                      |
| -------- | --------------------------------------------------------------------------------------------------------- |
| AC-003.1 | WHEN `update_secondary_indices(storage, wtxn, node)` is called THE SYSTEM SHALL update configured indices |
| AC-003.2 | THE SYSTEM SHALL index only properties present on the node                                                |

### US-004: Lookup by secondary index

| ID       | Acceptance Criterion                                                                                                            |
| -------- | ------------------------------------------------------------------------------------------------------------------------------- |
| AC-004.1 | WHEN `lookup_secondary_index(storage, rtxn, index_name, key)` is called THE SYSTEM SHALL return the matching node id if present |
| AC-004.2 | IF `index_name` does not exist THEN THE SYSTEM SHALL return `None`                                                              |

## 3. Adjacency Queries

### US-005: Neighbor listing

| ID       | Acceptance Criterion                                                                                                 |
| -------- | -------------------------------------------------------------------------------------------------------------------- |
| AC-005.1 | WHEN `outgoing_neighbors(storage, rtxn, node_id, label_hash)` is called THE SYSTEM SHALL return destination node ids |
| AC-005.2 | WHEN `incoming_neighbors(storage, rtxn, node_id, label_hash)` is called THE SYSTEM SHALL return source node ids      |
