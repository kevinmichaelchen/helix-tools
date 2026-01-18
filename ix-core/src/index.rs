use anyhow::Result;

use crate::entity::EntityKind;
use crate::repo::IxchelRepo;

#[derive(Debug, Default, Clone, Copy)]
pub struct SyncStats {
    pub scanned: u32,
    pub added: u32,
    pub modified: u32,
    pub deleted: u32,
    pub unchanged: u32,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub score: f32,
    pub id: String,
    pub kind: Option<EntityKind>,
    pub title: String,
}

pub trait IndexBackend: Send + Sync {
    fn sync(&mut self, repo: &IxchelRepo) -> Result<SyncStats>;
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchHit>>;
    fn health_check(&self) -> Result<()>;
}
