use super::types::{Edge, Node};
use anyhow::Result;
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub struct BinaryGraph {
    pub nodes: Vec<Node>,
    pub osm_to_idx: HashMap<u64, u32>,
    pub edges: Vec<Edge>,
}

pub fn save_binary<P: AsRef<Path>>(
    path: P,
    nodes: &[Node],
    osm_to_idx: &HashMap<u64, u32>,
    edges: &[Edge],
) -> Result<()> {
    let file = File::create(path)?;
    let writer = BufWriter::with_capacity(1024 * 1024, file);
    let bg = BinaryGraph {
        nodes: nodes.to_vec(),
        osm_to_idx: osm_to_idx.clone(),
        edges: edges.to_vec(),
    };
    bincode::serialize_into(writer, &bg)?;
    Ok(())
}

pub fn load_binary<P: AsRef<Path>>(path: P) -> Result<BinaryGraph> {
    let file = File::open(path)?;
    let reader = BufReader::with_capacity(1024 * 1024, file);
    let bg: BinaryGraph = bincode::deserialize_from(reader)?;
    Ok(bg)
}
