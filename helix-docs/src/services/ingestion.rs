use std::sync::Arc;

use crate::domain::{Document, Source};
use crate::error::Result;
use crate::ports::{ChunkRepository, DocumentRepository, FetchClient, SourceRepository};

pub struct IngestionService<S, D, C, F>
where
    S: SourceRepository,
    D: DocumentRepository,
    C: ChunkRepository,
    F: FetchClient,
{
    source_repo: Arc<S>,
    doc_repo: Arc<D>,
    #[allow(dead_code)]
    chunk_repo: Arc<C>,
    fetch_client: Arc<F>,
}

impl<S, D, C, F> IngestionService<S, D, C, F>
where
    S: SourceRepository,
    D: DocumentRepository,
    C: ChunkRepository,
    F: FetchClient,
{
    pub const fn new(
        source_repo: Arc<S>,
        doc_repo: Arc<D>,
        chunk_repo: Arc<C>,
        fetch_client: Arc<F>,
    ) -> Self {
        Self {
            source_repo,
            doc_repo,
            chunk_repo,
            fetch_client,
        }
    }

    pub async fn ingest_all(&self, force: bool, _concurrency: usize) -> Result<IngestionResult> {
        let sources = self.source_repo.list().await?;
        let mut result = IngestionResult::default();

        for source in sources {
            match self.ingest_source(&source, force).await {
                Ok(source_result) => {
                    result.documents_fetched += source_result.documents_fetched;
                    result.documents_skipped += source_result.documents_skipped;
                    result.sources_processed += 1;
                }
                Err(e) => {
                    result
                        .errors
                        .push(format!("{}: {}", source.library_name(), e));
                }
            }
        }

        Ok(result)
    }

    pub async fn ingest_source(&self, source: &Source, _force: bool) -> Result<IngestionResult> {
        let mut result = IngestionResult::default();

        let paths = self.fetch_client.list_paths(source).await?;

        for path in paths {
            let fetched = self.fetch_client.fetch(source, &path).await?;

            let doc = Document::new(source.id.clone(), path, fetched.content);
            self.doc_repo.upsert(&doc).await?;
            result.documents_fetched += 1;
        }

        result.sources_processed = 1;
        Ok(result)
    }
}

#[derive(Debug, Default)]
pub struct IngestionResult {
    pub sources_processed: usize,
    pub documents_fetched: usize,
    pub documents_skipped: usize,
    pub chunks_created: usize,
    pub embeddings_generated: usize,
    pub errors: Vec<String>,
}
