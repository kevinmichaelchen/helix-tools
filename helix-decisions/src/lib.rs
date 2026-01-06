//! # helix-decisions
//!
//! Decision graph infrastructure: semantic search, relationship tracking, and immutable records.
//!
//! ## Example
//!
//! ```no_run
//! use helix_decisions::{DecisionSearcher, Status};
//! use std::path::Path;
//!
//! # fn main() -> anyhow::Result<()> {
//! let mut searcher = DecisionSearcher::new()?;
//! searcher.sync(Path::new(".decisions/"))?;
//!
//! let results = searcher.search("database migration", 10, None, None)?;
//! for result in results.results {
//!     println!("{}: {} (score: {:.2})", result.id, result.title, result.score);
//! }
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod delta;
pub mod embeddings;
pub mod hooks;
pub mod loader;
pub mod searcher;
pub mod storage;
pub mod types;

pub use searcher::DecisionSearcher;
pub use types::{
    ChainNode, ChainResponse, Decision, RelatedDecision, RelatedResponse, RelationType,
    Relationship, SearchResponse, SearchResult, Status,
};
