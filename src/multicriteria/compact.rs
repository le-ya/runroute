//! Precomputed compact anchor graph.

use crate::dijkstra::shortest_path;
use crate::geo::euclidean_dist;
use crate::graph::types::NodeIndex;
use crate::graph::Graph;
use crate::profiles::{routing_profile_for_target, Profile};
use hashbrown::HashMap;
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub struct AnchorSegment {
    pub source: NodeIndex,
    pub target: NodeIndex,
    pub nodes: Vec<NodeIndex>,
    pub distance_m: f64,
    pub dplus_m: f64,
    pub dminus_m: f64,
    pub edge_keys: Vec<(NodeIndex, NodeIndex)>,
    pub surface_distances: HashMap<String, f64>,
    pub way_distances: HashMap<String, f64>,
}

#[derive(Debug, Clone)]
pub struct CompactGraph {
    pub anchors: Vec<NodeIndex>,
    pub anchor_to_idx: HashMap<NodeIndex, usize>,
    /// Outgoing segments for each anchor index
    pub out_segments: Vec<Vec<AnchorSegment>>,
}

impl CompactGraph {
    pub fn build(
        graph: &Graph,
        anchors: &[NodeIndex],
        profile: &Profile,
        target_m: f64,
        dplus_target_m: Option<f64>,
        neighbor_count: usize,
        mandatory: &[NodeIndex],
    ) -> Self {
        let route_profile = routing_profile_for_target(profile, target_m, dplus_target_m);
        let mut pairs: hashbrown::HashSet<(NodeIndex, NodeIndex)> = hashbrown::HashSet::new();

        // 1. Geographic nearest + relief connections
        for &a in anchors {
            let ax = graph.nodes[a as usize].x;
            let ay = graph.nodes[a as usize].y;
            let a_elev = graph.nodes[a as usize].elevation as f64;

            let mut others: Vec<NodeIndex> = anchors.iter().copied().filter(|&o| o != a).collect();
            others.sort_by(|&o1, &o2| {
                let d1 = euclidean_dist(ax, ay, graph.nodes[o1 as usize].x, graph.nodes[o1 as usize].y);
                let d2 = euclidean_dist(ax, ay, graph.nodes[o2 as usize].x, graph.nodes[o2 as usize].y);
                d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
            });

            for &o in others.iter().take(neighbor_count) {
                pairs.insert((a, o));
                pairs.insert((o, a));
            }

            // Relief connections
            let mut relief_candidates: Vec<NodeIndex> = others
                .into_iter()
                .filter(|&o| {
                    let d = euclidean_dist(ax, ay, graph.nodes[o as usize].x, graph.nodes[o as usize].y);
                    d <= target_m * 0.45
                })
                .collect();

            relief_candidates.sort_by(|&o1, &o2| {
                let d1 = euclidean_dist(ax, ay, graph.nodes[o1 as usize].x, graph.nodes[o1 as usize].y);
                let d2 = euclidean_dist(ax, ay, graph.nodes[o2 as usize].x, graph.nodes[o2 as usize].y);
                let drop1 = (graph.nodes[o1 as usize].elevation as f64 - a_elev).abs() / d1.max(1.0);
                let drop2 = (graph.nodes[o2 as usize].elevation as f64 - a_elev).abs() / d2.max(1.0);
                drop2.partial_cmp(&drop1).unwrap_or(std::cmp::Ordering::Equal)
            });

            for &o in relief_candidates.iter().take(2) {
                pairs.insert((a, o));
                pairs.insert((o, a));
            }
        }

        // 2. Direct relief access for mandatory points (departures & home)
        for &m in mandatory {
            let mx = graph.nodes[m as usize].x;
            let my = graph.nodes[m as usize].y;

            let mut access: Vec<NodeIndex> = anchors.iter().copied().filter(|&o| o != m).collect();
            access.sort_by(|&o1, &o2| {
                let e1 = graph.nodes[o1 as usize].elevation;
                let e2 = graph.nodes[o2 as usize].elevation;
                e2.partial_cmp(&e1).unwrap_or(std::cmp::Ordering::Equal).then_with(|| {
                    let d1 = euclidean_dist(mx, my, graph.nodes[o1 as usize].x, graph.nodes[o1 as usize].y);
                    let d2 = euclidean_dist(mx, my, graph.nodes[o2 as usize].x, graph.nodes[o2 as usize].y);
                    d1.partial_cmp(&d2).unwrap_or(std::cmp::Ordering::Equal)
                })
            });

            for &a in access.iter().take(6.max(neighbor_count)) {
                pairs.insert((m, a));
                pairs.insert((a, m));
            }
        }

        let sorted_pairs: Vec<(NodeIndex, NodeIndex)> = pairs.into_iter().collect();

        // 3. Parallel Dijkstra evaluation across all CPU threads
        let segments_results: Vec<Option<AnchorSegment>> = sorted_pairs
            .par_iter()
            .map(|&(start, end)| {
                if let Some((path_nodes, _)) = shortest_path(graph, start, end, &route_profile) {
                    if path_nodes.len() >= 2 {
                        let mut dist = 0.0;
                        let mut dp = 0.0;
                        let mut dm = 0.0;
                        let mut edge_keys = Vec::with_capacity(path_nodes.len() - 1);
                        let mut surf_dist: HashMap<String, f64> = HashMap::new();
                        let mut way_dist: HashMap<String, f64> = HashMap::new();

                        for (&u, &v) in path_nodes.iter().zip(path_nodes.iter().skip(1)) {
                            let key = if u < v { (u, v) } else { (v, u) };
                            edge_keys.push(key);

                            // Find best edge between u and v
                            if let Some(&e_idx) = graph.out_edges[u as usize]
                                .iter()
                                .find(|&&e| graph.edges[e as usize].target == v)
                            {
                                let e = &graph.edges[e_idx as usize];
                                let l = e.length as f64;
                                dist += l;
                                dp += e.d_plus as f64;
                                dm += e.d_minus as f64;

                                *surf_dist.entry(e.surface.as_str().to_string()).or_insert(0.0) += l;
                                *way_dist.entry(e.way.as_str().to_string()).or_insert(0.0) += l;
                            }
                        }

                        if dist > 0.0 && dist <= target_m * 0.75 {
                            return Some(AnchorSegment {
                                source: start,
                                target: end,
                                nodes: path_nodes,
                                distance_m: dist,
                                dplus_m: dp,
                                dminus_m: dm,
                                edge_keys,
                                surface_distances: surf_dist,
                                way_distances: way_dist,
                            });
                        }
                    }
                }
                None
            })
            .collect();

        let mut anchor_to_idx = HashMap::new();
        for (i, &a) in anchors.iter().enumerate() {
            anchor_to_idx.insert(a, i);
        }

        let mut out_segments = vec![Vec::new(); anchors.len()];
        for seg_opt in segments_results {
            if let Some(seg) = seg_opt {
                if let Some(&src_idx) = anchor_to_idx.get(&seg.source) {
                    out_segments[src_idx].push(seg);
                }
            }
        }

        CompactGraph {
            anchors: anchors.to_vec(),
            anchor_to_idx,
            out_segments,
        }
    }

    pub fn number_of_edges(&self) -> usize {
        self.out_segments.iter().map(|s| s.len()).sum()
    }
}
