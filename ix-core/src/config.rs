use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Failed to write config file {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to serialize config: {source}")]
    Serialize {
        #[source]
        source: toml::ser::Error,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IxchelConfig {
    #[serde(default)]
    pub embedding: EmbeddingConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

impl Default for IxchelConfig {
    fn default() -> Self {
        Self {
            embedding: EmbeddingConfig::default(),
            storage: StorageConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    #[serde(default = "default_embedding_provider")]
    pub provider: String,
    #[serde(default = "default_embedding_model")]
    pub model: String,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default)]
    pub dimension: Option<usize>,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: default_embedding_provider(),
            model: default_embedding_model(),
            batch_size: default_batch_size(),
            dimension: None,
        }
    }
}

fn default_embedding_provider() -> String {
    "fastembed".to_string()
}

fn default_embedding_model() -> String {
    "BAAI/bge-small-en-v1.5".to_string()
}

const fn default_batch_size() -> usize {
    32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_storage_backend")]
    pub backend: String,

    /// Path relative to `.ixchel/` for rebuildable storage.
    #[serde(default = "default_storage_path")]
    pub path: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: default_storage_backend(),
            path: default_storage_path(),
        }
    }
}

fn default_storage_backend() -> String {
    "helixdb".to_string()
}

fn default_storage_path() -> String {
    "data/ixchel".to_string()
}

impl IxchelConfig {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let raw = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;

        toml::from_str(&raw).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })
    }

    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let raw =
            toml::to_string_pretty(self).map_err(|source| ConfigError::Serialize { source })?;
        std::fs::write(path, raw).map_err(|source| ConfigError::Write {
            path: path.to_path_buf(),
            source,
        })
    }
}
