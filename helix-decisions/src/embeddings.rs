use anyhow::Result;
pub use helix_embeddings::Embedder;

pub fn create_embedder() -> Result<Embedder> {
    Embedder::new().map_err(|e| anyhow::anyhow!("Failed to create embedder: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "Requires downloading model (~30MB)"]
    fn test_embed_text() {
        let embedder = create_embedder().unwrap();
        let embedding = embedder.embed("Hello, world!").unwrap();
        assert_eq!(embedding.len(), 384);
    }
}
