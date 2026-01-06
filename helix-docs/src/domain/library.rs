use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::SourceId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub source_id: SourceId,
    pub name: String,
    pub url: String,
    pub versions: Vec<Version>,
    pub document_count: usize,
    pub last_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub label: String,
    pub git_ref: Option<String>,
    pub document_count: usize,
}
