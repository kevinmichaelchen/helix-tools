//! Game of Thrones family tree graph demo with HelixDB.
//!
//! This crate demonstrates HelixDB's graph capabilities using
//! the family relationships from Game of Thrones.

pub mod error;
pub mod loader;
pub mod query;
pub mod storage;
pub mod types;

pub use error::{GotError, Result};
pub use loader::{BioLoader, FamilyTree, PersonBio, RelationshipDef};
pub use query::{PersonFamily, find_ancestors, find_descendants, get_person_with_family};
pub use storage::{GotStorage, IngestStats};
pub use types::{
    AncestorNode, DescendantNode, GraphStats, House, Person, RelationType, SearchResult,
};
