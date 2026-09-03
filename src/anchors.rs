//! Anchor selection algorithm for spatial distribution and vertical relief.

use crate::geo::euclidean_dist;
use crate::graph::types::NodeIndex;
use crate::graph::Graph;
use hashbrown::HashSet;

pub fn select_anchor_nodes(
    graph: &Graph,
    mandatory: &[NodeIndex],
    target_m: f64,
    max_count: usize,
    min_spacing_m: f64,
    dplus_target_m: Option<f64>,
) -> Vec<NodeIndex> {
    let mut selected: Vec<NodeIndex> = Vec::with_capacity(max_count);
    let mut selected_set: HashSet<NodeIndex> = HashSet::new();

    for &node in mandatory {
        if selected_set.insert(node) {
            selected.push(node);
        }
    }

    if graph.node_count() <= max_count {
        for i in 0..graph.node_count() as NodeIndex {
            if selected_set.insert(i) {
                selected.push(i);
            }
        }
        return selected;
    }

    let density = match dplus_target_m {
        Some(dp) if target_m > 0.0 => dp / (target_m / 1000.0),
        _ => 0.0,
    };
    let reach_factor = if density >= 30.0 { 1.15 } else { 0.92 };
    let reach = (target_m * 0.75).max(2000.0);

    let origin = selected.first().copied();
    let (ox, oy) = match origin {
        Some(o) => (graph.nodes[o as usize].x, graph.nodes[o as usize].y),
        None => (0.0, 0.0),
    };

    let req_coords: Vec<(f64, f64, f32)> = selected
        .iter()
        .map(|&p| {
            let n = &graph.nodes[p as usize];
            (n.x, n.y, n.elevation)
        })
        .collect();

    let mut eligible_nodes: Vec<NodeIndex> = Vec::new();
    let mut relief_efficiency: Vec<f64> = vec![0.0; graph.node_count()];
    let mut candidates: Vec<(f64, NodeIndex)> = Vec::new();

    for (i, node) in graph.nodes.iter().enumerate() {
        let node_idx = i as NodeIndex;
        if selected_set.contains(&node_idx) {
            continue;
        }

        let nx = node.x;
        let ny = node.y;
        let elev = node.elevation;

        if !req_coords.is_empty() {
            let mut min_dist = f64::INFINITY;
            let mut baseline = 0.0;
            for &(rx, ry, r_elev) in &req_coords {
                let d = euclidean_dist(nx, ny, rx, ry);
                if d < min_dist {
                    min_dist = d;
                    baseline = r_elev as f64;
                }
            }

            if min_dist > reach {
                continue;
            }

            if origin.is_some() {
                let dist_orig = euclidean_dist(nx, ny, ox, oy);
                if dist_orig + min_dist > target_m * reach_factor {
                    continue;
                }
            }

            relief_efficiency[i] = ((elev as f64 - baseline).max(0.0)) / min_dist.max(300.0);
        }

        eligible_nodes.push(node_idx);
        let score = graph.feature_score(node_idx);
        if score >= 2.0 && graph.junction_degree(node_idx) >= 2 {
            candidates.push((score, node_idx));
        }
    }

    candidates.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let mut add_distributed = |nodes: &[NodeIndex], limit: usize, spacing: f64| -> Vec<NodeIndex> {
        let mut added = Vec::new();
        for &node in nodes {
            if selected_set.contains(&node) {
                continue;
            }
            let nx = graph.nodes[node as usize].x;
            let ny = graph.nodes[node as usize].y;
            let far_enough = selected.iter().all(|&other| {
                let ox = graph.nodes[other as usize].x;
                let oy = graph.nodes[other as usize].y;
                euclidean_dist(nx, ny, ox, oy) >= spacing
            });

            if far_enough {
                selected.push(node);
                selected_set.insert(node);
                added.push(node);
                if added.len() >= limit || selected.len() >= max_count {
                    break;
                }
            }
        }
        added
    };

    // Relief bands
    let per_band = if density >= 30.0 {
        4.max(max_count / 10)
    } else {
        3.max(max_count / 15)
    };
    let high_spacing = if density >= 30.0 { 500.0 } else { 650.0 };

    let bands = [
        (550.0f32, f32::INFINITY),
        (450.0f32, 550.0f32),
        (350.0f32, 450.0f32),
    ];

    let mut highs = Vec::new();
    for &(low, high) in &bands {
        let mut band_nodes: Vec<NodeIndex> = eligible_nodes
            .iter()
            .copied()
            .filter(|&n| {
                let e = graph.nodes[n as usize].elevation;
                e >= low && e < high && graph.junction_degree(n) >= 2
            })
            .collect();

        band_nodes.sort_by(|&a, &b| {
            let eff_a = relief_efficiency[a as usize];
            let eff_b = relief_efficiency[b as usize];
            eff_b
                .partial_cmp(&eff_a)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    let ea = graph.nodes[a as usize].elevation;
                    let eb = graph.nodes[b as usize].elevation;
                    eb.partial_cmp(&ea).unwrap_or(std::cmp::Ordering::Equal)
                })
        });

        highs.extend(add_distributed(&band_nodes, per_band, high_spacing));
    }

    if highs.len() < 4.max(max_count / 5) {
        let mut all_high: Vec<NodeIndex> = eligible_nodes
            .iter()
            .copied()
            .filter(|&n| graph.junction_degree(n) >= 2)
            .collect();
        all_high.sort_by(|&a, &b| {
            let ea = graph.nodes[a as usize].elevation;
            let eb = graph.nodes[b as usize].elevation;
            eb.partial_cmp(&ea).unwrap_or(std::cmp::Ordering::Equal)
        });
        highs.extend(add_distributed(
            &all_high,
            4.max(max_count / 5) - highs.len(),
            high_spacing,
        ));
    }

    // Local saddles / valleys
    let mut local_lows = Vec::new();
    for &high in &highs {
        let h_node = &graph.nodes[high as usize];
        let h_elev = h_node.elevation;

        let nearby: Vec<NodeIndex> = eligible_nodes
            .iter()
            .copied()
            .filter(|&n| {
                let n_node = &graph.nodes[n as usize];
                let d = euclidean_dist(h_node.x, h_node.y, n_node.x, n_node.y);
                d >= 500.0 && d <= 2500.0 && graph.junction_degree(n) >= 2 && (h_elev - n_node.elevation) >= 45.0
            })
            .collect();

        if let Some(&best_low) = nearby.iter().max_by(|&&a, &&b| {
            let da = euclidean_dist(h_node.x, h_node.y, graph.nodes[a as usize].x, graph.nodes[a as usize].y);
            let db = euclidean_dist(h_node.x, h_node.y, graph.nodes[b as usize].x, graph.nodes[b as usize].y);
            let drop_a = (h_elev - graph.nodes[a as usize].elevation) as f64 / da.max(1.0);
            let drop_b = (h_elev - graph.nodes[b as usize].elevation) as f64 / db.max(1.0);
            drop_a.partial_cmp(&drop_b).unwrap_or(std::cmp::Ordering::Equal)
        }) {
            local_lows.push(best_low);
        }
    }

    let low_quota = if density >= 30.0 { 6.max(max_count / 5) } else { 4.max(max_count / 5) };
    add_distributed(&local_lows, low_quota, min_spacing_m);

    // Feature score candidates
    for &(_, cand_node) in &candidates {
        if selected.len() >= max_count {
            break;
        }
        let nx = graph.nodes[cand_node as usize].x;
        let ny = graph.nodes[cand_node as usize].y;
        let far_enough = selected.iter().all(|&other| {
            let ox = graph.nodes[other as usize].x;
            let oy = graph.nodes[other as usize].y;
            euclidean_dist(nx, ny, ox, oy) >= min_spacing_m
        });
        if far_enough {
            selected.push(cand_node);
            selected_set.insert(cand_node);
        }
    }

    selected
}
