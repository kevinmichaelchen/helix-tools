use crate::{SyncState, SyncStats};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct QueueKey {
    pub repo_root: String,
    pub tool: String,
}

impl QueueKey {
    pub fn new(repo_root: impl Into<String>, tool: impl Into<String>) -> Self {
        Self {
            repo_root: repo_root.into(),
            tool: tool.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncJob {
    pub id: String,
    pub key: QueueKey,
    pub directory: String,
    pub force: bool,
    pub state: SyncState,
    pub queued_at: Instant,
    pub started_at: Option<Instant>,
    pub completed_at: Option<Instant>,
    pub stats: Option<SyncStats>,
    pub error: Option<String>,
    state_tx: broadcast::Sender<SyncState>,
}

impl SyncJob {
    pub fn new(key: QueueKey, directory: String, force: bool) -> Self {
        let (state_tx, _) = broadcast::channel(16);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            key,
            directory,
            force,
            state: SyncState::Queued,
            queued_at: Instant::now(),
            started_at: None,
            completed_at: None,
            stats: None,
            error: None,
            state_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SyncState> {
        self.state_tx.subscribe()
    }

    fn set_state(&mut self, new_state: SyncState) {
        self.state = new_state;
        let _ = self.state_tx.send(new_state);
    }

    pub fn start(&mut self) {
        self.started_at = Some(Instant::now());
        self.set_state(SyncState::Running);
    }

    pub fn complete(&mut self, job_stats: SyncStats) {
        self.completed_at = Some(Instant::now());
        self.stats = Some(job_stats);
        self.set_state(SyncState::Done);
    }

    pub fn fail(&mut self, error: String) {
        self.completed_at = Some(Instant::now());
        self.error = Some(error);
        self.set_state(SyncState::Error);
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn queued_at_ms(&self) -> u64 {
        self.queued_at.elapsed().as_millis() as u64
    }
}

pub struct SyncQueue {
    jobs: Arc<RwLock<HashMap<String, SyncJob>>>,
    pending: Arc<RwLock<HashMap<QueueKey, String>>>,
}

impl SyncQueue {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
            pending: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn enqueue(
        &self,
        repo_root: &str,
        tool: &str,
        directory: &str,
        force: bool,
    ) -> (String, bool) {
        let key = QueueKey::new(repo_root, tool);

        {
            let pending = self.pending.read().await;
            if let Some(existing_id) = pending.get(&key) {
                let jobs = self.jobs.read().await;
                if let Some(job) = jobs.get(existing_id)
                    && job.state == SyncState::Queued
                    && !force
                {
                    return (existing_id.clone(), false);
                }
            }
        }

        let job = SyncJob::new(key.clone(), directory.to_string(), force);
        let id = job.id.clone();

        self.jobs.write().await.insert(id.clone(), job);
        self.pending.write().await.insert(key, id.clone());

        (id, true)
    }

    pub async fn get(&self, id: &str) -> Option<SyncJob> {
        self.jobs.read().await.get(id).cloned()
    }

    pub async fn get_pending(&self, key: &QueueKey) -> Option<SyncJob> {
        let id = self.pending.read().await.get(key).cloned()?;
        self.jobs.read().await.get(&id).cloned()
    }

    #[allow(clippy::significant_drop_tightening)]
    pub async fn start(&self, id: &str) -> bool {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(id)
            && job.state == SyncState::Queued
        {
            job.start();
            return true;
        }
        false
    }

    pub async fn complete(&self, id: &str, job_stats: SyncStats) {
        let key = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(id).map(|job| {
                job.complete(job_stats);
                job.key.clone()
            })
        };

        if let Some(key) = key {
            let mut pending = self.pending.write().await;
            if pending.get(&key).is_some_and(|pid| pid == id) {
                pending.remove(&key);
            }
        }
    }

    pub async fn fail(&self, id: &str, error: String) {
        let key = {
            let mut jobs = self.jobs.write().await;
            jobs.get_mut(id).map(|job| {
                job.fail(error);
                job.key.clone()
            })
        };

        if let Some(key) = key {
            let mut pending = self.pending.write().await;
            if pending.get(&key).is_some_and(|pid| pid == id) {
                pending.remove(&key);
            }
        }
    }

    #[allow(clippy::significant_drop_tightening)]
    pub async fn wait(&self, id: &str, timeout: Duration) -> Option<SyncState> {
        let mut rx = {
            let jobs = self.jobs.read().await;
            let job = jobs.get(id)?;

            if job.state == SyncState::Done || job.state == SyncState::Error {
                return Some(job.state);
            }

            job.subscribe()
        };

        let deadline = Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return None;
            }

            match tokio::time::timeout(remaining, rx.recv()).await {
                Ok(Ok(new_state)) => {
                    if new_state == SyncState::Done || new_state == SyncState::Error {
                        return Some(new_state);
                    }
                }
                Ok(Err(_)) | Err(_) => return None,
            }
        }
    }

    pub async fn list_queues(&self) -> Vec<crate::QueueInfo> {
        let pending = self.pending.read().await;
        let jobs = self.jobs.read().await;

        pending
            .iter()
            .filter_map(|(key, id)| {
                let job = jobs.get(id)?;
                let active = (job.state == SyncState::Running).then(|| id.clone());
                let pending_count = u32::from(job.state == SyncState::Queued);
                Some(crate::QueueInfo {
                    repo_root: key.repo_root.clone(),
                    tool: key.tool.clone(),
                    pending: pending_count,
                    active,
                })
            })
            .collect()
    }

    pub async fn cleanup_old(&self, max_age: Duration) {
        let now = Instant::now();
        let mut jobs = self.jobs.write().await;

        jobs.retain(|_, job| {
            job.completed_at
                .is_none_or(|completed_at| now.duration_since(completed_at) < max_age)
        });
    }
}

impl Default for SyncQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_enqueue_new_job() {
        let queue = SyncQueue::new();
        let (id, is_new) = queue
            .enqueue("/repo", "decisions", ".decisions", false)
            .await;

        assert!(is_new);
        assert!(!id.is_empty());

        let job = queue.get(&id).await.unwrap();
        assert_eq!(job.state, SyncState::Queued);
        assert_eq!(job.key.repo_root, "/repo");
        assert_eq!(job.key.tool, "decisions");
    }

    #[tokio::test]
    async fn test_enqueue_coalesces_duplicate() {
        let queue = SyncQueue::new();
        let (id1, is_new1) = queue
            .enqueue("/repo", "decisions", ".decisions", false)
            .await;
        let (id2, is_new2) = queue
            .enqueue("/repo", "decisions", ".decisions", false)
            .await;

        assert!(is_new1);
        assert!(!is_new2);
        assert_eq!(id1, id2);
    }

    #[tokio::test]
    async fn test_enqueue_force_creates_new() {
        let queue = SyncQueue::new();
        let (id1, _) = queue
            .enqueue("/repo", "decisions", ".decisions", false)
            .await;
        let (id2, is_new2) = queue
            .enqueue("/repo", "decisions", ".decisions", true)
            .await;

        assert!(is_new2);
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_different_repos_separate_queues() {
        let queue = SyncQueue::new();
        let (id1, is_new1) = queue
            .enqueue("/repo1", "decisions", ".decisions", false)
            .await;
        let (id2, is_new2) = queue
            .enqueue("/repo2", "decisions", ".decisions", false)
            .await;

        assert!(is_new1);
        assert!(is_new2);
        assert_ne!(id1, id2);
    }

    #[tokio::test]
    async fn test_job_lifecycle() {
        let queue = SyncQueue::new();
        let (id, _) = queue
            .enqueue("/repo", "decisions", ".decisions", false)
            .await;

        assert!(queue.start(&id).await);

        let job = queue.get(&id).await.unwrap();
        assert_eq!(job.state, SyncState::Running);

        queue
            .complete(
                &id,
                SyncStats {
                    files_scanned: 10,
                    files_updated: 2,
                    duration_ms: 100,
                },
            )
            .await;

        let job = queue.get(&id).await.unwrap();
        assert_eq!(job.state, SyncState::Done);
        assert!(job.stats.is_some());
    }

    #[tokio::test]
    async fn test_wait_already_complete() {
        let queue = SyncQueue::new();
        let (id, _) = queue
            .enqueue("/repo", "decisions", ".decisions", false)
            .await;

        queue.start(&id).await;
        queue.complete(&id, SyncStats::default()).await;

        let result = queue.wait(&id, Duration::from_millis(100)).await;
        assert_eq!(result, Some(SyncState::Done));
    }

    #[tokio::test]
    async fn test_list_queues() {
        let queue = SyncQueue::new();
        queue
            .enqueue("/repo1", "decisions", ".decisions", false)
            .await;
        queue.enqueue("/repo2", "hbd", ".tickets", false).await;

        let queues = queue.list_queues().await;
        assert_eq!(queues.len(), 2);
    }
}
