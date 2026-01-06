//! Core types for decision graph.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Proposed,
    Accepted,
    Superseded,
    Deprecated,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Proposed => write!(f, "proposed"),
            Self::Accepted => write!(f, "accepted"),
            Self::Superseded => write!(f, "superseded"),
            Self::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl std::str::FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "superseded" => Ok(Self::Superseded),
            "deprecated" => Ok(Self::Deprecated),
            _ => Err(format!("Invalid status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    Supersedes,
    Amends,
    DependsOn,
    RelatedTo,
}

impl RelationType {
    #[must_use]
    pub const fn as_edge_label(&self) -> &'static str {
        match self {
            Self::Supersedes => "SUPERSEDES",
            Self::Amends => "AMENDS",
            Self::DependsOn => "DEPENDS_ON",
            Self::RelatedTo => "RELATED_TO",
        }
    }

    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Supersedes => "supersedes",
            Self::Amends => "amends",
            Self::DependsOn => "depends on",
            Self::RelatedTo => "related to",
        }
    }
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub relation_type: RelationType,
    pub target_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

impl<T: Clone> OneOrMany<T> {
    #[must_use]
    pub fn to_vec(&self) -> Vec<T> {
        match self {
            Self::One(v) => vec![v.clone()],
            Self::Many(v) => v.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMetadata {
    pub id: u32,
    #[serde(default)]
    pub uuid: Option<String>,
    pub title: String,
    pub status: Status,
    pub date: NaiveDate,
    #[serde(default)]
    pub deciders: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub content_hash: Option<String>,
    #[serde(default)]
    pub git_commit: Option<String>,

    #[serde(default)]
    pub supersedes: Option<OneOrMany<u32>>,
    #[serde(default)]
    pub superseded_by: Option<u32>,
    #[serde(default)]
    pub amends: Option<OneOrMany<u32>>,
    #[serde(default)]
    pub depends_on: Option<OneOrMany<u32>>,
    #[serde(default)]
    pub related_to: Option<OneOrMany<u32>>,
}

impl DecisionMetadata {
    #[must_use]
    pub fn relationships(&self) -> Vec<Relationship> {
        let mut rels = Vec::new();

        if let Some(ref ids) = self.supersedes {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::Supersedes,
                    target_id: id,
                });
            }
        }
        if let Some(ref ids) = self.amends {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::Amends,
                    target_id: id,
                });
            }
        }
        if let Some(ref ids) = self.depends_on {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::DependsOn,
                    target_id: id,
                });
            }
        }
        if let Some(ref ids) = self.related_to {
            for id in ids.to_vec() {
                rels.push(Relationship {
                    relation_type: RelationType::RelatedTo,
                    target_id: id,
                });
            }
        }

        rels
    }
}

#[derive(Debug, Clone)]
pub struct Decision {
    pub metadata: DecisionMetadata,
    pub body: String,
    pub file_path: PathBuf,
    pub content_hash: String,
    pub embedding: Option<Vec<f32>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    pub id: u32,
    pub uuid: Option<String>,
    pub title: String,
    pub status: Status,
    pub score: f32,
    pub tags: Vec<String>,
    pub date: NaiveDate,
    pub deciders: Vec<String>,
    pub file_path: PathBuf,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<RelatedDecision>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelatedDecision {
    pub id: u32,
    pub title: String,
    pub relation: RelationType,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub query: String,
    pub count: usize,
    pub results: Vec<SearchResult>,
}

#[derive(Debug, Serialize)]
pub struct ChainResponse {
    pub root_id: u32,
    pub chain: Vec<ChainNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainNode {
    pub id: u32,
    pub title: String,
    pub status: Status,
    pub date: NaiveDate,
    pub is_current: bool,
}

#[derive(Debug, Serialize)]
pub struct RelatedResponse {
    pub decision_id: u32,
    pub related: Vec<RelatedDecision>,
}
