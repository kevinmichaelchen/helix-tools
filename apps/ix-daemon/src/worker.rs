//! Background sync worker that processes queued sync jobs.
//!
//! The worker polls the queue for pending jobs and executes them using `ix_app::sync()`.

use crate::SyncStats;
use crate::queue::SyncQueue;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

/// Interval between queue polls when idle.
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// Background worker that processes sync jobs from the queue.
pub struct SyncWorker {
    queue: Arc<SyncQueue>,
    shutdown_rx: broadcast::Receiver<()>,
}

impl SyncWorker {
    /// Create a new sync worker.
    pub const fn new(queue: Arc<SyncQueue>, shutdown_rx: broadcast::Receiver<()>) -> Self {
        Self { queue, shutdown_rx }
    }

    /// Run the worker loop until shutdown is signaled.
    #[allow(clippy::cognitive_complexity)]
    pub async fn run(mut self) {
        tracing::info!("Sync worker started");

        loop {
            tokio::select! {
                _ = self.shutdown_rx.recv() => {
                    tracing::info!("Sync worker received shutdown signal");
                    break;
                }
                () = tokio::time::sleep(POLL_INTERVAL) => {
                    if let Some(job) = self.queue.next_pending().await {
                        self.process_job(&job.id, &job.key.repo_root).await;
                    }
                }
            }
        }

        tracing::info!("Sync worker stopped");
    }

    /// Process a single sync job.
    #[allow(clippy::cognitive_complexity, clippy::cast_possible_truncation)]
    async fn process_job(&self, job_id: &str, repo_root: &str) {
        tracing::info!("Processing sync job {} for {}", job_id, repo_root);

        // Mark job as running
        if !self.queue.start(job_id).await {
            tracing::warn!("Failed to start job {} - may have been cancelled", job_id);
            return;
        }

        let start = Instant::now();

        // Execute sync in a blocking task to avoid blocking the async runtime
        let repo_root_owned = repo_root.to_string();
        let result = tokio::task::spawn_blocking(move || execute_sync(&repo_root_owned)).await;

        let duration_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(Ok(core_stats)) => {
                let stats = SyncStats {
                    files_scanned: u64::from(core_stats.scanned),
                    files_updated: u64::from(core_stats.added + core_stats.modified),
                    duration_ms,
                };
                tracing::info!(
                    "Sync job {} completed: scanned={}, added={}, modified={}, deleted={}, unchanged={}, duration={}ms",
                    job_id,
                    core_stats.scanned,
                    core_stats.added,
                    core_stats.modified,
                    core_stats.deleted,
                    core_stats.unchanged,
                    duration_ms
                );
                self.queue.complete(job_id, stats).await;
            }
            Ok(Err(e)) => {
                let error_msg = format!("Sync failed: {e}");
                tracing::error!("Sync job {} failed: {}", job_id, error_msg);
                self.queue.fail(job_id, error_msg).await;
            }
            Err(e) => {
                let error_msg = format!("Sync task panicked: {e}");
                tracing::error!("Sync job {} panicked: {}", job_id, error_msg);
                self.queue.fail(job_id, error_msg).await;
            }
        }
    }
}

/// Execute sync for a repository.
///
/// This is a blocking operation that opens the repo and syncs.
fn execute_sync(repo_root: &str) -> anyhow::Result<ix_core::index::SyncStats> {
    use std::path::Path;
    let repo = ix_core::repo::IxchelRepo::open_from(Path::new(repo_root))?;
    ix_app::sync(&repo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_worker_receives_shutdown() {
        let queue = Arc::new(SyncQueue::new());
        let (tx, rx) = broadcast::channel(1);

        let worker = SyncWorker::new(Arc::clone(&queue), rx);

        // Spawn worker
        let handle = tokio::spawn(async move {
            worker.run().await;
        });

        // Send shutdown
        tx.send(()).unwrap();

        // Worker should stop
        tokio::time::timeout(Duration::from_secs(2), handle)
            .await
            .expect("Worker should stop within timeout")
            .expect("Worker should not panic");
    }
}
