use std::sync::Arc;

use crate::domain::{Library, Source, Version};
use crate::error::Result;
use crate::ports::{DocumentRepository, SourceRepository};

pub struct LibraryService<S, D>
where
    S: SourceRepository,
    D: DocumentRepository,
{
    source_repo: Arc<S>,
    doc_repo: Arc<D>,
}

impl<S, D> LibraryService<S, D>
where
    S: SourceRepository,
    D: DocumentRepository,
{
    pub const fn new(source_repo: Arc<S>, doc_repo: Arc<D>) -> Self {
        Self {
            source_repo,
            doc_repo,
        }
    }

    pub async fn find(&self, pattern: &str) -> Result<Vec<Library>> {
        let sources = self.source_repo.list().await?;
        let pattern_lower = pattern.to_lowercase();

        let mut libraries = Vec::new();

        for source in sources {
            let name = source.library_name();
            if name.to_lowercase().contains(&pattern_lower) {
                let docs = self.doc_repo.list_by_source(&source.id).await?;
                let library = Library {
                    source_id: source.id.clone(),
                    name,
                    url: source.url.clone(),
                    versions: Self::collect_versions(&source, &docs),
                    document_count: docs.len(),
                    last_synced_at: source.last_synced_at,
                };
                libraries.push(library);
            }
        }

        Ok(libraries)
    }

    fn collect_versions(source: &Source, docs: &[crate::domain::Document]) -> Vec<Version> {
        use std::collections::HashMap;

        let mut version_counts: HashMap<String, usize> = HashMap::new();

        for doc in docs {
            if let Some(v) = &doc.version {
                *version_counts.entry(v.clone()).or_default() += 1;
            }
        }

        if version_counts.is_empty()
            && let Some(v) = &source.config.version
        {
            version_counts.insert(v.clone(), docs.len());
        }

        version_counts
            .into_iter()
            .map(|(label, count)| Version {
                label,
                git_ref: source.config.git_ref.clone(),
                document_count: count,
            })
            .collect()
    }
}
