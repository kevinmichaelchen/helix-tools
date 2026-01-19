use anyhow::{Context, Result};
use ix_core::index::{IndexBackend, SearchHit, SyncStats};
use ix_core::repo::IxchelRepo;

pub fn sync(repo: &IxchelRepo) -> Result<SyncStats> {
    match repo
        .config
        .storage
        .backend
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "helixdb" => {
            let mut index =
                ix_storage_helixdb::HelixDbIndex::open(repo).context("open helixdb index")?;
            IndexBackend::sync(&mut index, repo).context("sync helixdb index")
        }
        backend => anyhow::bail!("Unsupported storage backend: {backend}"),
    }
}

pub fn search(repo: &IxchelRepo, query: &str, limit: usize) -> Result<Vec<SearchHit>> {
    match repo
        .config
        .storage
        .backend
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "helixdb" => {
            let index =
                ix_storage_helixdb::HelixDbIndex::open(repo).context("open helixdb index")?;
            IndexBackend::search(&index, query, limit).context("search helixdb index")
        }
        backend => anyhow::bail!("Unsupported storage backend: {backend}"),
    }
}

pub fn health_check(repo: &IxchelRepo) -> Result<()> {
    match repo
        .config
        .storage
        .backend
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "helixdb" => {
            let index =
                ix_storage_helixdb::HelixDbIndex::open(repo).context("open helixdb index")?;
            IndexBackend::health_check(&index).context("helixdb health check")
        }
        backend => anyhow::bail!("Unsupported storage backend: {backend}"),
    }
}
