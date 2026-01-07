//! Delta detection for incremental indexing.

use crate::helix_backend::HelixDecisionBackend;
use crate::manifest::IndexManifest;
use crate::types::Decision;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub struct DeltaResult {
    pub to_add: Vec<Decision>,
    pub to_modify: Vec<Decision>,
    pub to_remove: Vec<String>,
    pub renamed: Vec<(String, String)>,
    pub unchanged_count: u32,
}

pub fn compute_delta(
    repo_root: &Path,
    current_decisions: Vec<Decision>,
    stored_hashes: HashMap<String, String>,
    manifest: &IndexManifest,
) -> DeltaResult {
    let mut to_add = Vec::new();
    let mut to_modify = Vec::new();
    let mut to_remove = Vec::new();
    let mut renamed = Vec::new();
    let mut unchanged_count = 0u32;

    let mut seen_paths: HashSet<String> = HashSet::new();
    let mut renamed_old_paths: HashSet<String> = HashSet::new();

    for decision in current_decisions {
        let normalized_path = HelixDecisionBackend::normalize_path(repo_root, &decision.file_path);
        seen_paths.insert(normalized_path.clone());

        match stored_hashes.get(&normalized_path) {
            Some(stored_hash) if stored_hash == &decision.content_hash => {
                unchanged_count += 1;
            }
            Some(_) => {
                to_modify.push(decision);
            }
            None => {
                let renamed_from = decision.metadata.uuid.as_ref().and_then(|uuid| {
                    manifest
                        .find_by_content_hash(&decision.content_hash)
                        .filter(|old_entry| old_entry.uuid.as_ref() == Some(uuid))
                        .map(|old_entry| old_entry.file_path.to_string_lossy().to_string())
                        .filter(|old_path| old_path != &normalized_path)
                });

                if let Some(old_path) = renamed_from {
                    renamed.push((old_path.clone(), normalized_path));
                    renamed_old_paths.insert(old_path);
                } else {
                    to_add.push(decision);
                }
            }
        }
    }

    for path in stored_hashes.keys() {
        if !seen_paths.contains(path) && !renamed_old_paths.contains(path) {
            to_remove.push(path.clone());
        }
    }

    DeltaResult {
        to_add,
        to_modify,
        to_remove,
        renamed,
        unchanged_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::ManifestEntry;
    use crate::types::{DecisionMetadata, Status};
    use chrono::NaiveDate;
    use std::path::PathBuf;

    fn make_decision(id: u32, path: &str, hash: &str) -> Decision {
        Decision {
            metadata: DecisionMetadata {
                id,
                uuid: None,
                title: format!("Decision {id}"),
                status: Status::Accepted,
                date: NaiveDate::from_ymd_opt(2026, 1, 5).unwrap(),
                deciders: vec![],
                tags: vec![],
                content_hash: None,
                git_commit: None,
                supersedes: None,
                superseded_by: None,
                amends: None,
                depends_on: None,
                related_to: None,
            },
            body: String::new(),
            file_path: PathBuf::from(path),
            content_hash: hash.to_string(),
            embedding: None,
        }
    }

    fn make_decision_with_uuid(id: u32, path: &str, hash: &str, uuid: &str) -> Decision {
        let mut decision = make_decision(id, path, hash);
        decision.metadata.uuid = Some(uuid.to_string());
        decision
    }

    #[test]
    fn test_no_changes() {
        let repo_root = PathBuf::from("/repo");
        let manifest = IndexManifest::new();
        let current = vec![make_decision(1, "001.md", "hash1")];
        let stored: HashMap<_, _> = [("001.md".to_string(), "hash1".to_string())]
            .into_iter()
            .collect();

        let delta = compute_delta(&repo_root, current, stored, &manifest);
        assert!(delta.to_add.is_empty());
        assert!(delta.to_modify.is_empty());
        assert!(delta.to_remove.is_empty());
        assert!(delta.renamed.is_empty());
        assert_eq!(delta.unchanged_count, 1);
    }

    #[test]
    fn test_new_decision() {
        let repo_root = PathBuf::from("/repo");
        let manifest = IndexManifest::new();
        let current = vec![make_decision(1, "001.md", "hash1")];
        let stored: HashMap<String, String> = HashMap::new();

        let delta = compute_delta(&repo_root, current, stored, &manifest);
        assert_eq!(delta.to_add.len(), 1);
        assert!(delta.to_modify.is_empty());
        assert!(delta.to_remove.is_empty());
        assert!(delta.renamed.is_empty());
        assert_eq!(delta.unchanged_count, 0);
    }

    #[test]
    fn test_changed_decision() {
        let repo_root = PathBuf::from("/repo");
        let manifest = IndexManifest::new();
        let current = vec![make_decision(1, "001.md", "hash2")];
        let stored: HashMap<_, _> = [("001.md".to_string(), "hash1".to_string())]
            .into_iter()
            .collect();

        let delta = compute_delta(&repo_root, current, stored, &manifest);
        assert!(delta.to_add.is_empty());
        assert_eq!(delta.to_modify.len(), 1);
        assert!(delta.to_remove.is_empty());
        assert!(delta.renamed.is_empty());
        assert_eq!(delta.unchanged_count, 0);
    }

    #[test]
    fn test_deleted_decision() {
        let repo_root = PathBuf::from("/repo");
        let manifest = IndexManifest::new();
        let current = vec![];
        let stored: HashMap<_, _> = [("001.md".to_string(), "hash1".to_string())]
            .into_iter()
            .collect();

        let delta = compute_delta(&repo_root, current, stored, &manifest);
        assert!(delta.to_add.is_empty());
        assert!(delta.to_modify.is_empty());
        assert_eq!(delta.to_remove.len(), 1);
        assert!(delta.renamed.is_empty());
        assert_eq!(delta.unchanged_count, 0);
    }

    #[test]
    fn test_path_normalization_absolute_vs_relative() {
        let repo_root = PathBuf::from("/repo");
        let manifest = IndexManifest::new();
        let current = vec![make_decision(1, "/repo/.decisions/001.md", "hash1")];
        let stored: HashMap<_, _> = [(".decisions/001.md".to_string(), "hash1".to_string())]
            .into_iter()
            .collect();

        let delta = compute_delta(&repo_root, current, stored, &manifest);
        assert!(delta.to_add.is_empty());
        assert!(delta.to_modify.is_empty());
        assert!(delta.to_remove.is_empty());
        assert!(delta.renamed.is_empty());
        assert_eq!(delta.unchanged_count, 1);
    }

    #[test]
    fn test_no_rename_without_uuid() {
        let repo_root = PathBuf::from("/repo");
        let mut manifest = IndexManifest::new();
        manifest.upsert(ManifestEntry::new(
            PathBuf::from(".decisions/001-old.md"),
            1704067200,
            1024,
            "same-hash".to_string(),
            1,
            None,
            "model",
        ));

        let current = vec![make_decision(1, ".decisions/001-new.md", "same-hash")];
        let stored: HashMap<_, _> =
            [(".decisions/001-old.md".to_string(), "same-hash".to_string())]
                .into_iter()
                .collect();

        let delta = compute_delta(&repo_root, current, stored, &manifest);
        assert_eq!(delta.to_add.len(), 1);
        assert!(delta.to_modify.is_empty());
        assert_eq!(delta.to_remove.len(), 1);
        assert!(delta.renamed.is_empty());
        assert_eq!(delta.unchanged_count, 0);
    }

    #[test]
    fn test_rename_detection_by_uuid() {
        let repo_root = PathBuf::from("/repo");
        let mut manifest = IndexManifest::new();
        let mut entry = ManifestEntry::new(
            PathBuf::from(".decisions/001-old.md"),
            1704067200,
            1024,
            "same-hash".to_string(),
            99,
            Some("hx-abc123".to_string()),
            "model",
        );
        entry.uuid = Some("hx-abc123".to_string());
        manifest.upsert(entry);

        let current = vec![make_decision_with_uuid(
            99,
            ".decisions/001-new.md",
            "same-hash",
            "hx-abc123",
        )];
        let stored: HashMap<_, _> =
            [(".decisions/001-old.md".to_string(), "same-hash".to_string())]
                .into_iter()
                .collect();

        let delta = compute_delta(&repo_root, current, stored, &manifest);
        assert!(delta.to_add.is_empty());
        assert!(delta.to_remove.is_empty());
        assert_eq!(delta.renamed.len(), 1);
    }

    #[test]
    fn test_no_rename_with_different_uuid() {
        let repo_root = PathBuf::from("/repo");
        let mut manifest = IndexManifest::new();
        let mut entry = ManifestEntry::new(
            PathBuf::from(".decisions/001-old.md"),
            1704067200,
            1024,
            "same-hash".to_string(),
            1,
            Some("hx-old111".to_string()),
            "model",
        );
        entry.uuid = Some("hx-old111".to_string());
        manifest.upsert(entry);

        let current = vec![make_decision_with_uuid(
            2,
            ".decisions/002-new.md",
            "same-hash",
            "hx-new222",
        )];
        let stored: HashMap<_, _> =
            [(".decisions/001-old.md".to_string(), "same-hash".to_string())]
                .into_iter()
                .collect();

        let delta = compute_delta(&repo_root, current, stored, &manifest);
        assert_eq!(delta.to_add.len(), 1);
        assert!(delta.renamed.is_empty());
        assert_eq!(delta.to_remove.len(), 1);
    }
}
