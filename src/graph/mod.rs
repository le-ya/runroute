pub mod binary;
pub mod graphml;
pub mod types;

use anyhow::Result;
use hashbrown::HashMap;
use std::path::Path;
pub use types::{Edge, EdgeIndex, Node, NodeIndex, SurfaceClass, WayClass};

pub struct Graph {
    pub nodes: Vec<Node>,
    pub osm_to_idx: HashMap<u64, NodeIndex>,
    pub edges: Vec<Edge>,
    /// Outgoing edge indices for each node
    pub out_edges: Vec<Vec<EdgeIndex>>,
    /// Incoming edge indices for each node
    pub in_edges: Vec<Vec<EdgeIndex>>,
}

impl Graph {
    pub fn new(nodes: Vec<Node>, osm_to_idx: HashMap<u64, NodeIndex>, edges: Vec<Edge>) -> Self {
        let n = nodes.len();
        let mut out_edges = vec![Vec::new(); n];
        let mut in_edges = vec![Vec::new(); n];

        for (edge_idx, edge) in edges.iter().enumerate() {
            let e = edge_idx as EdgeIndex;
            out_edges[edge.source as usize].push(e);
            in_edges[edge.target as usize].push(e);
        }

        Graph {
            nodes,
            osm_to_idx,
            edges,
            out_edges,
            in_edges,
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn get_node_by_osm_id(&self, osm_id: u64) -> Option<NodeIndex> {
        self.osm_to_idx.get(&osm_id).copied()
    }

    /// Find nearest node to metric (x, y) coordinates using fast Euclidean distance.
    pub fn nearest_node(&self, x: f64, y: f64) -> NodeIndex {
        let mut best_idx = 0;
        let mut best_dist_sq = f64::INFINITY;

        for (i, node) in self.nodes.iter().enumerate() {
            let dx = node.x - x;
            let dy = node.y - y;
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < best_dist_sq {
                best_dist_sq = dist_sq;
                best_idx = i as NodeIndex;
            }
        }
        best_idx
    }

    /// Junction degree (unique neighbors connected in either direction).
    pub fn junction_degree(&self, node: NodeIndex) -> usize {
        let mut neighbors = hashbrown::HashSet::new();
        for &e in &self.out_edges[node as usize] {
            neighbors.insert(self.edges[e as usize].target);
        }
        for &e in &self.in_edges[node as usize] {
            neighbors.insert(self.edges[e as usize].source);
        }
        neighbors.remove(&node);
        neighbors.len()
    }

    /// Feature score for anchor distribution.
    pub fn feature_score(&self, node: NodeIndex) -> f64 {
        let mut has_trail_or_path = false;
        let mut has_road = false;

        for &e in &self.out_edges[node as usize] {
            let edge = &self.edges[e as usize];
            if matches!(edge.surface, SurfaceClass::Trail | SurfaceClass::Path)
                || matches!(edge.way, WayClass::Path | WayClass::Pedestrian)
            {
                has_trail_or_path = true;
            }
            if matches!(edge.way, WayClass::MainRoad | WayClass::LocalRoad | WayClass::QuietRoad) {
                has_road = true;
            }
        }

        let degree = self.junction_degree(node);
        let mut score = degree as f64;
        if has_trail_or_path && has_road {
            score += 3.0; // Trail entrance from road
        } else if has_trail_or_path {
            score += 1.5;
        }
        score
    }

    /// Load from binary cache if available, or parse GraphML and write binary cache.
    pub fn load_or_convert(data_dir: &Path) -> Result<Self> {
        let bin_path = data_dir.join("graph.bin");
        let graphml_path = data_dir.join("graph.graphml");

        if bin_path.exists() {
            let t0 = std::time::Instant::now();
            let bg = binary::load_binary(&bin_path)?;
            eprintln!("[{:.2}s] Graphe binaire chargé: {} nœuds, {} arêtes ({:?})", t0.elapsed().as_secs_f64(), bg.nodes.len(), bg.edges.len(), bin_path);
            return Ok(Graph::new(bg.nodes, bg.osm_to_idx, bg.edges));
        }

        if graphml_path.exists() {
            eprintln!("Parsing du fichier GraphML XML ({:?})...", graphml_path);
            let t0 = std::time::Instant::now();
            let parsed = graphml::parse_graphml(&graphml_path)?;
            eprintln!("[{:.2}s] GraphML XML parsé ({} nœuds, {} arêtes)", t0.elapsed().as_secs_f64(), parsed.nodes.len(), parsed.edges.len());

            eprintln!("Sauvegarde du cache binaire rapide ({:?})...", bin_path);
            let _ = binary::save_binary(&bin_path, &parsed.nodes, &parsed.osm_to_idx, &parsed.edges);
            eprintln!("Cache binaire prêt ! Les prochains lancements seront quasi-instantanés (<50ms).");

            return Ok(Graph::new(parsed.nodes, parsed.osm_to_idx, parsed.edges));
        }

        anyhow::bail!("Aucun fichier graph.bin ou graph.graphml trouvé dans {:?}", data_dir);
    }
}
