//! HelixDB storage layer for the family tree graph.

use crate::error::{GotError, Result};
use crate::loader::{FamilyTree, RelationshipDef};
use crate::types::{GraphStats, House, Person, RelationType};
use bumpalo::Bump;
use helix_db::{
    helix_engine::{
        storage_core::{HelixGraphStorage, storage_methods::StorageMethods},
        traversal_core::config::{Config, GraphConfig},
        types::SecondaryIndex,
    },
    protocol::value::Value,
    utils::{items::Edge, label_hash::hash_label, properties::ImmutablePropertiesMap},
};
use helix_graph_ops as graph_ops;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

const NODE_LABEL: &str = "PERSON";

/// HelixDB storage wrapper for the Game of Thrones family tree.
pub struct GotStorage {
    storage: HelixGraphStorage,
    db_path: PathBuf,
    /// Maps person ID (string) to node ID (u128).
    id_to_node: HashMap<String, u128>,
}

impl GotStorage {
    /// Create or open a storage instance at the given path.
    pub fn new(db_path: &Path) -> Result<Self> {
        let graph_path = db_path.join("graph.db");
        std::fs::create_dir_all(&graph_path).map_err(|e| {
            GotError::DatabaseError(format!("Failed to create database directory: {e}"))
        })?;

        let config = Config {
            graph_config: Some(GraphConfig {
                secondary_indices: Some(vec![
                    SecondaryIndex::Index("id".to_string()),
                    SecondaryIndex::Index("house".to_string()),
                ]),
            }),
            db_max_size_gb: Some(1),
            ..Default::default()
        };

        let version_info =
            helix_db::helix_engine::storage_core::version_info::VersionInfo::default();

        let storage =
            HelixGraphStorage::new(&graph_path.to_string_lossy(), config, version_info)
                .map_err(|e| GotError::DatabaseError(format!("Failed to create storage: {e:?}")))?;

        Ok(Self {
            storage,
            db_path: db_path.to_path_buf(),
            id_to_node: HashMap::new(),
        })
    }

    /// Check if the database exists and has data.
    pub fn exists(db_path: &Path) -> bool {
        db_path.join("graph.db").exists()
    }

    /// Clear all data from the database.
    pub fn clear(&self) -> Result<()> {
        let mut wtxn =
            self.storage.graph_env.write_txn().map_err(|e| {
                GotError::DatabaseError(format!("Failed to start transaction: {e}"))
            })?;

        self.storage
            .nodes_db
            .clear(&mut wtxn)
            .map_err(|e| GotError::DatabaseError(format!("Failed to clear nodes: {e}")))?;

        self.storage
            .edges_db
            .clear(&mut wtxn)
            .map_err(|e| GotError::DatabaseError(format!("Failed to clear edges: {e}")))?;

        self.storage
            .out_edges_db
            .clear(&mut wtxn)
            .map_err(|e| GotError::DatabaseError(format!("Failed to clear out_edges: {e}")))?;

        self.storage
            .in_edges_db
            .clear(&mut wtxn)
            .map_err(|e| GotError::DatabaseError(format!("Failed to clear in_edges: {e}")))?;

        wtxn.commit()
            .map_err(|e| GotError::DatabaseError(format!("Failed to commit clear: {e}")))?;

        Ok(())
    }

    /// Ingest a family tree into the database.
    pub fn ingest(&mut self, tree: &FamilyTree) -> Result<IngestStats> {
        let mut stats = IngestStats::default();

        // First pass: insert all people as nodes
        for person in &tree.people {
            let node_id = self.insert_person(person)?;
            self.id_to_node.insert(person.id.clone(), node_id);
            stats.nodes_inserted += 1;
        }

        // Second pass: create all relationship edges
        for rel in &tree.relationships {
            match rel {
                RelationshipDef::ParentOf { from, to } => {
                    let from_node = self
                        .id_to_node
                        .get(from)
                        .copied()
                        .ok_or_else(|| GotError::PersonNotFound(from.clone()))?;

                    for child_id in to {
                        let to_node = self
                            .id_to_node
                            .get(child_id)
                            .copied()
                            .ok_or_else(|| GotError::PersonNotFound(child_id.clone()))?;
                        self.create_edge(from_node, to_node, RelationType::ParentOf)?;
                        stats.edges_inserted += 1;
                    }
                }
                RelationshipDef::SpouseOf { between } => {
                    if between.len() >= 2 {
                        let a = self
                            .id_to_node
                            .get(&between[0])
                            .copied()
                            .ok_or_else(|| GotError::PersonNotFound(between[0].clone()))?;
                        let b = self
                            .id_to_node
                            .get(&between[1])
                            .copied()
                            .ok_or_else(|| GotError::PersonNotFound(between[1].clone()))?;
                        // Bidirectional: create edges in both directions
                        self.create_edge(a, b, RelationType::SpouseOf)?;
                        self.create_edge(b, a, RelationType::SpouseOf)?;
                        stats.edges_inserted += 2;
                    }
                }
                RelationshipDef::SiblingOf { between } => {
                    // Create edges between all pairs (bidirectional)
                    for i in 0..between.len() {
                        for j in (i + 1)..between.len() {
                            let a = self
                                .id_to_node
                                .get(&between[i])
                                .copied()
                                .ok_or_else(|| GotError::PersonNotFound(between[i].clone()))?;
                            let b = self
                                .id_to_node
                                .get(&between[j])
                                .copied()
                                .ok_or_else(|| GotError::PersonNotFound(between[j].clone()))?;
                            self.create_edge(a, b, RelationType::SiblingOf)?;
                            self.create_edge(b, a, RelationType::SiblingOf)?;
                            stats.edges_inserted += 2;
                        }
                    }
                }
            }
        }

        Ok(stats)
    }

    /// Insert a person as a node in the graph.
    fn insert_person(&self, person: &Person) -> Result<u128> {
        let arena = Bump::new();
        let mut wtxn =
            self.storage.graph_env.write_txn().map_err(|e| {
                GotError::DatabaseError(format!("Failed to start transaction: {e}"))
            })?;

        let node_id = Uuid::new_v4().as_u128();
        let label: &str = arena.alloc_str(NODE_LABEL);

        let titles_json = serde_json::to_string(&person.titles).unwrap_or_default();
        let alias_str = person.alias.clone().unwrap_or_default();
        let is_alive_str = person.is_alive.to_string();

        let props: Vec<(&str, Value)> = vec![
            (arena.alloc_str("id"), Value::String(person.id.clone())),
            (arena.alloc_str("name"), Value::String(person.name.clone())),
            (
                arena.alloc_str("house"),
                Value::String(person.house.to_string()),
            ),
            (arena.alloc_str("titles"), Value::String(titles_json)),
            (arena.alloc_str("alias"), Value::String(alias_str)),
            (arena.alloc_str("is_alive"), Value::String(is_alive_str)),
        ];

        let properties = ImmutablePropertiesMap::new(props.len(), props.into_iter(), &arena);

        let node = helix_db::utils::items::Node {
            id: node_id,
            label,
            version: 1,
            properties: Some(properties),
        };

        graph_ops::put_node(&self.storage, &mut wtxn, &node)
            .map_err(|e| GotError::DatabaseError(format!("Failed to store node: {e}")))?;

        graph_ops::update_secondary_indices(&self.storage, &mut wtxn, &node).map_err(|e| {
            GotError::DatabaseError(format!("Failed to update secondary index: {e}"))
        })?;

        wtxn.commit()
            .map_err(|e| GotError::DatabaseError(format!("Failed to commit node: {e}")))?;

        Ok(node_id)
    }

    /// Create an edge between two nodes.
    fn create_edge(
        &self,
        from_node_id: u128,
        to_node_id: u128,
        relation_type: RelationType,
    ) -> Result<()> {
        let arena = Bump::new();
        let mut wtxn =
            self.storage.graph_env.write_txn().map_err(|e| {
                GotError::DatabaseError(format!("Failed to start transaction: {e}"))
            })?;

        let edge_id = Uuid::new_v4().as_u128();
        let edge_label = arena.alloc_str(relation_type.as_edge_label());

        let edge = Edge {
            id: edge_id,
            label: edge_label,
            version: 1,
            from_node: from_node_id,
            to_node: to_node_id,
            properties: None,
        };

        graph_ops::put_edge(&self.storage, &mut wtxn, &edge)
            .map_err(|e| GotError::DatabaseError(format!("Failed to store edge: {e}")))?;

        wtxn.commit()
            .map_err(|e| GotError::DatabaseError(format!("Failed to commit edge: {e}")))?;

        Ok(())
    }

    /// Look up a node ID by person ID using the secondary index.
    pub fn lookup_by_id(&self, person_id: &str) -> Result<Option<u128>> {
        let rtxn = self.storage.graph_env.read_txn().map_err(|e| {
            GotError::DatabaseError(format!("Failed to start read transaction: {e}"))
        })?;

        let key = Value::String(person_id.to_string());
        if let Some(node_id) =
            graph_ops::lookup_secondary_index(&self.storage, &rtxn, "id", &key)
                .map_err(|e| GotError::DatabaseError(format!("Failed to lookup: {e}")))?
        {
            return Ok(Some(node_id));
        }

        Ok(None)
    }

    /// Get a person from a node ID.
    pub fn get_person(&self, node_id: u128) -> Result<Person> {
        let arena = Bump::new();
        let rtxn = self.storage.graph_env.read_txn().map_err(|e| {
            GotError::DatabaseError(format!("Failed to start read transaction: {e}"))
        })?;

        let node = self
            .storage
            .get_node(&rtxn, &node_id, &arena)
            .map_err(|e| GotError::DatabaseError(format!("Failed to get node: {e:?}")))?;

        self.node_to_person(&node)
    }

    /// Convert a HelixDB node to a Person struct.
    fn node_to_person(&self, node: &helix_db::utils::items::Node<'_>) -> Result<Person> {
        let get_str = |name: &str| -> String {
            node.get_property(name)
                .and_then(|v| match v {
                    Value::String(s) => Some(s.clone()),
                    _ => None,
                })
                .unwrap_or_default()
        };

        let get_bool = |name: &str| -> bool {
            node.get_property(name)
                .and_then(|v| match v {
                    Value::String(s) => s.parse().ok(),
                    _ => None,
                })
                .unwrap_or(false)
        };

        let id = get_str("id");
        let name = get_str("name");
        let house_str = get_str("house");
        let titles_json = get_str("titles");
        let alias_str = get_str("alias");
        let is_alive = get_bool("is_alive");

        let house: House = house_str
            .parse()
            .map_err(|e| GotError::DatabaseError(format!("Invalid house: {e}")))?;

        let titles: Vec<String> = serde_json::from_str(&titles_json).unwrap_or_default();
        let alias = if alias_str.is_empty() {
            None
        } else {
            Some(alias_str)
        };

        Ok(Person {
            id,
            name,
            house,
            titles,
            alias,
            is_alive,
        })
    }

    /// Get all nodes connected by incoming edges of a specific type.
    /// For PARENT_OF: returns parents of the given node.
    pub fn get_incoming_neighbors(
        &self,
        node_id: u128,
        relation_type: RelationType,
    ) -> Result<Vec<u128>> {
        let rtxn = self.storage.graph_env.read_txn().map_err(|e| {
            GotError::DatabaseError(format!("Failed to start read transaction: {e}"))
        })?;

        let label_hash = hash_label(relation_type.as_edge_label(), None);
        graph_ops::incoming_neighbors(&self.storage, &rtxn, node_id, &label_hash)
            .map_err(|e| GotError::DatabaseError(format!("Failed to read incoming edges: {e}")))
    }

    /// Get all nodes connected by outgoing edges of a specific type.
    /// For PARENT_OF: returns children of the given node.
    pub fn get_outgoing_neighbors(
        &self,
        node_id: u128,
        relation_type: RelationType,
    ) -> Result<Vec<u128>> {
        let rtxn = self.storage.graph_env.read_txn().map_err(|e| {
            GotError::DatabaseError(format!("Failed to start read transaction: {e}"))
        })?;

        let label_hash = hash_label(relation_type.as_edge_label(), None);
        graph_ops::outgoing_neighbors(&self.storage, &rtxn, node_id, &label_hash)
            .map_err(|e| GotError::DatabaseError(format!("Failed to read outgoing edges: {e}")))
    }

    /// Get statistics about the graph.
    pub fn get_stats(&self) -> Result<GraphStats> {
        let rtxn = self.storage.graph_env.read_txn().map_err(|e| {
            GotError::DatabaseError(format!("Failed to start read transaction: {e}"))
        })?;

        let arena = Bump::new();
        let mut node_count = 0;
        let mut edge_count = 0;
        let mut house_counts: HashMap<String, usize> = HashMap::new();

        // Count nodes and collect house statistics
        let iter = self
            .storage
            .nodes_db
            .iter(&rtxn)
            .map_err(|e| GotError::DatabaseError(format!("Failed to iterate nodes: {e}")))?;

        for result in iter {
            let (node_id, value) =
                result.map_err(|e| GotError::DatabaseError(format!("Failed to read node: {e}")))?;

            if let Ok(node) =
                helix_db::utils::items::Node::from_bincode_bytes(node_id, value, &arena)
            {
                node_count += 1;
                if let Some(Value::String(house)) = node.get_property("house") {
                    *house_counts.entry(house.clone()).or_insert(0) += 1;
                }
            }
        }

        // Count edges
        let edge_iter = self
            .storage
            .edges_db
            .iter(&rtxn)
            .map_err(|e| GotError::DatabaseError(format!("Failed to iterate edges: {e}")))?;

        for result in edge_iter {
            if result.is_ok() {
                edge_count += 1;
            }
        }

        Ok(GraphStats {
            node_count,
            edge_count,
            house_counts,
        })
    }

    /// Get all people belonging to a specific house.
    pub fn get_house_members(&self, house: House) -> Result<Vec<Person>> {
        let rtxn = self.storage.graph_env.read_txn().map_err(|e| {
            GotError::DatabaseError(format!("Failed to start read transaction: {e}"))
        })?;

        let arena = Bump::new();
        let house_str = house.to_string();
        let mut members = Vec::new();

        let iter = self
            .storage
            .nodes_db
            .iter(&rtxn)
            .map_err(|e| GotError::DatabaseError(format!("Failed to iterate nodes: {e}")))?;

        for result in iter {
            let (node_id, value) =
                result.map_err(|e| GotError::DatabaseError(format!("Failed to read node: {e}")))?;

            if let Ok(node) =
                helix_db::utils::items::Node::from_bincode_bytes(node_id, value, &arena)
                && let Some(Value::String(node_house)) = node.get_property("house")
                && node_house == &house_str
                && let Ok(person) = self.node_to_person(&node)
            {
                members.push(person);
            }
        }

        Ok(members)
    }

    /// Get the database path.
    #[must_use]
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }
}

/// Statistics from an ingest operation.
#[derive(Debug, Default)]
pub struct IngestStats {
    pub nodes_inserted: usize,
    pub edges_inserted: usize,
}

#[cfg(test)]
mod tests {
    use super::GotStorage;
    use crate::error::Result;
    use crate::loader::{FamilyTree, RelationshipDef};
    use crate::types::{House, Person, RelationType};
    use tempfile::TempDir;

    fn build_tree() -> FamilyTree {
        FamilyTree {
            houses: Vec::new(),
            people: vec![
                Person {
                    id: "ned-stark".to_string(),
                    name: "Eddard Stark".to_string(),
                    house: House::Stark,
                    titles: vec!["Lord of Winterfell".to_string()],
                    alias: None,
                    is_alive: true,
                },
                Person {
                    id: "catelyn-stark".to_string(),
                    name: "Catelyn Stark".to_string(),
                    house: House::Tully,
                    titles: Vec::new(),
                    alias: None,
                    is_alive: true,
                },
                Person {
                    id: "robb-stark".to_string(),
                    name: "Robb Stark".to_string(),
                    house: House::Stark,
                    titles: vec!["King in the North".to_string()],
                    alias: None,
                    is_alive: true,
                },
            ],
            relationships: vec![
                RelationshipDef::ParentOf {
                    from: "ned-stark".to_string(),
                    to: vec!["robb-stark".to_string()],
                },
                RelationshipDef::SpouseOf {
                    between: vec!["ned-stark".to_string(), "catelyn-stark".to_string()],
                },
            ],
        }
    }

    fn open_storage() -> Result<(TempDir, GotStorage)> {
        let temp = TempDir::new()?;
        let storage = GotStorage::new(temp.path())?;
        Ok((temp, storage))
    }

    #[test]
    fn test_ingest_and_relationship_queries() -> Result<()> {
        let (_temp, mut storage) = open_storage()?;
        let tree = build_tree();
        let stats = storage.ingest(&tree)?;

        assert_eq!(stats.nodes_inserted, 3);
        assert_eq!(stats.edges_inserted, 3);

        let ned_node = storage.lookup_by_id("ned-stark")?.expect("ned node");
        let robb_node = storage.lookup_by_id("robb-stark")?.expect("robb node");
        let catelyn_node = storage
            .lookup_by_id("catelyn-stark")?
            .expect("catelyn node");

        let ned = storage.get_person(ned_node)?;
        assert_eq!(ned.name, "Eddard Stark");
        assert_eq!(ned.house, House::Stark);

        let outgoing_parent = storage.get_outgoing_neighbors(ned_node, RelationType::ParentOf)?;
        assert_eq!(outgoing_parent, vec![robb_node]);

        let incoming_parent = storage.get_incoming_neighbors(robb_node, RelationType::ParentOf)?;
        assert_eq!(incoming_parent, vec![ned_node]);

        let spouse_out = storage.get_outgoing_neighbors(ned_node, RelationType::SpouseOf)?;
        assert_eq!(spouse_out, vec![catelyn_node]);

        let spouse_in = storage.get_incoming_neighbors(ned_node, RelationType::SpouseOf)?;
        assert_eq!(spouse_in, vec![catelyn_node]);

        Ok(())
    }

    #[test]
    fn test_stats_house_members_and_clear() -> Result<()> {
        let (_temp, mut storage) = open_storage()?;
        let tree = build_tree();
        storage.ingest(&tree)?;

        let stats = storage.get_stats()?;
        assert_eq!(stats.node_count, 3);
        assert_eq!(stats.edge_count, 3);
        assert_eq!(stats.house_counts.get("Stark").copied().unwrap_or(0), 2);
        assert_eq!(stats.house_counts.get("Tully").copied().unwrap_or(0), 1);

        let stark_members = storage.get_house_members(House::Stark)?;
        let mut stark_ids: Vec<String> = stark_members.into_iter().map(|p| p.id).collect();
        stark_ids.sort();
        assert_eq!(
            stark_ids,
            vec!["ned-stark".to_string(), "robb-stark".to_string()]
        );

        storage.clear()?;
        let cleared_stats = storage.get_stats()?;
        assert_eq!(cleared_stats.node_count, 0);
        assert_eq!(cleared_stats.edge_count, 0);
        assert!(cleared_stats.house_counts.is_empty());

        Ok(())
    }
}
