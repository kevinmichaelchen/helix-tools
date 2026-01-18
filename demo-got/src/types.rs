//! Core types for Game of Thrones family tree.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Noble houses of Westeros.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum House {
    Stark,
    Targaryen,
    Baratheon,
    Tully,
    Lannister,
}

impl fmt::Display for House {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stark => write!(f, "Stark"),
            Self::Targaryen => write!(f, "Targaryen"),
            Self::Baratheon => write!(f, "Baratheon"),
            Self::Tully => write!(f, "Tully"),
            Self::Lannister => write!(f, "Lannister"),
        }
    }
}

impl std::str::FromStr for House {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stark" => Ok(Self::Stark),
            "targaryen" => Ok(Self::Targaryen),
            "baratheon" => Ok(Self::Baratheon),
            "tully" => Ok(Self::Tully),
            "lannister" => Ok(Self::Lannister),
            _ => Err(format!("Unknown house: {s}")),
        }
    }
}

/// A person in the family tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    /// Unique identifier (e.g., "jon-snow", "ned-stark").
    pub id: String,
    /// Full name (e.g., "Jon Snow", "Eddard Stark").
    pub name: String,
    /// Primary house affiliation.
    pub house: House,
    /// Titles held (e.g., "King in the North").
    #[serde(default)]
    pub titles: Vec<String>,
    /// Common alias or nickname (e.g., "The Mad King").
    #[serde(default)]
    pub alias: Option<String>,
    /// Whether the person is alive at series start.
    #[serde(default = "default_alive")]
    pub is_alive: bool,
}

const fn default_alive() -> bool {
    true
}

/// Types of family relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationType {
    /// Parent to child relationship (directional).
    ParentOf,
    /// Marriage relationship (bidirectional).
    SpouseOf,
    /// Sibling relationship (bidirectional).
    SiblingOf,
}

impl RelationType {
    /// Returns the edge label used in HelixDB.
    #[must_use]
    pub const fn as_edge_label(&self) -> &'static str {
        match self {
            Self::ParentOf => "PARENT_OF",
            Self::SpouseOf => "SPOUSE_OF",
            Self::SiblingOf => "SIBLING_OF",
        }
    }

    /// Returns whether this relationship type is bidirectional.
    #[must_use]
    pub const fn is_bidirectional(&self) -> bool {
        match self {
            Self::ParentOf => false,
            Self::SpouseOf | Self::SiblingOf => true,
        }
    }
}

impl fmt::Display for RelationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParentOf => write!(f, "parent of"),
            Self::SpouseOf => write!(f, "spouse of"),
            Self::SiblingOf => write!(f, "sibling of"),
        }
    }
}

/// An ancestor in a traversal result.
#[derive(Debug, Clone)]
pub struct AncestorNode {
    pub person: Person,
    pub depth: u32,
}

/// A descendant in a traversal result.
#[derive(Debug, Clone)]
pub struct DescendantNode {
    pub person: Person,
    pub depth: u32,
}

/// Statistics about the graph database.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub house_counts: std::collections::HashMap<String, usize>,
}

/// A search result from semantic search.
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    /// The matched person.
    pub person: Person,
    /// Similarity score (0.0 to 1.0, higher is more similar).
    pub score: f32,
    /// Optional snippet from the bio that matched.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}
