use anyhow::Result;

pub trait StorageBackend: Send + Sync {
    fn health_check(&self) -> Result<()>;
}
