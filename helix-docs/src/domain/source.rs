use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::SourceId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: SourceId,
    pub url: String,
    pub kind: SourceType,
    pub config: SourceConfig,
    pub created_at: DateTime<Utc>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub sync_status: SyncStatus,
}

impl Source {
    pub fn new_github(url: String, config: SourceConfig) -> Self {
        Self {
            id: SourceId::generate(),
            url,
            kind: SourceType::GitHub,
            config,
            created_at: Utc::now(),
            last_synced_at: None,
            sync_status: SyncStatus::Pending,
        }
    }

    pub fn library_name(&self) -> String {
        if self.kind == SourceType::GitHub {
            self.url
                .trim_start_matches("https://github.com/")
                .trim_end_matches('/')
                .to_string()
        } else {
            self.url.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceType {
    GitHub,
    Website,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceConfig {
    pub docs_path: Option<String>,
    pub git_ref: Option<String>,
    pub version: Option<String>,
    pub etag: Option<String>,
    pub crawl_depth: Option<u32>,
    pub max_pages: Option<u32>,
    pub allow_paths: Vec<String>,
    pub deny_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum SyncStatus {
    #[default]
    Pending,
    Syncing,
    Synced,
    Error(String),
}
