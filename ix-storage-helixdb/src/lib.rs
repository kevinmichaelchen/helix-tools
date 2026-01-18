use anyhow::Result;

pub struct HelixDbStorage;

impl ix_core::storage::StorageBackend for HelixDbStorage {
    fn health_check(&self) -> Result<()> {
        Ok(())
    }
}
