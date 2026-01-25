use crate::queue::SyncQueue;
use crate::watcher::{RepoWatcher, WatchEvent};
use crate::worker::SyncWorker;
use crate::{
    Command, DEFAULT_IDLE_TIMEOUT_MS, DaemonError, EnqueueSyncPayload, EnqueueSyncResponse,
    ErrorCode, PROTOCOL_VERSION, PingResponse, Request, Response, ResponsePayload,
    ShutdownResponse, StatusPayload, StatusResponse, UnwatchPayload, UnwatchResponse,
    WaitSyncPayload, WaitSyncResponse, WatchPayload, WatchResponse,
};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::broadcast;

const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Size of the watch event buffer.
const WATCH_EVENT_BUFFER_SIZE: usize = 256;

pub struct Server {
    socket_path: String,
    idle_timeout_ms: u64,
    watch_enabled: bool,
    start_time: Instant,
    shutdown_tx: broadcast::Sender<()>,
    queue: Arc<SyncQueue>,
    last_activity: Arc<AtomicU64>,
}

impl Server {
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self::with_options(socket_path, DEFAULT_IDLE_TIMEOUT_MS, false)
    }

    pub fn with_idle_timeout(socket_path: impl Into<String>, idle_timeout_ms: u64) -> Self {
        Self::with_options(socket_path, idle_timeout_ms, false)
    }

    pub fn with_options(
        socket_path: impl Into<String>,
        idle_timeout_ms: u64,
        watch_enabled: bool,
    ) -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            socket_path: socket_path.into(),
            idle_timeout_ms,
            watch_enabled,
            start_time: Instant::now(),
            shutdown_tx,
            queue: Arc::new(SyncQueue::new()),
            last_activity: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn expanded_socket_path(&self) -> String {
        expand_tilde(&self.socket_path)
    }

    fn touch_activity(&self) {
        #[allow(clippy::cast_possible_truncation)]
        let now = self.start_time.elapsed().as_millis() as u64;
        self.last_activity.store(now, Ordering::Relaxed);
    }

    fn is_idle(&self) -> bool {
        if self.idle_timeout_ms == 0 {
            return false;
        }

        #[allow(clippy::cast_possible_truncation)]
        let now = self.start_time.elapsed().as_millis() as u64;
        let last = self.last_activity.load(Ordering::Relaxed);
        now.saturating_sub(last) > self.idle_timeout_ms
    }

    // Event loop handling multiple async channels (commands, signals, idle timeout) requires
    // this complexity; splitting would obscure the unified state machine logic.
    #[allow(clippy::cognitive_complexity)]
    pub async fn run(&self) -> Result<(), DaemonError> {
        let socket_path = self.expanded_socket_path();

        if let Some(parent) = Path::new(&socket_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        if Path::new(&socket_path).exists() {
            tokio::fs::remove_file(&socket_path).await?;
        }

        let listener = UnixListener::bind(&socket_path)?;
        tracing::info!("ixcheld listening on {}", socket_path);

        if self.idle_timeout_ms > 0 {
            tracing::info!("Idle timeout: {}ms", self.idle_timeout_ms);
        }

        if self.watch_enabled {
            tracing::info!("File watching enabled");
        }

        self.touch_activity();

        let mut shutdown_rx = self.shutdown_tx.subscribe();
        let idle_check_interval = Duration::from_secs(10);

        // Spawn the sync worker
        let worker = SyncWorker::new(Arc::clone(&self.queue), self.shutdown_tx.subscribe());
        let worker_handle = tokio::spawn(async move {
            worker.run().await;
        });

        // Create watcher if enabled
        let (watcher, mut watch_rx) = if self.watch_enabled {
            let (watcher, rx) = RepoWatcher::new(WATCH_EVENT_BUFFER_SIZE);
            (Some(Arc::new(watcher)), Some(rx))
        } else {
            (None, None)
        };

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _)) => {
                            self.touch_activity();
                            let queue = Arc::clone(&self.queue);
                            let start_time = self.start_time;
                            let shutdown_tx = self.shutdown_tx.clone();
                            let last_activity = Arc::clone(&self.last_activity);
                            let watcher_clone = watcher.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, queue, start_time, shutdown_tx, last_activity, watcher_clone).await {
                                    tracing::error!("Connection error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::error!("Accept error: {}", e);
                        }
                    }
                }
                Some(event) = async {
                    match &mut watch_rx {
                        Some(rx) => rx.recv().await,
                        None => std::future::pending().await,
                    }
                } => {
                    self.handle_watch_event(event).await;
                }
                _ = shutdown_rx.recv() => {
                    tracing::info!("Shutdown signal received");
                    break;
                }
                () = tokio::time::sleep(idle_check_interval), if self.idle_timeout_ms > 0 => {
                    if self.is_idle() && self.queue.list_queues().await.is_empty() {
                        tracing::info!("Idle timeout reached, shutting down");
                        // Signal the worker to stop before breaking
                        let _ = self.shutdown_tx.send(());
                        break;
                    }
                }
            }
        }

        // Wait for worker to finish
        let _ = worker_handle.await;

        let _ = tokio::fs::remove_file(&socket_path).await;
        Ok(())
    }

    /// Handle a file watch event by enqueueing a sync.
    async fn handle_watch_event(&self, event: WatchEvent) {
        tracing::debug!(
            "Watch event: {:?} for {} in {}",
            event.kind,
            event.changed_path.display(),
            event.repo_root.display()
        );

        // Touch activity timer (file changes should prevent idle shutdown)
        self.touch_activity();

        // Enqueue a sync for this repository
        // The queue will coalesce multiple rapid changes
        let repo_root = event.repo_root.to_string_lossy().to_string();
        let (sync_id, is_new) = self
            .queue
            .enqueue(&repo_root, "watcher", ".ixchel", false)
            .await;

        if is_new {
            tracing::info!(
                "File change detected, enqueued sync {} for {}",
                sync_id,
                repo_root
            );
        }
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

async fn handle_connection(
    stream: tokio::net::UnixStream,
    queue: Arc<SyncQueue>,
    start_time: Instant,
    shutdown_tx: broadcast::Sender<()>,
    last_activity: Arc<AtomicU64>,
    watcher: Option<Arc<RepoWatcher>>,
) -> Result<(), DaemonError> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let bytes_read = reader.read_line(&mut line).await?;

        if bytes_read == 0 {
            break;
        }

        #[allow(clippy::cast_possible_truncation)]
        let now = start_time.elapsed().as_millis() as u64;
        last_activity.store(now, Ordering::Relaxed);

        if line.len() > MAX_MESSAGE_SIZE {
            let resp = Response::error("", ErrorCode::InvalidRequest, "Message too large");
            let json = serde_json::to_string(&resp)?;
            writer.write_all(json.as_bytes()).await?;
            writer.write_all(b"\n").await?;
            continue;
        }

        let response = match serde_json::from_str::<Request>(line.trim()) {
            Ok(req) => {
                if req.version == PROTOCOL_VERSION {
                    handle_command(&req, &queue, start_time, &shutdown_tx, watcher.as_deref()).await
                } else {
                    Response::error(
                        &req.id,
                        ErrorCode::IncompatibleVersion,
                        format!(
                            "Protocol version mismatch: expected {PROTOCOL_VERSION}, got {}",
                            req.version
                        ),
                    )
                }
            }
            Err(e) => Response::error("", ErrorCode::InvalidRequest, e.to_string()),
        };

        let json = serde_json::to_string(&response)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
    }

    Ok(())
}

#[allow(clippy::too_many_lines)]
async fn handle_command(
    req: &Request,
    queue: &SyncQueue,
    start_time: Instant,
    shutdown_tx: &broadcast::Sender<()>,
    watcher: Option<&RepoWatcher>,
) -> Response {
    match &req.command {
        Command::Ping => Response::ok(
            &req.id,
            ResponsePayload::Ping(PingResponse {
                daemon_version: env!("CARGO_PKG_VERSION").to_string(),
            }),
        ),

        Command::EnqueueSync(EnqueueSyncPayload { directory, force }) => {
            let (sync_id, _is_new) = queue
                .enqueue(&req.repo_root, &req.tool, directory, *force)
                .await;

            queue.get(&sync_id).await.map_or_else(
                || {
                    Response::error(
                        &req.id,
                        ErrorCode::InternalError,
                        "Failed to create sync job",
                    )
                },
                |job| {
                    Response::ok(
                        &req.id,
                        ResponsePayload::EnqueueSync(EnqueueSyncResponse {
                            sync_id,
                            queued_at_ms: job.queued_at_ms(),
                        }),
                    )
                },
            )
        }

        Command::WaitSync(WaitSyncPayload {
            sync_id,
            timeout_ms,
        }) => {
            let timeout = Duration::from_millis(*timeout_ms);

            match queue.wait(sync_id, timeout).await {
                Some(final_state) => {
                    let job_stats = queue.get(sync_id).await.and_then(|j| j.stats);
                    Response::ok(
                        &req.id,
                        ResponsePayload::WaitSync(WaitSyncResponse {
                            sync_id: sync_id.clone(),
                            state: final_state,
                            stats: job_stats,
                        }),
                    )
                }
                None => Response::error(
                    &req.id,
                    ErrorCode::Timeout,
                    format!("Timeout waiting for sync {sync_id}"),
                ),
            }
        }

        Command::Status(StatusPayload { .. }) => {
            #[allow(clippy::cast_possible_truncation)]
            let uptime_ms = start_time.elapsed().as_millis() as u64;
            let queues = queue.list_queues().await;
            Response::ok(
                &req.id,
                ResponsePayload::Status(StatusResponse { queues, uptime_ms }),
            )
        }

        Command::Watch(WatchPayload { repo_root }) => {
            let Some(watcher) = watcher else {
                return Response::error(
                    &req.id,
                    ErrorCode::InternalError,
                    "File watching is not enabled. Start daemon with --watch flag.",
                );
            };

            let target_repo = if repo_root.is_empty() {
                &req.repo_root
            } else {
                repo_root
            };

            let was_watching = watcher
                .watched_repos()
                .await
                .iter()
                .any(|p| p.to_string_lossy() == *target_repo);

            if !was_watching && let Err(e) = watcher.watch_repo(Path::new(target_repo)).await {
                return Response::error(
                    &req.id,
                    ErrorCode::InternalError,
                    format!("Failed to start watching: {e}"),
                );
            }

            Response::ok(
                &req.id,
                ResponsePayload::Watch(WatchResponse {
                    repo_root: target_repo.clone(),
                    started: !was_watching,
                }),
            )
        }

        Command::Unwatch(UnwatchPayload { repo_root }) => {
            let Some(watcher) = watcher else {
                return Response::error(
                    &req.id,
                    ErrorCode::InternalError,
                    "File watching is not enabled. Start daemon with --watch flag.",
                );
            };

            let target_repo = if repo_root.is_empty() {
                &req.repo_root
            } else {
                repo_root
            };

            let was_watching = watcher
                .watched_repos()
                .await
                .iter()
                .any(|p| p.to_string_lossy() == *target_repo);

            if was_watching {
                let _ = watcher.unwatch_repo(Path::new(target_repo)).await;
            }

            Response::ok(
                &req.id,
                ResponsePayload::Unwatch(UnwatchResponse {
                    repo_root: target_repo.clone(),
                    stopped: was_watching,
                }),
            )
        }

        Command::Shutdown(payload) => {
            tracing::info!("Shutdown requested: {}", payload.reason);
            let _ = shutdown_tx.send(());
            Response::ok(&req.id, ResponsePayload::Shutdown(ShutdownResponse {}))
        }
    }
}

fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = dirs_next::home_dir()
    {
        return home.join(rest).to_string_lossy().to_string();
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_tilde() {
        let expanded = expand_tilde("~/.ixchel/run/ixcheld.sock");
        assert!(!expanded.starts_with('~'));
        assert!(expanded.contains(".ixchel/run/ixcheld.sock"));
    }

    #[test]
    fn test_expand_tilde_no_tilde() {
        let path = "/tmp/test.sock";
        assert_eq!(expand_tilde(path), path);
    }
}
