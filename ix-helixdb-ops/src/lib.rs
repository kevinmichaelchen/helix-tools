use heed3::{RoTxn, RwTxn};
use helix_db::helix_engine::storage_core::HelixGraphStorage;
use helix_db::helix_engine::types::GraphError;
use helix_db::protocol::value::Value;
use helix_db::utils::items::{Edge, Node};
use helix_db::utils::label_hash::hash_label;

pub type Result<T> = std::result::Result<T, GraphError>;

pub fn put_node(storage: &HelixGraphStorage, wtxn: &mut RwTxn<'_>, node: &Node<'_>) -> Result<()> {
    let node_bytes = node.to_bincode_bytes()?;
    storage
        .nodes_db
        .put(wtxn, HelixGraphStorage::node_key(&node.id), &node_bytes)?;
    Ok(())
}

pub fn update_secondary_indices(
    storage: &HelixGraphStorage,
    wtxn: &mut RwTxn<'_>,
    node: &Node<'_>,
) -> Result<()> {
    for (index_name, db) in &storage.secondary_indices {
        if let Some(value) = node.get_property(index_name) {
            let serialized = bincode::serialize(value)?;
            db.0.put(wtxn, &serialized, &node.id)?;
        }
    }
    Ok(())
}

pub fn put_edge(storage: &HelixGraphStorage, wtxn: &mut RwTxn<'_>, edge: &Edge<'_>) -> Result<()> {
    let edge_bytes = edge.to_bincode_bytes()?;
    storage
        .edges_db
        .put(wtxn, HelixGraphStorage::edge_key(&edge.id), &edge_bytes)?;

    let label_hash = hash_label(edge.label, None);
    let out_key = HelixGraphStorage::out_edge_key(&edge.from_node, &label_hash);
    let out_val = HelixGraphStorage::pack_edge_data(&edge.id, &edge.to_node);
    storage.out_edges_db.put(wtxn, &out_key, &out_val)?;

    let in_key = HelixGraphStorage::in_edge_key(&edge.to_node, &label_hash);
    let in_val = HelixGraphStorage::pack_edge_data(&edge.id, &edge.from_node);
    storage.in_edges_db.put(wtxn, &in_key, &in_val)?;

    Ok(())
}

pub fn lookup_secondary_index(
    storage: &HelixGraphStorage,
    rtxn: &RoTxn<'_>,
    index_name: &str,
    key: &Value,
) -> Result<Option<u128>> {
    let Some(db) = storage.secondary_indices.get(index_name) else {
        return Ok(None);
    };

    let serialized = bincode::serialize(key)?;
    Ok(db.0.get(rtxn, &serialized)?)
}

pub fn outgoing_neighbors(
    storage: &HelixGraphStorage,
    rtxn: &RoTxn<'_>,
    node_id: u128,
    label_hash: &[u8; 4],
) -> Result<Vec<u128>> {
    let out_key = HelixGraphStorage::out_edge_key(&node_id, label_hash);
    let mut neighbors = Vec::new();

    let iter = storage.out_edges_db.prefix_iter(rtxn, &out_key)?;
    for result in iter {
        let (_, value) = result?;
        let (_, to_node_id) = HelixGraphStorage::unpack_adj_edge_data(value)?;
        neighbors.push(to_node_id);
    }

    Ok(neighbors)
}

pub fn incoming_neighbors(
    storage: &HelixGraphStorage,
    rtxn: &RoTxn<'_>,
    node_id: u128,
    label_hash: &[u8; 4],
) -> Result<Vec<u128>> {
    let in_key = HelixGraphStorage::in_edge_key(&node_id, label_hash);
    let mut neighbors = Vec::new();

    let iter = storage.in_edges_db.prefix_iter(rtxn, &in_key)?;
    for result in iter {
        let (_, value) = result?;
        let (_, from_node_id) = HelixGraphStorage::unpack_adj_edge_data(value)?;
        neighbors.push(from_node_id);
    }

    Ok(neighbors)
}
