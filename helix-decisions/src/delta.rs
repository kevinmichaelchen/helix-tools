//! Delta detection for incremental indexing.

use crate::types::Decision;
use std::collections::{HashMap, HashSet};

pub struct DeltaResult {
    pub to_add: Vec<Decision>,
    pub to_remove: Vec<String>,
}

pub fn compute_delta(
    current_decisions: Vec<Decision>,
    stored_hashes: HashMap<String, String>,
) -> DeltaResult {
    let mut to_add = Vec::new();
    let mut to_remove = Vec::new();

    let mut seen_paths: HashSet<String> = HashSet::new();

    for decision in current_decisions {
        let path = decision.file_path.to_string_lossy().to_string();
        seen_paths.insert(path.clone());

        match stored_hashes.get(&path) {
            Some(stored_hash) if stored_hash == &decision.content_hash => {}
            _ => {
                to_add.push(decision);
            }
        }
    }

    for path in stored_hashes.keys() {
        if !seen_paths.contains(path) {
            to_remove.push(path.clone());
        }
    }

    DeltaResult { to_add, to_remove }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_no_changes() {
        let current = vec![make_decision(1, "001.md", "hash1")];
        let stored: HashMap<_, _> = [("001.md".to_string(), "hash1".to_string())]
            .into_iter()
            .collect();

        let delta = compute_delta(current, stored);
        assert!(delta.to_add.is_empty());
        assert!(delta.to_remove.is_empty());
    }

    #[test]
    fn test_new_decision() {
        let current = vec![make_decision(1, "001.md", "hash1")];
        let stored: HashMap<String, String> = HashMap::new();

        let delta = compute_delta(current, stored);
        assert_eq!(delta.to_add.len(), 1);
        assert!(delta.to_remove.is_empty());
    }

    #[test]
    fn test_changed_decision() {
        let current = vec![make_decision(1, "001.md", "hash2")];
        let stored: HashMap<_, _> = [("001.md".to_string(), "hash1".to_string())]
            .into_iter()
            .collect();

        let delta = compute_delta(current, stored);
        assert_eq!(delta.to_add.len(), 1);
        assert!(delta.to_remove.is_empty());
    }

    #[test]
    fn test_deleted_decision() {
        let current = vec![];
        let stored: HashMap<_, _> = [("001.md".to_string(), "hash1".to_string())]
            .into_iter()
            .collect();

        let delta = compute_delta(current, stored);
        assert!(delta.to_add.is_empty());
        assert_eq!(delta.to_remove.len(), 1);
    }
}
