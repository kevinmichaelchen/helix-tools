//! Manifest tracking for incremental indexing.
//!
//! The manifest tracks metadata about indexed decision files, enabling:
//! - Fast change detection via mtime + size checks
//! - Content hash verification for modified files
//! - Vector ID reuse when content is unchanged
//! - Rename detection via content hash matching

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Key used to store the manifest in HelixDB metadata.
pub const MANIFEST_KEY: &str = "manifest:helix-decisions:v1";

/// Current indexer version. Bump when embedding model or schema changes.
pub const INDEXER_VERSION: u32 = 1;

/// Entry in the index manifest for a single decision file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ManifestEntry {
    /// Relative path from repo root (e.g., ".decisions/001-foo.md")
    pub file_path: PathBuf,

    /// Last modified time (Unix timestamp in seconds)
    pub mtime: u64,

    /// File size in bytes
    pub size: u64,

    /// SHA256 hash of file content
    pub content_hash: String,

    /// Decision ID from frontmatter
    pub decision_id: u32,

    /// Optional UUID from frontmatter
    pub uuid: Option<String>,

    /// ID of the vector in storage (for reuse on updates)
    pub vector_id: Option<String>,

    /// ID of the node in HelixDB graph storage
    pub node_id: Option<u128>,

    /// Embedding model used (e.g., "bge-small-en-v1.5")
    pub embedding_model: String,

    /// Indexer version when this entry was created
    pub indexer_version: u32,
}

impl ManifestEntry {
    #[must_use]
    pub fn new(
        file_path: PathBuf,
        mtime: u64,
        size: u64,
        content_hash: String,
        decision_id: u32,
        uuid: Option<String>,
        embedding_model: &str,
    ) -> Self {
        Self {
            file_path,
            mtime,
            size,
            content_hash,
            decision_id,
            uuid,
            vector_id: None,
            node_id: None,
            embedding_model: embedding_model.to_string(),
            indexer_version: INDEXER_VERSION,
        }
    }

    /// Check if file stats indicate the file may have changed.
    ///
    /// Returns `true` if mtime or size differs (needs content hash check).
    #[must_use]
    pub fn stats_changed(&self, mtime: u64, size: u64) -> bool {
        self.mtime != mtime || self.size != size
    }

    /// Check if content hash differs (file was modified).
    #[must_use]
    pub fn content_changed(&self, content_hash: &str) -> bool {
        self.content_hash != content_hash
    }

    /// Check if re-embedding is needed due to model or indexer change.
    #[must_use]
    pub fn needs_reembed(&self, embedding_model: &str) -> bool {
        self.embedding_model != embedding_model || self.indexer_version != INDEXER_VERSION
    }
}

/// Index manifest tracking all indexed decision files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexManifest {
    /// Map from file path (as string) to manifest entry
    entries: HashMap<String, ManifestEntry>,
}

impl IndexManifest {
    /// Create a new empty manifest.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Load manifest from JSON bytes.
    ///
    /// Returns empty manifest if data is empty or invalid.
    #[must_use]
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.is_empty() {
            return Self::new();
        }
        serde_json::from_slice(data).unwrap_or_default()
    }

    /// Serialize manifest to JSON bytes.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap_or_default()
    }

    /// Get entry by file path.
    #[must_use]
    pub fn get(&self, path: &Path) -> Option<&ManifestEntry> {
        self.entries.get(&path.to_string_lossy().to_string())
    }

    /// Check if manifest contains entry for path.
    #[must_use]
    pub fn contains(&self, path: &Path) -> bool {
        self.entries
            .contains_key(&path.to_string_lossy().to_string())
    }

    /// Insert or update an entry.
    pub fn upsert(&mut self, entry: ManifestEntry) {
        let key = entry.file_path.to_string_lossy().to_string();
        self.entries.insert(key, entry);
    }

    /// Remove an entry by path.
    ///
    /// Returns the removed entry if it existed.
    pub fn remove(&mut self, path: &Path) -> Option<ManifestEntry> {
        self.entries.remove(&path.to_string_lossy().to_string())
    }

    /// Get all entries.
    pub fn entries(&self) -> impl Iterator<Item = &ManifestEntry> {
        self.entries.values()
    }

    /// Get all file paths in the manifest.
    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.entries.values().map(|e| &e.file_path)
    }

    /// Get entry count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if manifest is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Find entry by content hash (for rename detection).
    #[must_use]
    pub fn find_by_content_hash(&self, content_hash: &str) -> Option<&ManifestEntry> {
        self.entries
            .values()
            .find(|e| e.content_hash == content_hash)
    }

    /// Find entry by decision ID (for relationship lookups).
    #[must_use]
    pub fn find_by_decision_id(&self, decision_id: u32) -> Option<&ManifestEntry> {
        self.entries.values().find(|e| e.decision_id == decision_id)
    }
}

/// Get file mtime as Unix timestamp.
///
/// Returns 0 if metadata cannot be read.
pub fn get_mtime(path: &Path) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        })
        .unwrap_or(0)
}

/// Get file size in bytes.
///
/// Returns 0 if metadata cannot be read.
pub fn get_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(path: &str, hash: &str, decision_id: u32) -> ManifestEntry {
        ManifestEntry::new(
            PathBuf::from(path),
            1704067200, // 2024-01-01 00:00:00 UTC
            1024,
            hash.to_string(),
            decision_id,
            None,
            "bge-small-en-v1.5",
        )
    }

    #[test]
    fn test_manifest_entry_creation() {
        let entry = sample_entry(".decisions/001-test.md", "abc123", 1);

        assert_eq!(entry.file_path, PathBuf::from(".decisions/001-test.md"));
        assert_eq!(entry.mtime, 1704067200);
        assert_eq!(entry.size, 1024);
        assert_eq!(entry.content_hash, "abc123");
        assert_eq!(entry.decision_id, 1);
        assert!(entry.vector_id.is_none());
        assert_eq!(entry.embedding_model, "bge-small-en-v1.5");
        assert_eq!(entry.indexer_version, INDEXER_VERSION);
    }

    #[test]
    fn test_stats_changed() {
        let entry = sample_entry(".decisions/001-test.md", "abc123", 1);

        // Same stats
        assert!(!entry.stats_changed(1704067200, 1024));

        // Different mtime
        assert!(entry.stats_changed(1704067300, 1024));

        // Different size
        assert!(entry.stats_changed(1704067200, 2048));

        // Both different
        assert!(entry.stats_changed(1704067300, 2048));
    }

    #[test]
    fn test_content_changed() {
        let entry = sample_entry(".decisions/001-test.md", "abc123", 1);

        assert!(!entry.content_changed("abc123"));
        assert!(entry.content_changed("def456"));
    }

    #[test]
    fn test_needs_reembed() {
        let entry = sample_entry(".decisions/001-test.md", "abc123", 1);

        // Same model and version
        assert!(!entry.needs_reembed("bge-small-en-v1.5"));

        // Different model
        assert!(entry.needs_reembed("bge-large-en-v1.5"));
    }

    #[test]
    fn test_manifest_crud() {
        let mut manifest = IndexManifest::new();
        assert!(manifest.is_empty());
        assert_eq!(manifest.len(), 0);

        // Insert
        let entry1 = sample_entry(".decisions/001-test.md", "hash1", 1);
        manifest.upsert(entry1.clone());
        assert_eq!(manifest.len(), 1);
        assert!(manifest.contains(&PathBuf::from(".decisions/001-test.md")));

        // Get
        let retrieved = manifest.get(&PathBuf::from(".decisions/001-test.md"));
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().content_hash, "hash1");

        // Update (upsert existing)
        let mut entry1_updated = entry1;
        entry1_updated.content_hash = "hash1-updated".to_string();
        manifest.upsert(entry1_updated);
        assert_eq!(manifest.len(), 1);
        let retrieved = manifest.get(&PathBuf::from(".decisions/001-test.md"));
        assert_eq!(retrieved.unwrap().content_hash, "hash1-updated");

        // Insert another
        let entry2 = sample_entry(".decisions/002-another.md", "hash2", 2);
        manifest.upsert(entry2);
        assert_eq!(manifest.len(), 2);

        // Remove
        let removed = manifest.remove(&PathBuf::from(".decisions/001-test.md"));
        assert!(removed.is_some());
        assert_eq!(manifest.len(), 1);
        assert!(!manifest.contains(&PathBuf::from(".decisions/001-test.md")));
    }

    #[test]
    fn test_manifest_serialization() {
        let mut manifest = IndexManifest::new();
        manifest.upsert(sample_entry(".decisions/001-test.md", "hash1", 1));
        manifest.upsert(sample_entry(".decisions/002-another.md", "hash2", 2));

        // Serialize
        let bytes = manifest.to_bytes();
        assert!(!bytes.is_empty());

        // Deserialize
        let restored = IndexManifest::from_bytes(&bytes);
        assert_eq!(restored.len(), 2);
        assert!(restored.contains(&PathBuf::from(".decisions/001-test.md")));
        assert!(restored.contains(&PathBuf::from(".decisions/002-another.md")));
    }

    #[test]
    fn test_manifest_from_empty_bytes() {
        let manifest = IndexManifest::from_bytes(&[]);
        assert!(manifest.is_empty());
    }

    #[test]
    fn test_manifest_from_invalid_bytes() {
        let manifest = IndexManifest::from_bytes(b"not valid json");
        assert!(manifest.is_empty());
    }

    #[test]
    fn test_find_by_content_hash() {
        let mut manifest = IndexManifest::new();
        manifest.upsert(sample_entry(".decisions/001-test.md", "unique-hash-1", 1));
        manifest.upsert(sample_entry(
            ".decisions/002-another.md",
            "unique-hash-2",
            2,
        ));

        let found = manifest.find_by_content_hash("unique-hash-1");
        assert!(found.is_some());
        assert_eq!(found.unwrap().decision_id, 1);

        let not_found = manifest.find_by_content_hash("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_by_decision_id() {
        let mut manifest = IndexManifest::new();
        manifest.upsert(sample_entry(".decisions/001-test.md", "hash1", 1));
        manifest.upsert(sample_entry(".decisions/002-another.md", "hash2", 2));

        let found = manifest.find_by_decision_id(2);
        assert!(found.is_some());
        assert_eq!(
            found.unwrap().file_path,
            PathBuf::from(".decisions/002-another.md")
        );

        let not_found = manifest.find_by_decision_id(99);
        assert!(not_found.is_none());
    }

    #[test]
    fn test_vector_id_field() {
        let mut entry = sample_entry(".decisions/001-test.md", "hash1", 1);
        assert!(entry.vector_id.is_none());

        entry.vector_id = Some("vec-12345".to_string());
        assert_eq!(entry.vector_id, Some("vec-12345".to_string()));

        // Verify it serializes correctly
        let mut manifest = IndexManifest::new();
        manifest.upsert(entry);
        let bytes = manifest.to_bytes();
        let restored = IndexManifest::from_bytes(&bytes);
        let retrieved = restored
            .get(&PathBuf::from(".decisions/001-test.md"))
            .unwrap();
        assert_eq!(retrieved.vector_id, Some("vec-12345".to_string()));
    }
}
