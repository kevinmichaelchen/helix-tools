//! File watcher for detecting changes to `.ixchel/` directories.
//!
//! Uses the `notify` crate to watch for file system changes and emits
//! events via a channel when relevant files are modified.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

/// Event emitted when a watched file changes.
#[derive(Debug, Clone)]
pub struct WatchEvent {
    /// Repository root path.
    pub repo_root: PathBuf,
    /// Path to the changed file (relative to repo root).
    pub changed_path: PathBuf,
    /// Kind of change that occurred.
    pub kind: WatchEventKind,
}

/// Kind of file system change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchEventKind {
    /// File was created.
    Create,
    /// File was modified.
    Modify,
    /// File was deleted.
    Remove,
}

/// Directories within `.ixchel/` to ignore.
const IGNORED_DIRS: &[&str] = &["data", "models"];

/// Watcher for repository `.ixchel/` directories.
pub struct RepoWatcher {
    /// Active watchers by repo root path.
    watchers: Arc<Mutex<HashMap<PathBuf, RecommendedWatcher>>>,
    /// Channel sender for watch events.
    event_tx: mpsc::Sender<WatchEvent>,
}

impl RepoWatcher {
    /// Create a new repo watcher.
    ///
    /// Returns the watcher and a receiver for events.
    pub fn new(buffer_size: usize) -> (Self, mpsc::Receiver<WatchEvent>) {
        let (event_tx, event_rx) = mpsc::channel(buffer_size);
        let watcher = Self {
            watchers: Arc::new(Mutex::new(HashMap::new())),
            event_tx,
        };
        (watcher, event_rx)
    }

    /// Start watching a repository's `.ixchel/` directory.
    pub async fn watch_repo(&self, repo_root: &Path) -> Result<(), WatchError> {
        let ixchel_dir = repo_root.join(".ixchel");
        if !ixchel_dir.exists() {
            return Err(WatchError::NotInitialized(repo_root.to_path_buf()));
        }

        let mut watchers = self.watchers.lock().await;
        if watchers.contains_key(repo_root) {
            // Already watching
            return Ok(());
        }

        let repo_root_owned = repo_root.to_path_buf();
        let event_tx = self.event_tx.clone();

        // Create watcher with a callback that filters and sends events
        let watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    handle_notify_event(&repo_root_owned, event, &event_tx);
                }
            },
            Config::default(),
        )
        .map_err(|e| WatchError::WatcherFailed(e.to_string()))?;

        // Start watching - we need a mutable watcher
        let mut watcher = watcher;
        watcher
            .watch(&ixchel_dir, RecursiveMode::Recursive)
            .map_err(|e| WatchError::WatcherFailed(e.to_string()))?;

        tracing::info!("Started watching {}", ixchel_dir.display());
        watchers.insert(repo_root.to_path_buf(), watcher);
        drop(watchers);

        Ok(())
    }

    /// Stop watching a repository.
    pub async fn unwatch_repo(&self, repo_root: &Path) -> Result<(), WatchError> {
        let mut watchers = self.watchers.lock().await;
        if let Some(mut watcher) = watchers.remove(repo_root) {
            let ixchel_dir = repo_root.join(".ixchel");
            let _ = watcher.unwatch(&ixchel_dir);
            tracing::info!("Stopped watching {}", ixchel_dir.display());
        }
        drop(watchers);
        Ok(())
    }

    /// Get list of currently watched repositories.
    pub async fn watched_repos(&self) -> Vec<PathBuf> {
        self.watchers.lock().await.keys().cloned().collect()
    }
}

/// Handle a notify event and convert to our event type.
fn handle_notify_event(repo_root: &Path, event: Event, tx: &mpsc::Sender<WatchEvent>) {
    use notify::EventKind;

    let kind = match event.kind {
        EventKind::Create(_) => WatchEventKind::Create,
        EventKind::Modify(_) => WatchEventKind::Modify,
        EventKind::Remove(_) => WatchEventKind::Remove,
        // Ignore access, metadata, and other events
        _ => return,
    };

    for path in event.paths {
        // Only process .md files
        if path.extension().and_then(|s| s.to_str()) != Some("md") {
            continue;
        }

        // Check if this path is in an ignored directory
        if is_in_ignored_dir(repo_root, &path) {
            continue;
        }

        // Compute relative path
        let changed_path = path.strip_prefix(repo_root).unwrap_or(&path).to_path_buf();

        let watch_event = WatchEvent {
            repo_root: repo_root.to_path_buf(),
            changed_path,
            kind,
        };

        // Try to send, but don't block if the channel is full
        let _ = tx.try_send(watch_event);
    }
}

/// Check if a path is within an ignored directory.
fn is_in_ignored_dir(repo_root: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(repo_root).unwrap_or(path);

    // Check if any component after .ixchel is in IGNORED_DIRS
    let mut in_ixchel = false;
    for component in relative.components() {
        let name = component.as_os_str().to_string_lossy();
        if name == ".ixchel" {
            in_ixchel = true;
            continue;
        }
        if in_ixchel && IGNORED_DIRS.contains(&name.as_ref()) {
            return true;
        }
    }

    false
}

/// Errors that can occur during watching.
#[derive(Debug)]
pub enum WatchError {
    /// Repository is not initialized (no .ixchel directory).
    NotInitialized(PathBuf),
    /// Failed to create or configure the watcher.
    WatcherFailed(String),
}

impl std::fmt::Display for WatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotInitialized(path) => {
                write!(f, "Repository not initialized: {}", path.display())
            }
            Self::WatcherFailed(msg) => write!(f, "Watcher failed: {msg}"),
        }
    }
}

impl std::error::Error for WatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_in_ignored_dir() {
        let repo_root = PathBuf::from("/repo");

        // Should be ignored
        assert!(is_in_ignored_dir(
            &repo_root,
            &PathBuf::from("/repo/.ixchel/data/surrealkv/test.md")
        ));
        assert!(is_in_ignored_dir(
            &repo_root,
            &PathBuf::from("/repo/.ixchel/models/cache.md")
        ));

        // Should not be ignored
        assert!(!is_in_ignored_dir(
            &repo_root,
            &PathBuf::from("/repo/.ixchel/decisions/dec-123.md")
        ));
        assert!(!is_in_ignored_dir(
            &repo_root,
            &PathBuf::from("/repo/.ixchel/issues/iss-456.md")
        ));
        assert!(!is_in_ignored_dir(
            &repo_root,
            &PathBuf::from("/repo/.ixchel/config.md")
        ));
    }
}
