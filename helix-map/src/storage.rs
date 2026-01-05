use crate::model::Index;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub trait IndexStore {
    fn load(&self) -> Result<Option<Index>>;
    fn save(&self, index: &Index) -> Result<()>;
    fn path(&self) -> &Path;
}

#[derive(Debug, Clone)]
pub struct JsonStore {
    path: PathBuf,
}

impl JsonStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn for_root(root: &Path) -> Self {
        Self {
            path: root.join(".helix-map").join("index.json"),
        }
    }
}

impl IndexStore for JsonStore {
    fn load(&self) -> Result<Option<Index>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let data = fs::read(&self.path)
            .with_context(|| format!("failed to read index store at {}", self.path.display()))?;
        let index = serde_json::from_slice(&data)
            .with_context(|| format!("failed to parse index store at {}", self.path.display()))?;
        Ok(Some(index))
    }

    fn save(&self, index: &Index) -> Result<()> {
        let parent = self
            .path
            .parent()
            .context("index store path missing parent directory")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create index directory at {}", parent.display()))?;

        let tmp_path = self.path.with_extension("json.tmp");
        let data = serde_json::to_vec_pretty(index).context("failed to serialize index")?;
        fs::write(&tmp_path, data)
            .with_context(|| format!("failed to write temp index at {}", tmp_path.display()))?;

        if self.path.exists() {
            fs::remove_file(&self.path).with_context(|| {
                format!("failed to remove existing index at {}", self.path.display())
            })?;
        }

        fs::rename(&tmp_path, &self.path).with_context(|| {
            format!("failed to move index into place at {}", self.path.display())
        })?;
        Ok(())
    }

    fn path(&self) -> &Path {
        &self.path
    }
}
