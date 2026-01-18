use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use helix_config::{EmbeddingConfig, load_shared_config};
use std::sync::Mutex;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("Failed to initialize embedding model: {0}")]
    InitError(String),

    #[error("Failed to generate embedding: {0}")]
    EmbedError(String),

    #[error("Embedding provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("No embedding returned for input")]
    EmptyResult,

    #[error("Unknown provider: {0}")]
    UnknownProvider(String),

    #[error("Unknown model: {0}")]
    UnknownModel(String),

    #[error(
        "Embedding dimension mismatch for model {model}: expected {expected}, configured {configured}"
    )]
    DimensionMismatch {
        model: String,
        expected: usize,
        configured: usize,
    },
}

pub type Result<T> = std::result::Result<T, EmbeddingError>;

pub trait EmbeddingProvider: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn dimension(&self) -> usize;
    fn model_name(&self) -> &str;
    fn provider_name(&self) -> &'static str;
    fn batch_size(&self) -> usize {
        1
    }
}

pub struct Embedder {
    provider: Box<dyn EmbeddingProvider>,
}

impl Embedder {
    pub fn new() -> Result<Self> {
        let config = load_shared_config()
            .map(|c| c.embedding)
            .unwrap_or_default();
        Self::with_config(&config)
    }

    pub fn with_config(config: &EmbeddingConfig) -> Result<Self> {
        let provider = provider_from_config(config)?;
        Ok(Self { provider })
    }

    pub fn from_provider(provider: Box<dyn EmbeddingProvider>) -> Self {
        Self { provider }
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.provider.embed(text)
    }

    pub fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.provider.embed_batch(texts)
    }

    #[must_use]
    pub fn dimension(&self) -> usize {
        self.provider.dimension()
    }

    #[must_use]
    pub fn batch_size(&self) -> usize {
        self.provider.batch_size()
    }

    #[must_use]
    pub fn model_name(&self) -> &str {
        self.provider.model_name()
    }

    #[must_use]
    pub fn provider_name(&self) -> &'static str {
        self.provider.provider_name()
    }
}

struct FastEmbedProvider {
    model: Mutex<TextEmbedding>,
    model_name: String,
    dimension: usize,
    batch_size: usize,
}

impl FastEmbedProvider {
    fn new(config: &EmbeddingConfig) -> Result<Self> {
        let embedding_model = fastembed_model_from_string(&config.model)?;
        let (model_name, dimension) = {
            let model_info = TextEmbedding::get_model_info(&embedding_model)
                .map_err(|e| EmbeddingError::UnknownModel(format!("{}: {e}", config.model)))?;

            if let Some(configured_dim) = config.dimension
                && configured_dim != model_info.dim
            {
                return Err(EmbeddingError::DimensionMismatch {
                    model: config.model.clone(),
                    expected: model_info.dim,
                    configured: configured_dim,
                });
            }

            (model_info.model_code.clone(), model_info.dim)
        };

        let model = TextEmbedding::try_new(InitOptions::new(embedding_model))
            .map_err(|e| EmbeddingError::InitError(e.to_string()))?;

        Ok(Self {
            model: Mutex::new(model),
            model_name,
            dimension,
            batch_size: config.batch_size.max(1),
        })
    }
}

impl EmbeddingProvider for FastEmbedProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = {
            let model = self.model.lock().map_err(|_| {
                EmbeddingError::ProviderUnavailable("model lock poisoned".to_string())
            })?;

            model
                .embed(vec![text], None)
                .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
        };

        embeddings
            .into_iter()
            .next()
            .ok_or(EmbeddingError::EmptyResult)
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(self.batch_size) {
            let embeddings = {
                let model = self.model.lock().map_err(|_| {
                    EmbeddingError::ProviderUnavailable("model lock poisoned".to_string())
                })?;

                model
                    .embed(chunk.to_vec(), None)
                    .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            };
            all_embeddings.extend(embeddings);
        }

        Ok(all_embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn provider_name(&self) -> &'static str {
        "fastembed"
    }

    fn batch_size(&self) -> usize {
        self.batch_size
    }
}

fn provider_from_config(config: &EmbeddingConfig) -> Result<Box<dyn EmbeddingProvider>> {
    let provider = config.provider.trim().to_lowercase();
    match provider.as_str() {
        "fastembed" | "fastembed-rs" => Ok(Box::new(FastEmbedProvider::new(config)?)),
        _ => Err(EmbeddingError::UnknownProvider(config.provider.clone())),
    }
}

fn fastembed_model_from_string(model_name: &str) -> Result<EmbeddingModel> {
    let trimmed = model_name.trim();
    if trimmed.is_empty() {
        return Err(EmbeddingError::UnknownModel(model_name.to_string()));
    }

    if let Ok(model) = trimmed.parse() {
        return Ok(model);
    }

    let needle = normalize_model_token(trimmed);
    let needle_suffix = normalize_model_token(trimmed.rsplit('/').next().unwrap_or(trimmed));

    for info in TextEmbedding::list_supported_models() {
        for candidate in model_identifiers(&info.model_code) {
            if candidate == needle || candidate == needle_suffix {
                return Ok(info.model);
            }
        }
    }

    Err(EmbeddingError::UnknownModel(model_name.to_string()))
}

fn model_identifiers(model_code: &str) -> Vec<String> {
    let normalized = normalize_model_token(model_code);
    let suffix = model_code.rsplit('/').next().unwrap_or(model_code);
    let suffix_normalized = normalize_model_token(suffix);

    let mut identifiers = vec![normalized, suffix_normalized];

    for value in [suffix.strip_suffix("-onnx"), suffix.strip_suffix("-onnx-q")]
        .into_iter()
        .flatten()
    {
        identifiers.push(normalize_model_token(value));
    }

    identifiers
}

fn normalize_model_token(value: &str) -> String {
    value
        .chars()
        .filter(char::is_ascii_alphanumeric)
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_from_string() {
        assert!(fastembed_model_from_string("BAAI/bge-small-en-v1.5").is_ok());
        assert!(fastembed_model_from_string("bge-small-en-v1.5").is_ok());
        assert!(fastembed_model_from_string("all-MiniLM-L6-v2").is_ok());
        assert!(fastembed_model_from_string("AllMiniLML6V2").is_ok());
        assert!(fastembed_model_from_string("unknown-model").is_err());
    }

    #[test]
    #[ignore = "Requires downloading model (~30MB)"]
    fn test_embed_text() {
        let embedder = Embedder::new().unwrap();
        let embedding = embedder.embed("Hello, world!").unwrap();
        assert_eq!(embedding.len(), embedder.dimension());
    }

    #[test]
    #[ignore = "Requires downloading model (~30MB)"]
    fn test_embed_batch() {
        let embedder = Embedder::new().unwrap();
        let embeddings = embedder
            .embed_batch(&["First text", "Second text", "Third text"])
            .unwrap();
        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|e| e.len() == embedder.dimension()));
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
