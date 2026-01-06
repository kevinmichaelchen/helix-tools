use std::sync::Arc;

use crate::domain::{Source, SourceConfig, SourceId, SourceType};
use crate::error::{HelixDocsError, Result};
use crate::ports::SourceRepository;

pub struct SourceService<R: SourceRepository> {
    repo: Arc<R>,
}

impl<R: SourceRepository> SourceService<R> {
    pub const fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn add(&self, url: &str, config: SourceConfig) -> Result<Source> {
        if let Some(existing) = self.repo.get_by_url(url).await? {
            return Err(HelixDocsError::SourceExists(existing.id.to_string()));
        }

        let source_type = Self::detect_source_type(url)?;
        let source = match source_type {
            SourceType::GitHub => Source::new_github(url.to_string(), config),
            SourceType::Website => {
                return Err(HelixDocsError::Config(
                    "Website sources not yet implemented".to_string(),
                ));
            }
        };

        self.repo.create(&source).await?;
        Ok(source)
    }

    pub async fn list(&self) -> Result<Vec<Source>> {
        self.repo.list().await
    }

    pub async fn get(&self, id: &SourceId) -> Result<Option<Source>> {
        self.repo.get(id).await
    }

    pub async fn get_by_url(&self, url: &str) -> Result<Option<Source>> {
        self.repo.get_by_url(url).await
    }

    pub async fn remove(&self, id: &SourceId) -> Result<()> {
        self.repo.delete(id).await
    }

    fn detect_source_type(url: &str) -> Result<SourceType> {
        if url.starts_with("https://github.com/") || url.starts_with("http://github.com/") {
            Ok(SourceType::GitHub)
        } else if url.starts_with("https://") || url.starts_with("http://") {
            Ok(SourceType::Website)
        } else {
            Err(HelixDocsError::InvalidSourceUrl(url.to_string()))
        }
    }
}
