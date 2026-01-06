//! Local embeddings using fastembed.

use anyhow::Result;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

/// Wrapper around fastembed for generating embeddings.
pub struct Embedder {
    model: TextEmbedding,
}

impl Embedder {
    /// Create a new embedder with the default model (`AllMiniLML6V2`).
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(InitOptions::new(EmbeddingModel::AllMiniLML6V2))?;
        Ok(Self { model })
    }

    /// Embed a single text string.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text], None)?;
        embeddings
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No embedding returned"))
    }

    /// Embed multiple texts in a batch.
    #[allow(dead_code)]
    pub fn embed_batch(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        self.model.embed(texts, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires downloading model"]
    fn test_embed_text() {
        let embedder = Embedder::new().unwrap();
        let embedding = embedder.embed("Hello, world!").unwrap();

        // AllMiniLML6V2 produces 384-dimensional embeddings
        assert_eq!(embedding.len(), 384);
    }
}
