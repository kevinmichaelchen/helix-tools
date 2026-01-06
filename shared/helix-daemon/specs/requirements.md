# Requirements

This document defines requirements for `helix-daemon` using EARS notation.

## EARS Notation Reference

| Pattern | Template |
|---------|----------|
| Ubiquitous | THE SYSTEM SHALL `<action>` |
| Event-driven | WHEN `<trigger>` THE SYSTEM SHALL `<action>` |
| State-driven | WHILE `<state>` THE SYSTEM SHALL `<action>` |
| Optional | WHERE `<feature>` THE SYSTEM SHALL `<action>` |
| Complex | IF `<condition>` THEN THE SYSTEM SHALL `<action>` |

---

## 1. Daemon Identity

### US-001: Global Per-User Daemon

**As a** helix-tools user  
**I want** a single daemon per user  
**So that** all tools can share background sync and locking

| ID | Acceptance Criterion |
|----|---------------------|
| AC-001.1 | THE SYSTEM SHALL run one daemon per user session |
| AC-001.2 | THE SYSTEM SHALL namespace requests by `{repo_root, tool}` |
| AC-001.3 | THE SYSTEM SHALL enforce per-repo writer locks |

---

## 2. IPC Transport

### US-002: Local Socket IPC

**As a** CLI client  
**I want** local socket IPC  
**So that** requests are fast and secure

| ID | Acceptance Criterion |
|----|---------------------|
| AC-002.1 | THE SYSTEM SHALL listen on Unix socket `~/.helix/run/helixd.sock` |
| AC-002.2 | THE SYSTEM SHALL use a named pipe on Windows |
| AC-002.3 | THE SYSTEM SHALL reject non-local connections |

---

## 3. IPC Protocol

### US-003: Versioned JSON Line Protocol

**As a** CLI client  
**I want** a stable, versioned protocol  
**So that** tools can evolve without breaking compatibility

| ID | Acceptance Criterion |
|----|---------------------|
| AC-003.1 | THE SYSTEM SHALL accept JSON objects delimited by newlines |
| AC-003.2 | THE SYSTEM SHALL include a `version` field in all messages |
| AC-003.3 | THE SYSTEM SHALL return one response per request |
| AC-003.4 | IF the version is unsupported THEN THE SYSTEM SHALL return `incompatible_version` |

---

## 4. Commands

### US-004: Required Commands

**As a** CLI client  
**I want** a minimal set of daemon commands  
**So that** I can enqueue work and wait for completion

| ID | Acceptance Criterion |
|----|---------------------|
| AC-004.1 | THE SYSTEM SHALL support `ping` |
| AC-004.2 | THE SYSTEM SHALL support `enqueue_sync` |
| AC-004.3 | THE SYSTEM SHALL support `wait_sync` |
| AC-004.4 | THE SYSTEM SHALL support `status` |
| AC-004.5 | THE SYSTEM SHALL support `shutdown` (dev/test only) |

---

## 5. Queueing and Coalescing

### US-005: Per-Repo Queues

**As a** CLI client  
**I want** work to be queued per repo  
**So that** multiple invocations do not conflict

| ID | Acceptance Criterion |
|----|---------------------|
| AC-005.1 | THE SYSTEM SHALL maintain one queue per `{repo_root, tool}` |
| AC-005.2 | THE SYSTEM SHALL coalesce duplicate `enqueue_sync` requests |
| AC-005.3 | THE SYSTEM SHALL allow at most one active writer per repo |

---

## 6. Lifecycle

### US-006: Auto-Start and Idle Shutdown

**As a** CLI client  
**I want** the daemon to auto-start and exit when idle  
**So that** the system stays lightweight

| ID | Acceptance Criterion |
|----|---------------------|
| AC-006.1 | WHEN the socket is missing THE SYSTEM SHALL allow clients to start the daemon |
| AC-006.2 | THE SYSTEM SHALL exit after `idle_timeout_ms` with no active queues |
| AC-006.3 | IF shutdown is requested in production THEN THE SYSTEM SHALL reject it |
