//! Load decisions from filesystem.

use crate::types::{Decision, DecisionMetadata};
use anyhow::{Context, Result};
use gray_matter::{Matter, engine::YAML};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub fn load_decisions(dir: &Path) -> Result<Vec<Decision>> {
    let report = load_decisions_with_errors(dir)?;

    for error in &report.errors {
        eprintln!(
            "Warning: Skipping {}: {}",
            error.file_path.display(),
            error.message
        );
    }

    Ok(report.decisions)
}

pub struct LoadDecisionError {
    pub file_path: PathBuf,
    pub message: String,
}

pub struct LoadDecisionsReport {
    pub decisions: Vec<Decision>,
    pub errors: Vec<LoadDecisionError>,
    pub total_files: usize,
}

pub fn load_decisions_with_errors(dir: &Path) -> Result<LoadDecisionsReport> {
    let mut decisions = Vec::new();
    let mut errors = Vec::new();
    let mut total_files = 0usize;
    let matter = Matter::<YAML>::new();

    if !dir.exists() {
        anyhow::bail!("Directory not found: {}", dir.display());
    }

    for entry in std::fs::read_dir(dir).context("Failed to read directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().is_some_and(|e| e == "md") {
            total_files += 1;
            match load_decision(&path, &matter) {
                Ok(decision) => decisions.push(decision),
                Err(e) => errors.push(LoadDecisionError {
                    file_path: path,
                    message: e.to_string(),
                }),
            }
        }
    }

    Ok(LoadDecisionsReport {
        decisions,
        errors,
        total_files,
    })
}

fn load_decision(path: &Path, matter: &Matter<YAML>) -> Result<Decision> {
    let content = std::fs::read_to_string(path).context("Failed to read file")?;

    let parsed = matter.parse(&content);

    let metadata: DecisionMetadata = parsed
        .data
        .ok_or_else(|| anyhow::anyhow!("Missing YAML frontmatter"))?
        .deserialize()
        .context("Failed to parse frontmatter")?;

    let body = parsed.content;
    let content_hash = hash_content(&content);

    Ok(Decision {
        metadata,
        body,
        file_path: path.to_path_buf(),
        content_hash,
        embedding: None,
    })
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_decision(dir: &Path, filename: &str, content: &str) {
        let path = dir.join(filename);
        let mut file = std::fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    #[test]
    fn test_load_valid_decision() {
        let dir = TempDir::new().unwrap();
        create_test_decision(
            dir.path(),
            "001-test.md",
            r#"---
id: 1
title: Test Decision
status: accepted
date: 2026-01-05
deciders:
  - Alice
tags:
  - testing
---

# Context

This is a test decision.
"#,
        );

        let decisions = load_decisions(dir.path()).unwrap();
        assert_eq!(decisions.len(), 1);
        assert_eq!(decisions[0].metadata.id, 1);
        assert_eq!(decisions[0].metadata.title, "Test Decision");
    }

    #[test]
    fn test_skip_malformed_decision() {
        let dir = TempDir::new().unwrap();
        create_test_decision(dir.path(), "001-bad.md", "No frontmatter here");

        let decisions = load_decisions(dir.path()).unwrap();
        assert_eq!(decisions.len(), 0);
    }

    #[test]
    fn test_missing_directory() {
        let result = load_decisions(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_load_decisions_with_errors_reports_invalid_files() {
        let dir = TempDir::new().unwrap();
        create_test_decision(
            dir.path(),
            "001-good.md",
            r#"---
id: 1
title: Good Decision
status: accepted
date: 2026-01-05
---

Body
"#,
        );
        create_test_decision(dir.path(), "002-bad.md", "No frontmatter here");

        let report = load_decisions_with_errors(dir.path()).unwrap();
        assert_eq!(report.total_files, 2);
        assert_eq!(report.decisions.len(), 1);
        assert_eq!(report.errors.len(), 1);
        assert!(report.errors[0].message.contains("frontmatter"));
    }

    #[test]
    fn test_load_decision_with_relationships() {
        let dir = TempDir::new().unwrap();
        create_test_decision(
            dir.path(),
            "005-supersedes.md",
            r#"---
id: 5
title: PostgreSQL Selection
status: accepted
date: 2026-01-06
supersedes: 2
amends: [3, 4]
depends_on: 1
related_to: [6, 7]
---

# Decision

We will use PostgreSQL.
"#,
        );

        let decisions = load_decisions(dir.path()).unwrap();
        assert_eq!(decisions.len(), 1);

        let decision = &decisions[0];
        assert_eq!(decision.metadata.id, 5);

        let rels = decision.metadata.relationships();
        assert_eq!(rels.len(), 6);

        use crate::types::RelationType;
        assert!(
            rels.iter()
                .any(|r| r.relation_type == RelationType::Supersedes && r.target_id == 2)
        );
        assert!(
            rels.iter()
                .any(|r| r.relation_type == RelationType::Amends && r.target_id == 3)
        );
        assert!(
            rels.iter()
                .any(|r| r.relation_type == RelationType::Amends && r.target_id == 4)
        );
        assert!(
            rels.iter()
                .any(|r| r.relation_type == RelationType::DependsOn && r.target_id == 1)
        );
        assert!(
            rels.iter()
                .any(|r| r.relation_type == RelationType::RelatedTo && r.target_id == 6)
        );
    }
}
