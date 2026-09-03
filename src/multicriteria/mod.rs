pub mod compact;
pub mod label;
pub mod priority;

use crate::config::Config;
use crate::dijkstra::remaining_distances_to_destinations;
use crate::graph::types::NodeIndex;
use crate::graph::Graph;
use crate::profiles::Profile;
pub use compact::{AnchorSegment, CompactGraph};
use hashbrown::HashMap;
pub use label::{label_dominates, SearchLabel};
pub use priority::{label_priority, surface_penalty, way_penalty};
use serde::{Deserialize, Serialize};

pub const DIST_TOL: f64 = 0.05;
pub const DPLUS_TOL: f64 = 0.15;
pub const DPLUS_TOL_FLOOR_M: f64 = 40.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteResult {
    pub nodes: Vec<NodeIndex>,
    pub start_node: NodeIndex,
    pub end_node: NodeIndex,
    pub start_name: String,
    pub end_name: String,
    pub distance_m: f64,
    pub dplus_m: f64,
    pub raw_dplus_m: f64,
    pub dminus_m: f64,
    pub raw_dminus_m: f64,
    pub overlap_ratio: f64,
    pub longest_repeated_m: f64,
    pub immediate_u_turns: usize,
    pub dead_end_visits: usize,
    pub artificial_turn_penalty: f64,
    pub surface_distances_m: HashMap<String, f64>,
    pub way_distances_m: HashMap<String, f64>,
    pub score: f64,
    pub compliant: bool,
}

pub fn score_route(
    result: &RouteResult,
    target_m: f64,
    dplus_target_m: Option<f64>,
    profile: &Profile,
    route_mode: &str,
) -> (f64, bool) {
    let distance_error = (result.distance_m - target_m).abs() / target_m.max(1.0);

    let dplus_error = match dplus_target_m {
        Some(dp) if dp > 0.0 => {
            if result.dplus_m >= dp {
                -1.2 * (0.15f64).min((result.dplus_m - dp) / dp)
            } else {
                (dp - result.dplus_m) / dp
            }
        }
        _ => 0.0,
    };

    let surf_pen = surface_penalty(&result.surface_distances_m, profile);
    let way_pen = way_penalty(&result.way_distances_m, profile);

    let score = 3.0 * distance_error
        + 2.2 * dplus_error
        + 2.5 * result.overlap_ratio
        + 1.2 * surf_pen
        + 2.0 * way_pen
        + 0.08 * result.artificial_turn_penalty
        + 0.35 * result.dead_end_visits as f64
        + 2.0 * result.immediate_u_turns as f64
        + result.longest_repeated_m / target_m.max(1.0);

    let dplus_ok = match dplus_target_m {
        Some(dp) if dp > 0.0 => {
            (result.dplus_m - dp).abs() <= (DPLUS_TOL * dp).max(DPLUS_TOL_FLOOR_M)
        }
        _ => true,
    };

    let overlap_limit = if route_mode == "natural" {
        profile.max_overlap
    } else {
        profile.max_overlap.max(0.90)
    };

    let natural_ok = route_mode == "vertical"
        || (result.immediate_u_turns == 0 && result.dead_end_visits == 0);

    let compliant = distance_error <= DIST_TOL
        && dplus_ok
        && result.overlap_ratio <= overlap_limit
        && natural_ok;

    (score, compliant)
}

pub fn make_route_result(
    graph: &Graph,
    nodes: Vec<NodeIndex>,
    profile: &Profile,
    start_name: &str,
    end_name: &str,
    target_m: f64,
    dplus_target_m: Option<f64>,
    route_mode: &str,
) -> RouteResult {
    let mut dist = 0.0;
    let mut dp = 0.0;
    let mut raw_dp = 0.0;
    let mut dm = 0.0;
    let mut raw_dm = 0.0;
    let mut surf_dist = HashMap::new();
    let mut way_dist = HashMap::new();
    let mut edge_counts: HashMap<(NodeIndex, NodeIndex), usize> = HashMap::new();
    let mut immediate_u_turns = 0;

    for i in 0..nodes.len().saturating_sub(1) {
        let u = nodes[i];
        let v = nodes[i + 1];

        if i >= 1 && nodes[i - 1] == v {
            immediate_u_turns += 1;
        }

        let key = if u < v { (u, v) } else { (v, u) };
        *edge_counts.entry(key).or_insert(0) += 1;

        if let Some(&e_idx) = graph.out_edges[u as usize]
            .iter()
            .find(|&&e| graph.edges[e as usize].target == v)
        {
            let edge = &graph.edges[e_idx as usize];
            let l = edge.length as f64;
            dist += l;
            dp += edge.d_plus as f64;
            dm += edge.d_minus as f64;
            raw_dp += edge.raw_d_plus as f64;
            raw_dm += edge.raw_d_minus as f64;

            *surf_dist.entry(edge.surface.as_str().to_string()).or_insert(0.0) += l;
            *way_dist.entry(edge.way.as_str().to_string()).or_insert(0.0) += l;
        }
    }

    let mut repeated_m = 0.0;
    let mut longest_repeated = 0.0;
    for (&(u, v), &count) in &edge_counts {
        if count > 1 {
            let edge_len = graph.out_edges[u as usize]
                .iter()
                .find(|&&e| graph.edges[e as usize].target == v)
                .map(|&e| graph.edges[e as usize].length as f64)
                .unwrap_or(0.0);
            let rep = edge_len * (count - 1) as f64;
            repeated_m += rep;
            if rep > longest_repeated {
                longest_repeated = rep;
            }
        }
    }

    let overlap_ratio = if dist > 0.0 { repeated_m / dist } else { 0.0 };
    let start_node = nodes.first().copied().unwrap_or(0);
    let end_node = nodes.last().copied().unwrap_or(0);

    let mut res = RouteResult {
        nodes,
        start_node,
        end_node,
        start_name: start_name.to_string(),
        end_name: end_name.to_string(),
        distance_m: dist,
        dplus_m: dp,
        raw_dplus_m: raw_dp,
        dminus_m: dm,
        raw_dminus_m: raw_dm,
        overlap_ratio,
        longest_repeated_m: longest_repeated,
        immediate_u_turns,
        dead_end_visits: 0,
        artificial_turn_penalty: 0.0,
        surface_distances_m: surf_dist,
        way_distances_m: way_dist,
        score: 0.0,
        compliant: false,
    };

    let (sc, comp) = score_route(&res, target_m, dplus_target_m, profile, route_mode);
    res.score = sc;
    res.compliant = comp;
    res
}

pub fn search_anchor_routes(
    graph: &Graph,
    compact: &CompactGraph,
    origin: NodeIndex,
    destinations: &[NodeIndex],
    endpoint_names: &HashMap<NodeIndex, String>,
    start_name: &str,
    target_m: f64,
    dplus_target_m: Option<f64>,
    profile: &Profile,
    route_mode: &str,
    config: &Config,
) -> Vec<RouteResult> {
    let lower_bounds = remaining_distances_to_destinations(graph, destinations);

    let start_label = SearchLabel::new(origin);
    let mut frontier = vec![start_label];
    let mut candidates: Vec<RouteResult> = Vec::new();
    let mut dominance: HashMap<(NodeIndex, i64, i64), Vec<SearchLabel>> = HashMap::new();

    let distance_upper = target_m * (1.0 + config.search_distance_slack);
    let max_hops = if target_m >= 18000.0
        || match dplus_target_m {
            Some(dp) => dp / (target_m / 1000.0) >= 30.0,
            None => false,
        } {
        14
    } else {
        config.search_max_hops
    };

    let beam_width = if target_m >= 18000.0
        || match dplus_target_m {
            Some(dp) => dp / (target_m / 1000.0) >= 30.0,
            None => false,
        } {
        300
    } else {
        config.search_beam_width
    };

    for _depth in 0..max_hops {
        let mut next_frontier: Vec<SearchLabel> = Vec::new();

        for label in &frontier {
            let Some(&src_idx) = compact.anchor_to_idx.get(&label.anchor) else {
                continue;
            };

            for seg in &compact.out_segments[src_idx] {
                if seg.nodes.len() < 2 {
                    continue;
                }

                // Check immediate reverse
                if label.nodes.len() >= 2 && label.nodes[label.nodes.len() - 2] == seg.nodes[1] {
                    continue;
                }

                // Overlap calculation
                let mut rep_dist = 0.0;
                for key in &seg.edge_keys {
                    if label.used_edges.contains(key) {
                        if let Some(&e_idx) = graph.out_edges[key.0 as usize]
                            .iter()
                            .find(|&&e| graph.edges[e as usize].target == key.1)
                        {
                            rep_dist += graph.edges[e_idx as usize].length as f64;
                        }
                    }
                }

                let pred_rep = label.repeated_m + rep_dist;
                let new_dist = label.distance_m + seg.distance_m;

                let overlap_lim = if route_mode == "natural" {
                    profile.max_overlap
                } else {
                    0.80
                };
                if pred_rep / new_dist.max(1.0) > overlap_lim {
                    continue;
                }

                if new_dist > distance_upper {
                    continue;
                }

                let rem = lower_bounds
                    .get(seg.target as usize)
                    .copied()
                    .unwrap_or(0.0);
                if new_dist + rem > distance_upper {
                    continue;
                }

                let mut new_used = label.used_edges.clone();
                for &k in &seg.edge_keys {
                    new_used.insert(k);
                }

                let mut new_nodes = label.nodes.clone();
                new_nodes.extend_from_slice(&seg.nodes[1..]);

                let mut new_anchors = label.anchor_path.clone();
                new_anchors.push(seg.target);

                let mut new_surf = label.surface_distances_m.clone();
                for (k, v) in &seg.surface_distances {
                    *new_surf.entry(k.clone()).or_insert(0.0) += v;
                }

                let mut new_way = label.way_distances_m.clone();
                for (k, v) in &seg.way_distances {
                    *new_way.entry(k.clone()).or_insert(0.0) += v;
                }

                let new_dplus = label.dplus_m + seg.dplus_m;
                let new_dminus = label.dminus_m + seg.dminus_m;

                let new_label = SearchLabel {
                    anchor: seg.target,
                    nodes: new_nodes,
                    anchor_path: new_anchors,
                    distance_m: new_dist,
                    dplus_m: new_dplus,
                    dminus_m: new_dminus,
                    repeated_m: pred_rep,
                    turn_penalty: label.turn_penalty,
                    relief_cycles: label.relief_cycles,
                    used_edges: new_used,
                    surface_distances_m: new_surf,
                    way_distances_m: new_way,
                };

                // Check destination arrival
                let is_dest = destinations.contains(&seg.target);
                if is_dest && new_label.anchor_path.len() >= 3 {
                    let end_name = endpoint_names
                        .get(&seg.target)
                        .map(|s| s.as_str())
                        .unwrap_or("end");
                    let result = make_route_result(
                        graph,
                        new_label.nodes.clone(),
                        profile,
                        start_name,
                        end_name,
                        target_m,
                        dplus_target_m,
                        route_mode,
                    );

                    let dist_err = (result.distance_m - target_m).abs() / target_m.max(1.0);
                    if dist_err <= DIST_TOL * 1.5 {
                        candidates.push(result);
                    }
                }

                let is_primary_end = endpoint_names.get(&seg.target).map(|s| s.as_str()) == Some("home")
                    || seg.target == origin
                    || !destinations.iter().any(|d| endpoint_names.get(d).map(|s| s.as_str()) == Some("home"));

                if is_primary_end && new_dist >= target_m * (1.0 + DIST_TOL) {
                    continue;
                }

                let dist_bucket = (new_dist / (target_m * 0.04).max(250.0)) as i64;
                let dp_bucket = (new_dplus / (dplus_target_m.unwrap_or(100.0) * 0.10).max(25.0)) as i64;
                let key = (seg.target, dist_bucket, dp_bucket);

                let existing = dominance.entry(key).or_default();
                if existing.iter().any(|other| label_dominates(other, &new_label)) {
                    continue;
                }
                existing.retain(|other| !label_dominates(&new_label, other));
                existing.push(new_label.clone());

                next_frontier.push(new_label);
            }
        }

        if next_frontier.is_empty() {
            break;
        }

        next_frontier.sort_by(|a, b| {
            let p_a = label_priority(a, target_m, dplus_target_m, profile, route_mode, &lower_bounds);
            let p_b = label_priority(b, target_m, dplus_target_m, profile, route_mode, &lower_bounds);
            p_a.partial_cmp(&p_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        next_frontier.truncate(beam_width);
        frontier = next_frontier;
    }

    candidates.sort_by(|a, b| {
        b.compliant
            .cmp(&a.compliant)
            .then_with(|| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
    });

    candidates
}
