# Tasks

**Status:** Phase 1-4 Complete, Phase 5 Complete\
**Updated:** 2026-01-06

## Phase 1: Crate Skeleton ✅

- [x] Create `shared/helix-daemon` crate with `lib.rs`
- [x] Define protocol structs (Request, Response, Command enum)
- [x] Add serde serialization + validation helpers

## Phase 2: IPC Server ✅

- [x] Implement Unix socket server (`~/.helix/run/helixd.sock`)
- [ ] Implement Windows named pipe server (placeholder or feature-gated)
- [x] Parse JSON line messages with size limits
- [x] Route commands to handlers

## Phase 3: Queue + Locks ✅

- [x] Implement per `{repo_root, tool}` queue
- [x] Coalesce duplicate `enqueue_sync` requests
- [x] Add per-repo writer lock abstraction (via SyncQueue namespacing)
- [x] Track sync states (queued, running, done, error)

## Phase 4: helixd Binary ✅

- [x] Add `helixd` binary entrypoint
- [x] Implement auto-start + retry behavior for CLI clients
- [x] Add idle timeout configuration (default 5 min, `--idle-timeout` flag)
- [x] Implement `status`, `ping`, `shutdown`

## Phase 5: Tests ✅

- [x] Protocol round-trip tests
- [x] Queue coalescing tests
- [x] `wait_sync` timeout tests
- [x] Multi-repo lock isolation tests
- [x] Integration tests (20 tests total)

## Phase 6: Sync Worker (PLANNED)

> The daemon currently stubs `enqueue_sync` and `wait_sync` with "not yet implemented".
> This phase adds actual sync execution.

- [ ] Implement sync worker thread/task pool
- [ ] Call tool-specific sync logic (e.g., ixchel indexing)
- [ ] Handle sync errors gracefully (retry, backoff)
- [ ] Emit progress updates for long-running syncs

## Notes

- Windows named pipe support deferred (Unix socket works on macOS/Linux)
- Sync worker is the main remaining piece for end-to-end functionality
- Client auto-starts daemon on first connection attempt
