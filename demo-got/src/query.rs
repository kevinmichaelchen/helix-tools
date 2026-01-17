//! Graph traversal queries for the family tree.

use crate::error::{GotError, Result};
use crate::storage::GotStorage;
use crate::types::{AncestorNode, DescendantNode, Person, RelationType};
use std::collections::{HashSet, VecDeque};

/// Find all ancestors of a person using BFS traversal.
///
/// Follows PARENT_OF edges in reverse (from child to parents).
pub fn find_ancestors(
    storage: &GotStorage,
    person_id: &str,
    max_depth: usize,
) -> Result<Vec<AncestorNode>> {
    let start_node_id = storage
        .lookup_by_id(person_id)?
        .ok_or_else(|| GotError::PersonNotFound(person_id.to_string()))?;

    let mut ancestors = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    // Start with the person's parents (depth 1)
    visited.insert(start_node_id);

    // Get parents (incoming PARENT_OF edges)
    let parents = storage.get_incoming_neighbors(start_node_id, RelationType::ParentOf)?;
    for parent_id in parents {
        if visited.insert(parent_id) {
            queue.push_back((parent_id, 1u32));
        }
    }

    while let Some((current_id, depth)) = queue.pop_front() {
        if depth as usize > max_depth {
            continue;
        }

        let person = storage.get_person(current_id)?;
        ancestors.push(AncestorNode { person, depth });

        // Get this person's parents
        let parents = storage.get_incoming_neighbors(current_id, RelationType::ParentOf)?;
        for parent_id in parents {
            if visited.insert(parent_id) {
                queue.push_back((parent_id, depth + 1));
            }
        }
    }

    // Sort by depth for nice output
    ancestors.sort_by_key(|a| a.depth);

    Ok(ancestors)
}

/// Find all descendants of a person using BFS traversal.
///
/// Follows PARENT_OF edges forward (from parent to children).
pub fn find_descendants(
    storage: &GotStorage,
    person_id: &str,
    max_depth: usize,
) -> Result<Vec<DescendantNode>> {
    let start_node_id = storage
        .lookup_by_id(person_id)?
        .ok_or_else(|| GotError::PersonNotFound(person_id.to_string()))?;

    let mut descendants = Vec::new();
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();

    visited.insert(start_node_id);

    // Get children (outgoing PARENT_OF edges)
    let children = storage.get_outgoing_neighbors(start_node_id, RelationType::ParentOf)?;
    for child_id in children {
        if visited.insert(child_id) {
            queue.push_back((child_id, 1u32));
        }
    }

    while let Some((current_id, depth)) = queue.pop_front() {
        if depth as usize > max_depth {
            continue;
        }

        let person = storage.get_person(current_id)?;
        descendants.push(DescendantNode { person, depth });

        // Get this person's children
        let children = storage.get_outgoing_neighbors(current_id, RelationType::ParentOf)?;
        for child_id in children {
            if visited.insert(child_id) {
                queue.push_back((child_id, depth + 1));
            }
        }
    }

    // Sort by depth for nice output
    descendants.sort_by_key(|d| d.depth);

    Ok(descendants)
}

/// Get a person with their immediate family (parents, spouse, children, siblings).
pub fn get_person_with_family(storage: &GotStorage, person_id: &str) -> Result<PersonFamily> {
    let node_id = storage
        .lookup_by_id(person_id)?
        .ok_or_else(|| GotError::PersonNotFound(person_id.to_string()))?;

    let person = storage.get_person(node_id)?;

    // Get parents (incoming PARENT_OF)
    let parent_ids = storage.get_incoming_neighbors(node_id, RelationType::ParentOf)?;
    let mut parents = Vec::new();
    for parent_id in parent_ids {
        parents.push(storage.get_person(parent_id)?);
    }

    // Get children (outgoing PARENT_OF)
    let child_ids = storage.get_outgoing_neighbors(node_id, RelationType::ParentOf)?;
    let mut children = Vec::new();
    for child_id in child_ids {
        children.push(storage.get_person(child_id)?);
    }

    // Get spouse (outgoing SPOUSE_OF)
    let spouse_ids = storage.get_outgoing_neighbors(node_id, RelationType::SpouseOf)?;
    let mut spouses = Vec::new();
    for spouse_id in spouse_ids {
        spouses.push(storage.get_person(spouse_id)?);
    }

    // Get siblings (outgoing SIBLING_OF)
    let sibling_ids = storage.get_outgoing_neighbors(node_id, RelationType::SiblingOf)?;
    let mut siblings = Vec::new();
    for sibling_id in sibling_ids {
        siblings.push(storage.get_person(sibling_id)?);
    }

    Ok(PersonFamily {
        person,
        parents,
        spouses,
        children,
        siblings,
    })
}

/// A person with their immediate family connections.
#[derive(Debug)]
pub struct PersonFamily {
    pub person: Person,
    pub parents: Vec<Person>,
    pub spouses: Vec<Person>,
    pub children: Vec<Person>,
    pub siblings: Vec<Person>,
}
