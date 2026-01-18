use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::entity::{EntityKind, kind_from_id};

pub const IXCHEL_DIR_NAME: &str = ".ixchel";

#[derive(Debug, Clone)]
pub struct IxchelPaths {
    repo_root: PathBuf,
}

impl IxchelPaths {
    #[must_use]
    pub fn new(repo_root: PathBuf) -> Self {
        Self { repo_root }
    }

    #[must_use]
    pub fn repo_root(&self) -> &Path {
        &self.repo_root
    }

    #[must_use]
    pub fn ixchel_dir(&self) -> PathBuf {
        self.repo_root.join(IXCHEL_DIR_NAME)
    }

    #[must_use]
    pub fn config_path(&self) -> PathBuf {
        self.ixchel_dir().join("config.toml")
    }

    #[must_use]
    pub fn data_dir(&self) -> PathBuf {
        self.ixchel_dir().join("data")
    }

    #[must_use]
    pub fn kind_dir(&self, kind: EntityKind) -> PathBuf {
        self.ixchel_dir().join(kind.directory_name())
    }

    #[must_use]
    pub fn entity_path(&self, id: &str) -> Option<PathBuf> {
        let kind = kind_from_id(id)?;
        Some(self.kind_dir(kind).join(format!("{id}.md")))
    }

    pub fn ensure_layout(&self) -> Result<()> {
        std::fs::create_dir_all(self.data_dir())
            .with_context(|| format!("Failed to create {}", self.data_dir().display()))?;

        for kind in [
            EntityKind::Decision,
            EntityKind::Issue,
            EntityKind::Idea,
            EntityKind::Report,
            EntityKind::Source,
            EntityKind::Citation,
            EntityKind::Agent,
            EntityKind::Session,
        ] {
            let dir = self.kind_dir(kind);
            std::fs::create_dir_all(&dir)
                .with_context(|| format!("Failed to create {}", dir.display()))?;
        }

        Ok(())
    }
}

#[must_use]
pub fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}
