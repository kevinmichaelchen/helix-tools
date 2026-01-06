use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use helix_config::{EmbeddingConfig, load_shared_config};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("Failed to initialize embedding model: {0}")]
    InitError(String),

    #[error("Failed to generate embedding: {0}")]
    EmbedError(String),

    #[error("No embedding returned for input")]
    EmptyResult,

    #[error("Unknown model: {0}")]
    UnknownModel(String),
}

pub type Result<T> = std::result::Result<T, EmbeddingError>;

pub struct Embedder {
    model: TextEmbedding,
    batch_size: usize,
}

impl Embedder {
    pub fn new() -> Result<Self> {
        let config = load_shared_config()
            .map(|c| c.embedding)
            .unwrap_or_default();
        Self::with_config(&config)
    }

    pub fn with_config(config: &EmbeddingConfig) -> Result<Self> {
        let embedding_model = model_from_string(&config.model)?;
        let model = TextEmbedding::try_new(InitOptions::new(embedding_model))
            .map_err(|e| EmbeddingError::InitError(e.to_string()))?;

        Ok(Self {
            model,
            batch_size: config.batch_size,
        })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self
            .model
            .embed(vec![text], None)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        embeddings
            .into_iter()
            .next()
            .ok_or(EmbeddingError::EmptyResult)
    }

    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(self.batch_size) {
            let embeddings = self
                .model
                .embed(chunk.to_vec(), None)
                .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;
            all_embeddings.extend(embeddings);
        }

        Ok(all_embeddings)
    }

    #[must_use]
    pub const fn dimension(&self) -> usize {
        384
    }

    #[must_use]
    pub const fn batch_size(&self) -> usize {
        self.batch_size
    }
}

fn model_from_string(model_name: &str) -> Result<EmbeddingModel> {
    match model_name {
        "BAAI/bge-small-en-v1.5" | "bge-small-en-v1.5" => Ok(EmbeddingModel::BGESmallENV15),
        "BAAI/bge-base-en-v1.5" | "bge-base-en-v1.5" => Ok(EmbeddingModel::BGEBaseENV15),
        "BAAI/bge-large-en-v1.5" | "bge-large-en-v1.5" => Ok(EmbeddingModel::BGELargeENV15),
        "sentence-transformers/all-MiniLM-L6-v2" | "all-MiniLM-L6-v2" | "AllMiniLML6V2" => {
            Ok(EmbeddingModel::AllMiniLML6V2)
        }
        _ => Err(EmbeddingError::UnknownModel(model_name.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_from_string() {
        assert!(model_from_string("BAAI/bge-small-en-v1.5").is_ok());
        assert!(model_from_string("bge-small-en-v1.5").is_ok());
        assert!(model_from_string("all-MiniLM-L6-v2").is_ok());
        assert!(model_from_string("AllMiniLML6V2").is_ok());
        assert!(model_from_string("unknown-model").is_err());
    }

    #[test]
    #[ignore = "Requires downloading model (~30MB)"]
    fn test_embed_text() {
        let embedder = Embedder::new().unwrap();
        let embedding = embedder.embed("Hello, world!").unwrap();
        assert_eq!(embedding.len(), 384);
    }

    #[test]
    #[ignore = "Requires downloading model (~30MB)"]
    fn test_embed_batch() {
        let embedder = Embedder::new().unwrap();
        let embeddings = embedder
            .embed_batch(&["First text", "Second text", "Third text"])
            .unwrap();
        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|e| e.len() == 384));
    }

    #[test]
    fn test_embed_batch_empty() {
        let config = EmbeddingConfig::default();
        if let Ok(embedder) = Embedder::with_config(&config) {
            let result = embedder.embed_batch(&[]).unwrap();
            assert!(result.is_empty());
        }
    }
}
