//! Pluggable embedding infrastructure for Ixchel.
//!
//! Supports multiple backends via feature flags:
//! - `fastembed` (default): ONNX-based, CPU-only
//! - `candle`: Hugging Face Candle, supports Metal/CUDA

use ix_config::{EmbeddingConfig, load_shared_config};
use std::sync::Mutex;
use thiserror::Error;

#[cfg(feature = "fastembed")]
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

#[cfg(feature = "candle")]
use candle_core::{Device, Tensor};
#[cfg(feature = "candle")]
use candle_nn::VarBuilder;
#[cfg(feature = "candle")]
use candle_transformers::models::bert::{BertModel, Config as BertConfig, DTYPE};
#[cfg(feature = "candle")]
use hf_hub::{Repo, RepoType, api::sync::Api};
#[cfg(feature = "candle")]
use tokenizers::{PaddingParams, PaddingStrategy, Tokenizer, TruncationParams, TruncationStrategy};

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

    #[error("Provider not available: {provider} (enable the '{feature}' feature)")]
    ProviderNotCompiled { provider: String, feature: String },
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

// =============================================================================
// FastEmbed Provider
// =============================================================================

#[cfg(feature = "fastembed")]
struct FastEmbedProvider {
    model: Mutex<TextEmbedding>,
    model_name: String,
    dimension: usize,
    batch_size: usize,
}

#[cfg(feature = "fastembed")]
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

#[cfg(feature = "fastembed")]
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

        let mut embedding = embeddings
            .into_iter()
            .next()
            .ok_or(EmbeddingError::EmptyResult)?;
        l2_normalize(&mut embedding);
        Ok(embedding)
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

        for embedding in &mut all_embeddings {
            l2_normalize(embedding);
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

#[cfg(feature = "fastembed")]
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

#[cfg(feature = "fastembed")]
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

fn l2_normalize(embedding: &mut [f32]) {
    let norm = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm <= 0.0 {
        return;
    }

    for x in embedding {
        *x /= norm;
    }
}

// =============================================================================
// Candle Provider
// =============================================================================

#[cfg(feature = "candle")]
struct CandleProvider {
    model: Mutex<BertModel>,
    tokenizer: Mutex<Tokenizer>,
    device: Device,
    model_name: String,
    dimension: usize,
    batch_size: usize,
}

#[cfg(feature = "candle")]
impl CandleProvider {
    fn new(config: &EmbeddingConfig) -> Result<Self> {
        let device = Self::select_device();
        let model_id = if config.model.is_empty() {
            "sentence-transformers/all-MiniLM-L6-v2"
        } else {
            &config.model
        };

        let (model, tokenizer, dimension) = Self::load_model(model_id, &device)?;

        if let Some(configured_dim) = config.dimension
            && configured_dim != dimension
        {
            return Err(EmbeddingError::DimensionMismatch {
                model: model_id.to_string(),
                expected: dimension,
                configured: configured_dim,
            });
        }

        Ok(Self {
            model: Mutex::new(model),
            tokenizer: Mutex::new(tokenizer),
            device,
            model_name: model_id.to_string(),
            dimension,
            batch_size: config.batch_size.max(1),
        })
    }

    #[allow(clippy::missing_const_for_fn)] // Can't be const when metal/cuda features call non-const fns
    fn select_device() -> Device {
        #[cfg(feature = "metal")]
        {
            Device::new_metal(0).unwrap_or(Device::Cpu)
        }
        #[cfg(all(feature = "cuda", not(feature = "metal")))]
        {
            Device::new_cuda(0).unwrap_or(Device::Cpu)
        }
        #[cfg(not(any(feature = "metal", feature = "cuda")))]
        {
            Device::Cpu
        }
    }

    fn load_model(model_id: &str, device: &Device) -> Result<(BertModel, Tokenizer, usize)> {
        let api = Api::new().map_err(|e| EmbeddingError::InitError(e.to_string()))?;
        let repo = api.repo(Repo::new(model_id.to_string(), RepoType::Model));

        // Download model files
        let config_path = repo
            .get("config.json")
            .map_err(|e| EmbeddingError::InitError(format!("Failed to get config: {e}")))?;
        let tokenizer_path = repo
            .get("tokenizer.json")
            .map_err(|e| EmbeddingError::InitError(format!("Failed to get tokenizer: {e}")))?;
        let weights_path = repo
            .get("model.safetensors")
            .or_else(|_| repo.get("pytorch_model.bin"))
            .map_err(|e| EmbeddingError::InitError(format!("Failed to get weights: {e}")))?;

        // Load config
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| EmbeddingError::InitError(format!("Failed to read config: {e}")))?;
        let bert_config: BertConfig = serde_json::from_str(&config_str)
            .map_err(|e| EmbeddingError::InitError(format!("Failed to parse config: {e}")))?;
        let dimension = bert_config.hidden_size;

        // Load tokenizer with padding + truncation
        let mut tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| EmbeddingError::InitError(format!("Failed to load tokenizer: {e}")))?;
        tokenizer
            .with_truncation(Some(TruncationParams {
                max_length: bert_config.max_position_embeddings,
                strategy: TruncationStrategy::LongestFirst,
                ..Default::default()
            }))
            .map_err(|e| {
                EmbeddingError::InitError(format!("Failed to configure tokenizer truncation: {e}"))
            })?;
        tokenizer.with_padding(Some(PaddingParams {
            strategy: PaddingStrategy::BatchLongest,
            ..Default::default()
        }));

        // Load model weights
        // SAFETY: We just downloaded this file from HuggingFace Hub and trust its contents.
        // Memory-mapping provides significant performance benefits for large model files.
        #[allow(unsafe_code)]
        let vb = if weights_path
            .extension()
            .is_some_and(|ext| ext == "safetensors")
        {
            unsafe {
                VarBuilder::from_mmaped_safetensors(&[weights_path], DTYPE, device).map_err(
                    |e| EmbeddingError::InitError(format!("Failed to load weights: {e}")),
                )?
            }
        } else {
            VarBuilder::from_pth(&weights_path, DTYPE, device)
                .map_err(|e| EmbeddingError::InitError(format!("Failed to load weights: {e}")))?
        };

        let model = BertModel::load(vb, &bert_config)
            .map_err(|e| EmbeddingError::InitError(format!("Failed to build model: {e}")))?;

        Ok((model, tokenizer, dimension))
    }

    fn embed_tokens(&self, token_ids: &Tensor, attention_mask: &Tensor) -> Result<Tensor> {
        let token_type_ids = token_ids
            .zeros_like()
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        // Hold the lock only for the forward pass
        let embeddings = self
            .model
            .lock()
            .map_err(|_| EmbeddingError::ProviderUnavailable("model lock poisoned".to_string()))?
            .forward(token_ids, &token_type_ids, Some(attention_mask))
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        // Mean pooling with attention mask
        let mask_expanded = attention_mask
            .unsqueeze(2)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .broadcast_as(embeddings.shape())
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .to_dtype(embeddings.dtype())
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        let masked = embeddings
            .mul(&mask_expanded)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        let summed = masked
            .sum(1)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        let mask_sum = mask_expanded
            .sum(1)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .clamp(1e-9, f64::MAX)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        let pooled = summed
            .div(&mask_sum)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        // L2 normalize
        let norm = pooled
            .sqr()
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .sum(1)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .sqrt()
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .unsqueeze(1)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .clamp(1e-9, f64::MAX)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .broadcast_as(pooled.shape())
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        pooled
            .div(&norm)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))
    }
}

#[cfg(feature = "candle")]
impl EmbeddingProvider for CandleProvider {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let encoding = self
            .tokenizer
            .lock()
            .map_err(|_| {
                EmbeddingError::ProviderUnavailable("tokenizer lock poisoned".to_string())
            })?
            .encode(text, true)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        let ids: Vec<u32> = encoding.get_ids().to_vec();
        let mask: Vec<u32> = encoding.get_attention_mask().to_vec();

        let token_ids = Tensor::new(&ids[..], &self.device)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .unsqueeze(0)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        let attention_mask = Tensor::new(&mask[..], &self.device)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .unsqueeze(0)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

        let embeddings = self.embed_tokens(&token_ids, &attention_mask)?;

        embeddings
            .squeeze(0)
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
            .to_vec1()
            .map_err(|e| EmbeddingError::EmbedError(e.to_string()))
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_embeddings = Vec::with_capacity(texts.len());

        for chunk in texts.chunks(self.batch_size) {
            let encodings = self
                .tokenizer
                .lock()
                .map_err(|_| {
                    EmbeddingError::ProviderUnavailable("tokenizer lock poisoned".to_string())
                })?
                .encode_batch(chunk.to_vec(), true)
                .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

            let batch_len = encodings.len();
            let seq_len = encodings
                .iter()
                .map(tokenizers::Encoding::len)
                .max()
                .unwrap_or(0);

            let mut ids_flat: Vec<u32> = Vec::with_capacity(batch_len * seq_len);
            let mut mask_flat: Vec<u32> = Vec::with_capacity(batch_len * seq_len);

            for enc in &encodings {
                ids_flat.extend(enc.get_ids());
                mask_flat.extend(enc.get_attention_mask());
            }

            let token_ids = Tensor::new(&ids_flat[..], &self.device)
                .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
                .reshape((batch_len, seq_len))
                .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

            let attention_mask = Tensor::new(&mask_flat[..], &self.device)
                .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?
                .reshape((batch_len, seq_len))
                .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

            let embeddings = self.embed_tokens(&token_ids, &attention_mask)?;

            let batch_embeddings: Vec<Vec<f32>> = embeddings
                .to_vec2()
                .map_err(|e| EmbeddingError::EmbedError(e.to_string()))?;

            all_embeddings.extend(batch_embeddings);
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
        #[cfg(feature = "metal")]
        {
            "candle-metal"
        }
        #[cfg(all(feature = "cuda", not(feature = "metal")))]
        {
            "candle-cuda"
        }
        #[cfg(not(any(feature = "metal", feature = "cuda")))]
        {
            "candle-cpu"
        }
    }

    fn batch_size(&self) -> usize {
        self.batch_size
    }
}

// =============================================================================
// Provider Factory
// =============================================================================

fn provider_from_config(config: &EmbeddingConfig) -> Result<Box<dyn EmbeddingProvider>> {
    let provider = config.provider.trim().to_lowercase();
    match provider.as_str() {
        #[cfg(feature = "fastembed")]
        "fastembed" | "fastembed-rs" => Ok(Box::new(FastEmbedProvider::new(config)?)),

        #[cfg(not(feature = "fastembed"))]
        "fastembed" | "fastembed-rs" => Err(EmbeddingError::ProviderNotCompiled {
            provider: "fastembed".to_string(),
            feature: "fastembed".to_string(),
        }),

        #[cfg(feature = "candle")]
        "candle" | "candle-rs" => Ok(Box::new(CandleProvider::new(config)?)),

        #[cfg(not(feature = "candle"))]
        "candle" | "candle-rs" => Err(EmbeddingError::ProviderNotCompiled {
            provider: "candle".to_string(),
            feature: "candle".to_string(),
        }),

        _ => Err(EmbeddingError::UnknownProvider(config.provider.clone())),
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "fastembed")]
    fn test_fastembed_model_from_string() {
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

    #[test]
    #[cfg(feature = "candle")]
    #[ignore = "Requires downloading model (~90MB)"]
    fn test_candle_embed_text() {
        let config = EmbeddingConfig {
            provider: "candle".to_string(),
            model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            ..Default::default()
        };
        let embedder = Embedder::with_config(&config).unwrap();
        let embedding = embedder.embed("Hello, world!").unwrap();
        assert_eq!(embedding.len(), 384);
        assert!(embedder.provider_name().starts_with("candle"));
    }

    #[test]
    #[cfg(feature = "candle")]
    #[ignore = "Requires downloading model (~90MB)"]
    fn test_candle_embed_batch() {
        let config = EmbeddingConfig {
            provider: "candle".to_string(),
            model: "sentence-transformers/all-MiniLM-L6-v2".to_string(),
            batch_size: 2,
            ..Default::default()
        };
        let embedder = Embedder::with_config(&config).unwrap();
        let embeddings = embedder
            .embed_batch(&["First text", "Second text", "Third text"])
            .unwrap();
        assert_eq!(embeddings.len(), 3);
        assert!(embeddings.iter().all(|e| e.len() == 384));
    }

    #[test]
    #[cfg(feature = "candle")]
    #[ignore = "Requires downloading model (~1.3GB)"]
    fn test_candle_bge_large() {
        let config = EmbeddingConfig {
            provider: "candle".to_string(),
            model: "BAAI/bge-large-en-v1.5".to_string(),
            batch_size: 8,
            ..Default::default()
        };
        let embedder = Embedder::with_config(&config).unwrap();

        // Verify model loaded correctly
        assert_eq!(embedder.dimension(), 1024);
        assert_eq!(embedder.model_name(), "BAAI/bge-large-en-v1.5");

        // Test single embedding
        let embedding = embedder.embed("Hello, world!").unwrap();
        assert_eq!(embedding.len(), 1024);

        // Test batch embedding
        let embeddings = embedder
            .embed_batch(&["First text", "Second text"])
            .unwrap();
        assert_eq!(embeddings.len(), 2);
        assert!(embeddings.iter().all(|e| e.len() == 1024));
    }
}
