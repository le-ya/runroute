use crate::graph::types::NodeIndex;
use hashbrown::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct SearchLabel {
    pub anchor: NodeIndex,
    pub nodes: Vec<NodeIndex>,
    pub anchor_path: Vec<NodeIndex>,
    pub distance_m: f64,
    pub dplus_m: f64,
    pub dminus_m: f64,
    pub repeated_m: f64,
    pub turn_penalty: f64,
    pub relief_cycles: usize,
    pub used_edges: HashSet<(NodeIndex, NodeIndex)>,
    pub surface_distances_m: HashMap<String, f64>,
    pub way_distances_m: HashMap<String, f64>,
}

impl SearchLabel {
    pub fn new(origin: NodeIndex) -> Self {
        SearchLabel {
            anchor: origin,
            nodes: vec![origin],
            anchor_path: vec![origin],
            distance_m: 0.0,
            dplus_m: 0.0,
            dminus_m: 0.0,
            repeated_m: 0.0,
            turn_penalty: 0.0,
            relief_cycles: 0,
            used_edges: HashSet::new(),
            surface_distances_m: HashMap::new(),
            way_distances_m: HashMap::new(),
        }
    }
}

pub fn label_dominates(a: &SearchLabel, b: &SearchLabel) -> bool {
    if a.distance_m > b.distance_m
        || a.dplus_m < b.dplus_m
        || a.repeated_m > b.repeated_m
        || a.turn_penalty > b.turn_penalty
    {
        return false;
    }

    a.distance_m <= b.distance_m
        && a.dplus_m >= b.dplus_m
        && a.repeated_m <= b.repeated_m
        && a.turn_penalty <= b.turn_penalty
        && (a.distance_m < b.distance_m
            || a.dplus_m > b.dplus_m
            || a.repeated_m < b.repeated_m
            || a.turn_penalty < b.turn_penalty)
}
