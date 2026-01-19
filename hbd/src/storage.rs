use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::{HbdError, Result};
use crate::id::matches_partial;
use crate::markdown;
use crate::types::Issue;

const ISSUE_DIR: &str = "issues";

pub struct TicketStore {
    root: PathBuf,
}

impl TicketStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn from_current_dir() -> Result<Self> {
        let root = find_project_root()?;
        Ok(Self::new(root))
    }

    pub fn tickets_dir(&self) -> PathBuf {
        self.root
            .join(ix_core::paths::IXCHEL_DIR_NAME)
            .join(ISSUE_DIR)
    }

    pub fn is_initialized(&self) -> bool {
        self.root.join(ix_core::paths::IXCHEL_DIR_NAME).exists()
    }

    pub fn init(&self) -> Result<()> {
        if self.is_initialized() {
            return Err(HbdError::AlreadyInitialized(self.root.clone()));
        }

        ix_core::repo::IxchelRepo::init_at(&self.root, false)
            .map_err(|e| HbdError::Other(e.to_string()))?;

        Ok(())
    }

    pub fn issue_path(&self, id: &str) -> PathBuf {
        self.tickets_dir().join(format!("{id}.md"))
    }

    pub fn read_issue(&self, id: &str) -> Result<Issue> {
        let path = self.issue_path(id);
        if !path.exists() {
            return Err(HbdError::IssueNotFound(id.to_string()));
        }
        let content = fs::read_to_string(&path)?;
        markdown::parse_issue(&content, &path)
    }

    pub fn write_issue(&self, issue: &Issue) -> Result<()> {
        if !self.is_initialized() {
            return Err(HbdError::NotInitialized);
        }

        let path = self.issue_path(&issue.id);
        let content = markdown::render_issue(issue);
        fs::write(path, content)?;
        Ok(())
    }

    pub fn delete_issue(&self, id: &str) -> Result<()> {
        let path = self.issue_path(id);
        if !path.exists() {
            return Err(HbdError::IssueNotFound(id.to_string()));
        }
        fs::remove_file(path)?;
        Ok(())
    }

    pub fn list_issue_ids(&self) -> Result<Vec<String>> {
        if !self.is_initialized() {
            return Err(HbdError::NotInitialized);
        }

        let mut ids = Vec::new();
        for entry in fs::read_dir(self.tickets_dir())? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "md")
                && let Some(stem) = path.file_stem()
            {
                ids.push(stem.to_string_lossy().into_owned());
            }
        }
        ids.sort();
        Ok(ids)
    }

    pub fn read_all_issues(&self) -> Result<Vec<Issue>> {
        let ids = self.list_issue_ids()?;
        let mut issues = Vec::with_capacity(ids.len());
        for id in ids {
            match self.read_issue(&id) {
                Ok(issue) => issues.push(issue),
                Err(e) => {
                    tracing::warn!("skipping malformed issue {id}: {e}");
                }
            }
        }
        Ok(issues)
    }

    pub fn read_all_issues_map(&self) -> Result<HashMap<String, Issue>> {
        let issues = self.read_all_issues()?;
        Ok(issues.into_iter().map(|i| (i.id.clone(), i)).collect())
    }

    pub fn resolve_id(&self, partial: &str) -> Result<String> {
        let ids = self.list_issue_ids()?;
        let matches: Vec<_> = ids
            .iter()
            .filter(|id| matches_partial(id, partial))
            .collect();

        match matches.len() {
            0 => Err(HbdError::IssueNotFound(partial.to_string())),
            1 => Ok(matches[0].clone()),
            _ => Err(HbdError::AmbiguousId(
                partial.to_string(),
                matches.into_iter().cloned().collect(),
            )),
        }
    }

    pub fn exists(&self, id: &str) -> bool {
        self.issue_path(id).exists()
    }
}

fn find_project_root() -> Result<PathBuf> {
    let current_dir = std::env::current_dir()?;
    ix_core::paths::find_git_root(&current_dir)
        .ok_or_else(|| HbdError::Other("not inside a git repository".to_string()))
}
