//! YAML loader for family tree data.

use crate::error::{GotError, Result};
use crate::types::{House, Person};
use serde::Deserialize;
use std::path::Path;

/// The root structure of the family tree YAML file.
#[derive(Debug, Deserialize)]
pub struct FamilyTree {
    /// House definitions with metadata.
    #[serde(default)]
    pub houses: Vec<HouseInfo>,
    /// All people in the family tree.
    pub people: Vec<Person>,
    /// Relationship definitions.
    pub relationships: Vec<RelationshipDef>,
}

/// Information about a noble house.
#[derive(Debug, Deserialize)]
pub struct HouseInfo {
    pub name: String,
    #[serde(default)]
    pub seat: Option<String>,
    #[serde(default)]
    pub words: Option<String>,
}

/// A relationship definition from the YAML file.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RelationshipDef {
    /// Parent-child relationship (one parent, multiple children).
    ParentOf { from: String, to: Vec<String> },
    /// Spousal relationship (exactly two people).
    SpouseOf { between: Vec<String> },
    /// Sibling relationship (two or more people).
    SiblingOf { between: Vec<String> },
}

impl FamilyTree {
    /// Load a family tree from a YAML file.
    pub fn load(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path).map_err(|e| GotError::LoadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let tree: Self = serde_yaml::from_str(&contents)?;
        tree.validate()?;
        Ok(tree)
    }

    /// Validate the family tree data.
    fn validate(&self) -> Result<()> {
        let person_ids: std::collections::HashSet<_> =
            self.people.iter().map(|p| p.id.as_str()).collect();

        // Validate all relationship references
        for rel in &self.relationships {
            match rel {
                RelationshipDef::ParentOf { from, to } => {
                    if !person_ids.contains(from.as_str()) {
                        return Err(GotError::InvalidRelationship(format!(
                            "Unknown person in parent_of.from: {from}"
                        )));
                    }
                    for child in to {
                        if !person_ids.contains(child.as_str()) {
                            return Err(GotError::InvalidRelationship(format!(
                                "Unknown person in parent_of.to: {child}"
                            )));
                        }
                    }
                }
                RelationshipDef::SpouseOf { between } | RelationshipDef::SiblingOf { between } => {
                    for person in between {
                        if !person_ids.contains(person.as_str()) {
                            return Err(GotError::InvalidRelationship(format!(
                                "Unknown person in relationship: {person}"
                            )));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get a person by their ID.
    #[must_use]
    pub fn get_person(&self, id: &str) -> Option<&Person> {
        self.people.iter().find(|p| p.id == id)
    }

    /// Get all people belonging to a specific house.
    #[must_use]
    pub fn get_house_members(&self, house: House) -> Vec<&Person> {
        self.people.iter().filter(|p| p.house == house).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_relationship_def() {
        let yaml = r#"
type: parent_of
from: ned-stark
to:
  - robb-stark
  - sansa-stark
"#;
        let rel: RelationshipDef = serde_yaml::from_str(yaml).unwrap();
        match rel {
            RelationshipDef::ParentOf { from, to } => {
                assert_eq!(from, "ned-stark");
                assert_eq!(to.len(), 2);
            }
            _ => panic!("Expected ParentOf"),
        }
    }

    #[test]
    fn test_parse_spouse_relationship() {
        let yaml = r#"
type: spouse_of
between:
  - ned-stark
  - catelyn-stark
"#;
        let rel: RelationshipDef = serde_yaml::from_str(yaml).unwrap();
        match rel {
            RelationshipDef::SpouseOf { between } => {
                assert_eq!(between.len(), 2);
            }
            _ => panic!("Expected SpouseOf"),
        }
    }
}
