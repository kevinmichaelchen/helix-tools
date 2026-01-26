use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityKind {
    Decision,
    Issue,
    Idea,
    Report,
    Source,
    Citation,
    Agent,
    Session,
}

impl EntityKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Decision => "decision",
            Self::Issue => "issue",
            Self::Idea => "idea",
            Self::Report => "report",
            Self::Source => "source",
            Self::Citation => "citation",
            Self::Agent => "agent",
            Self::Session => "session",
        }
    }

    #[must_use]
    pub const fn directory_name(self) -> &'static str {
        match self {
            Self::Decision => "decisions",
            Self::Issue => "issues",
            Self::Idea => "ideas",
            Self::Report => "reports",
            Self::Source => "sources",
            Self::Citation => "citations",
            Self::Agent => "agents",
            Self::Session => "sessions",
        }
    }

    #[must_use]
    pub const fn id_prefix(self) -> &'static str {
        match self {
            Self::Decision => "dec",
            Self::Issue => "iss",
            Self::Idea => "idea",
            Self::Report => "rpt",
            Self::Source => "src",
            Self::Citation => "cite",
            Self::Agent => "agt",
            Self::Session => "ses",
        }
    }
}

#[derive(Debug, Error)]
pub enum ParseEntityKindError {
    #[error("Unknown entity kind: {0}")]
    UnknownKind(String),
}

impl FromStr for EntityKind {
    type Err = ParseEntityKindError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase();
        match normalized.as_str() {
            "decision" | "decisions" => Ok(Self::Decision),
            "issue" | "issues" => Ok(Self::Issue),
            "idea" | "ideas" => Ok(Self::Idea),
            "report" | "reports" => Ok(Self::Report),
            "source" | "sources" => Ok(Self::Source),
            "citation" | "citations" => Ok(Self::Citation),
            "agent" | "agents" => Ok(Self::Agent),
            "session" | "sessions" => Ok(Self::Session),
            _ => Err(ParseEntityKindError::UnknownKind(s.to_string())),
        }
    }
}

#[must_use]
pub fn kind_from_id(id: &str) -> Option<EntityKind> {
    let (prefix, _) = id.split_once('-')?;
    match prefix {
        "dec" => Some(EntityKind::Decision),
        "iss" | "bd" => Some(EntityKind::Issue),
        "idea" => Some(EntityKind::Idea),
        "rpt" => Some(EntityKind::Report),
        "src" => Some(EntityKind::Source),
        "cite" => Some(EntityKind::Citation),
        "agt" => Some(EntityKind::Agent),
        "ses" => Some(EntityKind::Session),
        _ => None,
    }
}

#[must_use]
pub fn looks_like_entity_id(id: &str) -> bool {
    crate::id::parse_id(id).is_ok()
}
