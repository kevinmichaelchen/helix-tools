use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{HelixDocsError, Result};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub github_token: Option<String>,
    pub db_path: PathBuf,
    pub ingest: IngestConfig,
    pub search: SearchConfig,
    pub freshness: FreshnessConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestConfig {
    pub concurrency: usize,
    pub extensions: Vec<String>,
}

impl Default for IngestConfig {
    fn default() -> Self {
        Self {
            concurrency: 5,
            extensions: vec![
                "md".to_string(),
                "mdx".to_string(),
                "txt".to_string(),
                "rst".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    pub default_mode: String,
    pub default_limit: usize,
    pub rrf_k: f32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_mode: "hybrid".to_string(),
            default_limit: 10,
            rrf_k: 60.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessConfig {
    pub stale_days: u32,
    pub use_etag: bool,
}

impl Default for FreshnessConfig {
    fn default() -> Self {
        Self {
            stale_days: 7,
            use_etag: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let global = Self::load_global()?;
        let project = Self::load_project()?;
        let merged = Self::merge(global, project);
        Ok(merged.with_env_overrides())
    }

    fn load_global() -> Result<Self> {
        let config_dir = directories::ProjectDirs::from("", "", "helix").map_or_else(
            || PathBuf::from("~/.config/helix"),
            |d| d.config_dir().to_path_buf(),
        );

        let config_path = config_dir.join("config.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content).map_err(|e| HelixDocsError::Config(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    fn load_project() -> Result<Self> {
        let config_path = PathBuf::from(".helix/helix-docs.toml");

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content).map_err(|e| HelixDocsError::Config(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    fn merge(global: Self, project: Self) -> Self {
        Self {
            github_token: project.github_token.or(global.github_token),
            db_path: if project.db_path.as_os_str().is_empty() {
                global.db_path
            } else {
                project.db_path
            },
            ingest: project.ingest,
            search: project.search,
            freshness: project.freshness,
        }
    }

    fn with_env_overrides(mut self) -> Self {
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            self.github_token = Some(token);
        }
        self
    }

    pub fn detect_github_token() -> Option<String> {
        if let Ok(token) = std::env::var("GITHUB_TOKEN") {
            return Some(token);
        }

        std::process::Command::new("gh")
            .args(["auth", "token"])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    }
}
