# Tasks

## Phase 1: Crate Skeleton

- [ ] Create `shared/helix-daemon` crate with `lib.rs`
- [ ] Define protocol structs (request/response/envelope)
- [ ] Add serde serialization + validation helpers

## Phase 2: IPC Server

- [ ] Implement Unix socket server (`~/.helix/run/helixd.sock`)
- [ ] Implement Windows named pipe server (placeholder or feature-gated)
- [ ] Parse JSON line messages with size limits
- [ ] Route commands to handlers

## Phase 3: Queue + Locks

- [ ] Implement per `{repo_root, tool}` queue
- [ ] Coalesce duplicate `enqueue_sync` requests
- [ ] Add per-repo writer lock abstraction
- [ ] Track sync states (queued, running, done, error)

## Phase 4: helixd Binary

- [ ] Add `helixd` binary entrypoint
- [ ] Implement auto-start + retry behavior for CLI clients
- [ ] Add idle timeout configuration
- [ ] Implement `status`, `ping`, `shutdown`

## Phase 5: Tests

- [ ] Protocol round-trip tests
- [ ] Queue coalescing tests
- [ ] `wait_sync` timeout tests
- [ ] Multi-repo lock isolation tests
