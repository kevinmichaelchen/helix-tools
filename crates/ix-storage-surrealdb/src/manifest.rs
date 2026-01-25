//! Sync manifest for tracking entity state between syncs.
//!
//! The manifest stores content hashes and file paths for each entity,
//! enabling incremental sync that skips unchanged files.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entry in the sync manifest tracking an entity's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Content hash (blake3) of the file at last sync.
    pub content_hash: String,
    /// Relative file path from repo root.
    pub file_path: String,
    /// Unix timestamp (seconds) of last sync.
    pub last_synced: u64,
}

/// Manifest tracking all synced entities.
///
/// Used to determine which files have changed since the last sync,
/// enabling incremental updates instead of full rebuilds.
#[derive(Debug, Default)]
pub struct SyncManifest {
    /// Map from `entity_id` to manifest entry.
    entries: HashMap<String, ManifestEntry>,
}

impl SyncManifest {
    /// Create an empty manifest.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a manifest from existing entries.
    pub const fn from_entries(entries: HashMap<String, ManifestEntry>) -> Self {
        Self { entries }
    }

    /// Get an entry by entity ID.
    #[allow(dead_code)]
    pub fn get(&self, entity_id: &str) -> Option<&ManifestEntry> {
        self.entries.get(entity_id)
    }

    /// Insert or update an entry.
    pub fn insert(&mut self, entity_id: String, entry: ManifestEntry) {
        self.entries.insert(entity_id, entry);
    }

    /// Remove an entry by entity ID.
    #[allow(dead_code)]
    pub fn remove(&mut self, entity_id: &str) -> Option<ManifestEntry> {
        self.entries.remove(entity_id)
    }

    /// Get all entity IDs in the manifest.
    pub fn entity_ids(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    /// Check if the manifest contains an entity.
    #[allow(dead_code)]
    pub fn contains(&self, entity_id: &str) -> bool {
        self.entries.contains_key(entity_id)
    }

    /// Get the number of entries.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the manifest is empty.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Result of comparing a file against the manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncAction {
    /// Entity is new, needs to be inserted.
    Insert,
    /// Entity has changed, needs to be updated.
    Update,
    /// Entity is unchanged, skip processing.
    Skip,
}

impl SyncManifest {
    /// Determine what action to take for an entity based on its current hash and path.
    ///
    /// Returns `Update` if either the content hash or file path has changed.
    pub fn action_for(
        &self,
        entity_id: &str,
        current_hash: &str,
        current_path: &str,
    ) -> SyncAction {
        match self.entries.get(entity_id) {
            Some(entry)
                if entry.content_hash == current_hash && entry.file_path == current_path =>
            {
                SyncAction::Skip
            }
            Some(_) => SyncAction::Update,
            None => SyncAction::Insert,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PATH: &str = ".ixchel/decisions/dec-123.md";

    #[test]
    fn test_manifest_action_for_new() {
        let manifest = SyncManifest::new();
        assert_eq!(
            manifest.action_for("dec-123", "abc123", TEST_PATH),
            SyncAction::Insert
        );
    }

    #[test]
    fn test_manifest_action_for_unchanged() {
        let mut manifest = SyncManifest::new();
        manifest.insert(
            "dec-123".to_string(),
            ManifestEntry {
                content_hash: "abc123".to_string(),
                file_path: TEST_PATH.to_string(),
                last_synced: 1000,
            },
        );
        assert_eq!(
            manifest.action_for("dec-123", "abc123", TEST_PATH),
            SyncAction::Skip
        );
    }

    #[test]
    fn test_manifest_action_for_content_changed() {
        let mut manifest = SyncManifest::new();
        manifest.insert(
            "dec-123".to_string(),
            ManifestEntry {
                content_hash: "abc123".to_string(),
                file_path: TEST_PATH.to_string(),
                last_synced: 1000,
            },
        );
        assert_eq!(
            manifest.action_for("dec-123", "different_hash", TEST_PATH),
            SyncAction::Update
        );
    }

    #[test]
    fn test_manifest_action_for_path_changed() {
        let mut manifest = SyncManifest::new();
        manifest.insert(
            "dec-123".to_string(),
            ManifestEntry {
                content_hash: "abc123".to_string(),
                file_path: TEST_PATH.to_string(),
                last_synced: 1000,
            },
        );
        // Same content hash but different path should trigger update
        assert_eq!(
            manifest.action_for("dec-123", "abc123", ".ixchel/decisions/renamed.md"),
            SyncAction::Update
        );
    }
}
