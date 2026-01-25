//! ix-daemon: Global per-user daemon for Ixchel.
//!
//! Provides IPC, sync queueing, and single-writer enforcement across repos and tools.
//!
//! # Protocol
//!
//! All messages are UTF-8 JSON lines over Unix socket (`~/.ixchel/run/ixcheld.sock`).
//! See `specs/design.md` for the full protocol specification.

mod client;
mod queue;
mod server;
mod watcher;
mod worker;

pub use client::Client;
pub use queue::{QueueKey, SyncJob, SyncQueue};
pub use server::Server;
pub use watcher::{RepoWatcher, WatchError, WatchEvent, WatchEventKind};
pub use worker::SyncWorker;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Protocol version. Increment on breaking changes.
pub const PROTOCOL_VERSION: u32 = 1;

/// Default socket path (Unix).
pub const DEFAULT_SOCKET_PATH: &str = "~/.ixchel/run/ixcheld.sock";

/// Default idle timeout before daemon shuts down (milliseconds).
pub const DEFAULT_IDLE_TIMEOUT_MS: u64 = 300_000; // 5 minutes

// ============================================================================
// Errors
// ============================================================================

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Incompatible protocol version: expected {expected}, got {got}")]
    IncompatibleVersion { expected: u32, got: u32 },

    #[error("Repository not found: {0}")]
    RepoNotFound(String),

    #[error("Timeout waiting for sync: {0}")]
    Timeout(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Error codes for protocol responses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    IncompatibleVersion,
    RepoNotFound,
    Timeout,
    InternalError,
}

impl From<&DaemonError> for ErrorCode {
    fn from(err: &DaemonError) -> Self {
        match err {
            DaemonError::InvalidRequest(_) => Self::InvalidRequest,
            DaemonError::IncompatibleVersion { .. } => Self::IncompatibleVersion,
            DaemonError::RepoNotFound(_) => Self::RepoNotFound,
            DaemonError::Timeout(_) => Self::Timeout,
            DaemonError::Internal(_) | DaemonError::Io(_) | DaemonError::Json(_) => {
                Self::InternalError
            }
        }
    }
}

// ============================================================================
// Protocol: Request
// ============================================================================

/// Request envelope sent from CLI to daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Protocol version.
    pub version: u32,

    /// Unique request ID for correlation.
    pub id: String,

    /// Absolute path to repository root.
    pub repo_root: String,

    /// Tool name (e.g., "decisions", "issues", "reports").
    pub tool: String,

    /// Command to execute.
    pub command: Command,
}

impl Request {
    /// Create a new request with a random ID.
    pub fn new(repo_root: impl Into<String>, tool: impl Into<String>, command: Command) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            id: uuid::Uuid::new_v4().to_string(),
            repo_root: repo_root.into(),
            tool: tool.into(),
            command,
        }
    }
}

/// Commands supported by the daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "payload", rename_all = "snake_case")]
pub enum Command {
    /// Health check.
    Ping,

    /// Enqueue a sync job.
    EnqueueSync(EnqueueSyncPayload),

    /// Wait for a sync job to complete.
    WaitSync(WaitSyncPayload),

    /// Query daemon status.
    Status(StatusPayload),

    /// Start watching a repository for changes.
    Watch(WatchPayload),

    /// Stop watching a repository.
    Unwatch(UnwatchPayload),

    /// Shutdown the daemon (dev/test only).
    Shutdown(ShutdownPayload),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnqueueSyncPayload {
    /// Directory to sync (e.g., ".ixchel/decisions").
    pub directory: String,

    /// Force full resync even if no changes detected.
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitSyncPayload {
    /// Sync ID returned by `enqueue_sync`.
    pub sync_id: String,

    /// Timeout in milliseconds.
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
}

const fn default_timeout_ms() -> u64 {
    30_000
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusPayload {
    /// Filter by repo root (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root: Option<String>,

    /// Filter by tool (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WatchPayload {
    /// Repository root to watch (uses `repo_root` from request if empty).
    #[serde(default)]
    pub repo_root: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UnwatchPayload {
    /// Repository root to stop watching (uses `repo_root` from request if empty).
    #[serde(default)]
    pub repo_root: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShutdownPayload {
    /// Reason for shutdown.
    #[serde(default)]
    pub reason: String,
}

// ============================================================================
// Protocol: Response
// ============================================================================

/// Response envelope sent from daemon to CLI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Protocol version.
    pub version: u32,

    /// Request ID this response correlates to.
    pub id: String,

    /// Response status.
    #[serde(flatten)]
    pub result: ResponseResult,
}

impl Response {
    /// Create a success response.
    pub fn ok(id: impl Into<String>, payload: ResponsePayload) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            id: id.into(),
            result: ResponseResult::Ok { payload },
        }
    }

    /// Create an error response.
    pub fn error(id: impl Into<String>, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            id: id.into(),
            result: ResponseResult::Error {
                error: ErrorInfo {
                    code,
                    message: message.into(),
                },
            },
        }
    }

    /// Create an error response from a [`DaemonError`].
    pub fn from_error(id: impl Into<String>, err: &DaemonError) -> Self {
        Self::error(id, ErrorCode::from(err), err.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ResponseResult {
    Ok { payload: ResponsePayload },
    Error { error: ErrorInfo },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: ErrorCode,
    pub message: String,
}

/// Response payloads for each command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponsePayload {
    Ping(PingResponse),
    EnqueueSync(EnqueueSyncResponse),
    WaitSync(WaitSyncResponse),
    Status(StatusResponse),
    Watch(WatchResponse),
    Unwatch(UnwatchResponse),
    Shutdown(ShutdownResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResponse {
    pub daemon_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnqueueSyncResponse {
    pub sync_id: String,
    pub queued_at_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitSyncResponse {
    pub sync_id: String,
    pub state: SyncState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<SyncStats>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncState {
    Queued,
    Running,
    Done,
    Error,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SyncStats {
    pub files_scanned: u64,
    pub files_updated: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub queues: Vec<QueueInfo>,
    pub uptime_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInfo {
    pub repo_root: String,
    pub tool: String,
    pub pending: u32,
    pub active: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchResponse {
    /// Repository root being watched.
    pub repo_root: String,
    /// Whether watching was newly started (false if already watching).
    pub started: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnwatchResponse {
    /// Repository root that was unwatched.
    pub repo_root: String,
    /// Whether watching was actually stopped (false if wasn't watching).
    pub stopped: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ShutdownResponse {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = Request::new("/path/to/repo", "decisions", Command::Ping);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"version\":1"));
        assert!(json.contains("\"command\":\"ping\""));
    }

    #[test]
    fn test_enqueue_sync_request() {
        let req = Request::new(
            "/path/to/repo",
            "decisions",
            Command::EnqueueSync(EnqueueSyncPayload {
                directory: ".ixchel/decisions".to_string(),
                force: false,
            }),
        );
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"command\":\"enqueue_sync\""));
        assert!(json.contains("\".ixchel/decisions\""));
    }

    #[test]
    fn test_response_ok_serialization() {
        let resp = Response::ok(
            "test-id",
            ResponsePayload::Ping(PingResponse {
                daemon_version: "0.1.0".to_string(),
            }),
        );
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"ok\""));
        assert!(json.contains("\"daemon_version\":\"0.1.0\""));
    }

    #[test]
    fn test_response_error_serialization() {
        let resp = Response::error("test-id", ErrorCode::RepoNotFound, "repo not found");
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"status\":\"error\""));
        assert!(json.contains("\"code\":\"repo_not_found\""));
    }

    #[test]
    fn test_request_roundtrip() {
        let req = Request::new(
            "/path/to/repo",
            "issues",
            Command::WaitSync(WaitSyncPayload {
                sync_id: "sync-123".to_string(),
                timeout_ms: 5000,
            }),
        );
        let json = serde_json::to_string(&req).unwrap();
        let parsed: Request = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.repo_root, "/path/to/repo");
        assert_eq!(parsed.tool, "issues");
    }

    #[test]
    fn test_sync_state_values() {
        assert_eq!(
            serde_json::to_string(&SyncState::Queued).unwrap(),
            "\"queued\""
        );
        assert_eq!(
            serde_json::to_string(&SyncState::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(serde_json::to_string(&SyncState::Done).unwrap(), "\"done\"");
        assert_eq!(
            serde_json::to_string(&SyncState::Error).unwrap(),
            "\"error\""
        );
    }
}
