use crate::{
    Command, DEFAULT_SOCKET_PATH, DaemonError, EnqueueSyncPayload, Request, Response,
    ResponseResult, SyncState, UnwatchPayload, WaitSyncPayload, WatchPayload,
};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

const CONNECT_RETRY_DELAY_MS: u64 = 100;
const MAX_CONNECT_RETRIES: u32 = 5;

pub struct Client {
    socket_path: String,
}

impl Client {
    pub fn new() -> Self {
        Self::with_socket_path(DEFAULT_SOCKET_PATH)
    }

    pub fn with_socket_path(path: impl Into<String>) -> Self {
        Self {
            socket_path: path.into(),
        }
    }

    fn expanded_socket_path(&self) -> String {
        expand_tilde(&self.socket_path)
    }

    pub async fn connect(&self) -> Result<UnixStream, DaemonError> {
        self.connect_with_flags(&[]).await
    }

    /// Connect to the daemon, starting it with `--watch` flag if needed.
    pub async fn connect_for_watch(&self) -> Result<UnixStream, DaemonError> {
        self.connect_with_flags(&["--watch"]).await
    }

    async fn connect_with_flags(&self, extra_args: &[&str]) -> Result<UnixStream, DaemonError> {
        let socket_path = self.expanded_socket_path();

        for attempt in 0..MAX_CONNECT_RETRIES {
            match UnixStream::connect(&socket_path).await {
                Ok(stream) => return Ok(stream),
                Err(e)
                    if e.kind() == std::io::ErrorKind::NotFound
                        || e.kind() == std::io::ErrorKind::ConnectionRefused =>
                {
                    if attempt == 0 {
                        self.start_daemon_with_flags(extra_args).await?;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        CONNECT_RETRY_DELAY_MS * (u64::from(attempt) + 1),
                    ))
                    .await;
                }
                Err(e) => return Err(e.into()),
            }
        }

        Err(DaemonError::Internal(format!(
            "Failed to connect to daemon after {MAX_CONNECT_RETRIES} retries"
        )))
    }

    async fn start_daemon_with_flags(&self, extra_args: &[&str]) -> Result<(), DaemonError> {
        let socket_path = self.expanded_socket_path();

        if let Some(parent) = Path::new(&socket_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let ixcheld_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("ixcheld")))
            .filter(|p| p.exists())
            .unwrap_or_else(|| "ixcheld".into());

        let mut cmd = tokio::process::Command::new(ixcheld_path);
        cmd.arg("--socket")
            .arg(&socket_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        for arg in extra_args {
            cmd.arg(arg);
        }

        cmd.spawn()
            .map_err(|e| DaemonError::Internal(format!("Failed to start ixcheld: {e}")))?;

        Ok(())
    }

    pub async fn send(&self, request: Request) -> Result<Response, DaemonError> {
        let mut stream = self.connect().await?;

        let json = serde_json::to_string(&request)?;
        stream.write_all(json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: Response = serde_json::from_str(line.trim())?;
        Ok(response)
    }

    pub async fn ping(&self) -> Result<String, DaemonError> {
        let request = Request::new("", "", Command::Ping);
        let response = self.send(request).await?;

        match response.result {
            ResponseResult::Ok { payload } => {
                if let crate::ResponsePayload::Ping(ping) = payload {
                    Ok(ping.daemon_version)
                } else {
                    Err(DaemonError::Internal("Unexpected response type".into()))
                }
            }
            ResponseResult::Error { error } => Err(DaemonError::Internal(error.message)),
        }
    }

    pub async fn enqueue_sync(
        &self,
        repo_root: &str,
        tool: &str,
        directory: &str,
        force: bool,
    ) -> Result<String, DaemonError> {
        let request = Request::new(
            repo_root,
            tool,
            Command::EnqueueSync(EnqueueSyncPayload {
                directory: directory.to_string(),
                force,
            }),
        );
        let response = self.send(request).await?;

        match response.result {
            ResponseResult::Ok { payload } => {
                if let crate::ResponsePayload::EnqueueSync(enqueue) = payload {
                    Ok(enqueue.sync_id)
                } else {
                    Err(DaemonError::Internal("Unexpected response type".into()))
                }
            }
            ResponseResult::Error { error } => Err(DaemonError::Internal(error.message)),
        }
    }

    pub async fn wait_sync(
        &self,
        repo_root: &str,
        tool: &str,
        sync_id: &str,
        timeout_ms: u64,
    ) -> Result<SyncState, DaemonError> {
        let request = Request::new(
            repo_root,
            tool,
            Command::WaitSync(WaitSyncPayload {
                sync_id: sync_id.to_string(),
                timeout_ms,
            }),
        );
        let response = self.send(request).await?;

        match response.result {
            ResponseResult::Ok { payload } => {
                if let crate::ResponsePayload::WaitSync(wait) = payload {
                    Ok(wait.state)
                } else {
                    Err(DaemonError::Internal("Unexpected response type".into()))
                }
            }
            ResponseResult::Error { error } => Err(DaemonError::Internal(error.message)),
        }
    }

    pub async fn sync(
        &self,
        repo_root: &str,
        tool: &str,
        directory: &str,
        wait: bool,
    ) -> Result<SyncState, DaemonError> {
        let sync_id = self.enqueue_sync(repo_root, tool, directory, false).await?;

        if wait {
            self.wait_sync(repo_root, tool, &sync_id, 30_000).await
        } else {
            Ok(SyncState::Queued)
        }
    }

    pub async fn shutdown(&self, reason: &str) -> Result<(), DaemonError> {
        let request = Request::new(
            "",
            "",
            Command::Shutdown(crate::ShutdownPayload {
                reason: reason.to_string(),
            }),
        );
        let _ = self.send(request).await;
        Ok(())
    }

    /// Start watching a repository for file changes.
    ///
    /// Returns `(repo_root, started)` where `started` is true if watching was newly started.
    /// If the daemon isn't running, it will be started with `--watch` flag.
    pub async fn watch(&self, repo_root: &str) -> Result<(String, bool), DaemonError> {
        let request = Request::new(
            repo_root,
            "",
            Command::Watch(WatchPayload {
                repo_root: String::new(), // Use the one from request envelope
            }),
        );
        // Use connect_for_watch to ensure daemon is started with --watch flag
        let response = self.send_for_watch(request).await?;

        match response.result {
            ResponseResult::Ok { payload } => {
                if let crate::ResponsePayload::Watch(watch) = payload {
                    Ok((watch.repo_root, watch.started))
                } else {
                    Err(DaemonError::Internal("Unexpected response type".into()))
                }
            }
            ResponseResult::Error { error } => Err(DaemonError::Internal(error.message)),
        }
    }

    /// Send a request, starting the daemon with `--watch` if needed.
    async fn send_for_watch(&self, request: Request) -> Result<Response, DaemonError> {
        let mut stream = self.connect_for_watch().await?;

        let json = serde_json::to_string(&request)?;
        stream.write_all(json.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: Response = serde_json::from_str(line.trim())?;
        Ok(response)
    }

    /// Stop watching a repository for file changes.
    ///
    /// Returns `(repo_root, stopped)` where `stopped` is true if watching was actually stopped.
    pub async fn unwatch(&self, repo_root: &str) -> Result<(String, bool), DaemonError> {
        let request = Request::new(
            repo_root,
            "",
            Command::Unwatch(UnwatchPayload {
                repo_root: String::new(), // Use the one from request envelope
            }),
        );
        let response = self.send(request).await?;

        match response.result {
            ResponseResult::Ok { payload } => {
                if let crate::ResponsePayload::Unwatch(unwatch) = payload {
                    Ok((unwatch.repo_root, unwatch.stopped))
                } else {
                    Err(DaemonError::Internal("Unexpected response type".into()))
                }
            }
            ResponseResult::Error { error } => Err(DaemonError::Internal(error.message)),
        }
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
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
