use async_trait::async_trait;

use crate::domain::Source;
use crate::error::Result;

#[async_trait]
pub trait FetchClient: Send + Sync {
    fn supports(&self, source: &Source) -> bool;
    async fn list_paths(&self, source: &Source) -> Result<Vec<String>>;
    async fn fetch(&self, source: &Source, path: &str) -> Result<FetchedDocument>;
    async fn check_freshness(
        &self,
        source: &Source,
        path: &str,
        etag: Option<&str>,
    ) -> Result<FreshnessCheck>;
}

#[derive(Debug, Clone)]
pub struct FetchedDocument {
    pub path: String,
    pub content: String,
    pub etag: Option<String>,
}

#[derive(Debug, Clone)]
pub enum FreshnessCheck {
    Fresh,
    Stale { new_etag: Option<String> },
    Unknown,
}
