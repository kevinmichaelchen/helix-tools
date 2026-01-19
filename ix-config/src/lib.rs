use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use thiserror::Error;

/// Get the Ixchel home directory (`~/.ixchel` or `$IXCHEL_HOME`).
///
/// This is the root directory for all Ixchel global config/state/data.
///
/// # Environment Override
/// Set `IXCHEL_HOME` to override the default location.
#[must_use]
pub fn ixchel_home() -> PathBuf {
    if let Ok(home) = std::env::var("IXCHEL_HOME") {
        return PathBuf::from(home);
    }
    if let Ok(home) = std::env::var("HELIX_HOME") {
        return PathBuf::from(home);
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".ixchel")
}

/// Get the config directory (`~/.ixchel/config`).
///
/// Contains user-editable TOML configuration files.
#[must_use]
pub fn ixchel_config_dir() -> PathBuf {
    ixchel_home().join("config")
}

/// Get the data directory (`~/.ixchel/data`).
///
/// Contains caches and databases (auto-generated, safe to delete).
#[must_use]
pub fn ixchel_data_dir() -> PathBuf {
    ixchel_home().join("data")
}

/// Get the state directory (`~/.ixchel/state`).
///
/// Contains runtime metadata (agents, locks, ephemeral caches).
#[must_use]
pub fn ixchel_state_dir() -> PathBuf {
    ixchel_home().join("state")
}

/// Get the log directory (`~/.ixchel/log`).
///
/// Contains operation logs for debugging.
#[must_use]
pub fn ixchel_log_dir() -> PathBuf {
    ixchel_home().join("log")
}

/// Shared configuration used by multiple Ixchel tools.
///
/// Loaded from `~/.ixchel/config/config.toml` and `.ixchel/config.toml`.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct IxchelConfig {
    #[serde(default)]
    pub github: GitHubConfig,
    #[serde(default)]
    pub embedding: EmbeddingConfig,
    #[serde(default)]
    pub storage: StorageConfig,
}

pub type SharedConfig = IxchelConfig;

/// GitHub-related configuration.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct GitHubConfig {
    /// GitHub personal access token. Can also be set via `GITHUB_TOKEN` or `GH_TOKEN`.
    pub token: Option<String>,
}

/// Embedding model configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingConfig {
    /// Provider implementation to use (e.g. "fastembed").
    #[serde(default = "default_embedding_provider")]
    pub provider: String,
    /// The embedding model to use.
    #[serde(default = "default_embedding_model")]
    pub model: String,
    /// Batch size for embedding operations.
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    /// Optional dimension override for providers that don't advertise dims.
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

/// Storage configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StorageConfig {
    /// Storage backend to use (e.g. "helixdb").
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
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let raw = toml::to_string_pretty(self).map_err(|source| ConfigError::SerializeError {
            path: path.to_path_buf(),
            source,
        })?;
        std::fs::write(path, raw).map_err(|source| ConfigError::WriteError {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(())
    }
}

/// Load the shared configuration from global and project config files.
///
/// # Errors
/// Returns an error if config files exist but cannot be read or parsed.
pub fn load_shared_config() -> Result<SharedConfig, ConfigError> {
    ConfigLoader::new("").load()
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file {}: {source}", path.display())]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse config file {}: {source}", path.display())]
    ParseError {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Failed to write config file {}: {source}", path.display())]
    WriteError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to serialize config file {}: {source}", path.display())]
    SerializeError {
        path: PathBuf,
        #[source]
        source: toml::ser::Error,
    },
}

pub fn load_config<T: DeserializeOwned + Default>(tool_name: &str) -> Result<T, ConfigError> {
    ConfigLoader::new(tool_name).load()
}

pub struct ConfigLoader {
    tool_name: String,
    env_prefix: Option<String>,
    project_dir: Option<PathBuf>,
    global_dir: Option<PathBuf>,
}

impl ConfigLoader {
    pub fn new(tool_name: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            env_prefix: None,
            project_dir: None,
            global_dir: None,
        }
    }

    #[must_use]
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.env_prefix = Some(prefix.into());
        self
    }

    #[must_use]
    pub fn with_project_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.project_dir = Some(path.into());
        self
    }

    #[must_use]
    pub fn with_global_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.global_dir = Some(path.into());
        self
    }

    pub fn load<T: DeserializeOwned + Default>(self) -> Result<T, ConfigError> {
        let mut merged = toml::Table::new();

        let global_dir = self.global_dir.unwrap_or_else(ixchel_config_dir);

        let project_dir = self.project_dir.or_else(find_project_config_dir);
        if let Some(dir) = project_dir {
            if let Some(table) = load_toml_file(&dir.join("config.toml"))? {
                merge_tables(&mut merged, table);
            }

            if !self.tool_name.is_empty() {
                let tool_config = dir.join(format!("{}.toml", self.tool_name));
                if let Some(table) = load_toml_file(&tool_config)? {
                    merge_tables(&mut merged, table);
                }
            }
        }

        if let Some(table) = load_toml_file(&global_dir.join("config.toml"))? {
            merge_tables(&mut merged, table);
        }

        if !self.tool_name.is_empty() {
            let tool_config = global_dir.join(format!("{}.toml", self.tool_name));
            if let Some(table) = load_toml_file(&tool_config)? {
                merge_tables(&mut merged, table);
            }
        }

        if merged.is_empty() {
            return Ok(T::default());
        }

        let value = toml::Value::Table(merged);
        value.try_into().map_err(|e| ConfigError::ParseError {
            path: PathBuf::from("<merged>"),
            source: e,
        })
    }
}

fn load_toml_file(path: &Path) -> Result<Option<toml::Table>, ConfigError> {
    if !path.exists() {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path).map_err(|e| ConfigError::ReadError {
        path: path.to_path_buf(),
        source: e,
    })?;

    let table: toml::Table = toml::from_str(&content).map_err(|e| ConfigError::ParseError {
        path: path.to_path_buf(),
        source: e,
    })?;

    Ok(Some(table))
}

fn merge_tables(base: &mut toml::Table, overlay: toml::Table) {
    for (key, value) in overlay {
        match (base.get_mut(&key), value) {
            (Some(toml::Value::Table(base_table)), toml::Value::Table(overlay_table)) => {
                merge_tables(base_table, overlay_table);
            }
            (_, value) => {
                base.insert(key, value);
            }
        }
    }
}

/// Get the global config directory (`~/.ixchel/config`).
///
/// This is a convenience alias for [`ixchel_config_dir`].
#[must_use]
#[deprecated(since = "0.2.0", note = "use ixchel_config_dir() instead")]
pub fn global_config_dir() -> Option<PathBuf> {
    Some(ixchel_config_dir())
}

/// Find the project config directory (`.ixchel/`) by walking to git root.
///
/// Returns `None` if no `.ixchel/` directory exists at the repository root.
#[must_use]
pub fn find_project_config_dir() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let root = find_git_root(&cwd)?;
    let ixchel_dir = root.join(".ixchel");
    ixchel_dir.exists().then_some(ixchel_dir)
}

#[deprecated(since = "0.2.0", note = "use find_project_config_dir() instead")]
pub fn project_config_dir() -> Option<PathBuf> {
    find_project_config_dir()
}

#[must_use]
fn find_git_root(start: &Path) -> Option<PathBuf> {
    let mut current = Some(start);
    while let Some(dir) = current {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }
    None
}

/// Detect GitHub token from multiple sources.
///
/// Detection order (highest priority first):
/// 1. `GITHUB_TOKEN` environment variable
/// 2. `GH_TOKEN` environment variable
/// 3. `github.token` in config files
/// 4. `gh auth token` command output
#[must_use]
pub fn detect_github_token() -> Option<String> {
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        return Some(token);
    }

    if let Ok(token) = std::env::var("GH_TOKEN") {
        return Some(token);
    }

    if let Ok(config) = load_shared_config()
        && let Some(token) = config.github.token
    {
        return Some(token);
    }

    std::process::Command::new("gh")
        .args(["auth", "token"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Default, Deserialize, PartialEq)]
    struct TestConfig {
        #[serde(default)]
        value: i32,
        #[serde(default)]
        nested: NestedConfig,
    }

    #[derive(Debug, Default, Deserialize, PartialEq)]
    struct NestedConfig {
        #[serde(default)]
        inner: String,
    }

    #[test]
    fn test_merge_tables_simple() {
        let mut base: toml::Table = toml::toml! {
            value = 1
            other = "kept"
        };

        let overlay: toml::Table = toml::toml! {
            value = 2
        };

        merge_tables(&mut base, overlay);

        assert_eq!(base.get("value").unwrap().as_integer(), Some(2));
        assert_eq!(base.get("other").unwrap().as_str(), Some("kept"));
    }

    #[test]
    fn test_merge_tables_nested() {
        let mut base: toml::Table = toml::toml! {
            [nested]
            inner = "base"
            other = "kept"
        };

        let overlay: toml::Table = toml::toml! {
            [nested]
            inner = "overlay"
        };

        merge_tables(&mut base, overlay);

        let nested = base.get("nested").unwrap().as_table().unwrap();
        assert_eq!(nested.get("inner").unwrap().as_str(), Some("overlay"));
        assert_eq!(nested.get("other").unwrap().as_str(), Some("kept"));
    }

    #[test]
    fn test_load_missing_returns_default() {
        let config: TestConfig = ConfigLoader::new("nonexistent")
            .with_global_dir("/nonexistent/path")
            .with_project_dir("/nonexistent/path")
            .load()
            .unwrap();

        assert_eq!(config, TestConfig::default());
    }
}
