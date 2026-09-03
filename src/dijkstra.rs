//! High-performance Dijkstra shortest path algorithm with Rayon parallelism.

use crate::costs::edge_cost;
use crate::graph::types::NodeIndex;
use crate::graph::Graph;
use crate::profiles::Profile;
use rayon::prelude::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, PartialEq)]
struct State {
    cost: f64,
    node: NodeIndex,
}

impl Eq for State {}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Compute shortest path between `start` and `target` for `profile`.
/// Returns `Some((nodes, cost))` or `None` if no path exists.
pub fn shortest_path(
    graph: &Graph,
    start: NodeIndex,
    target: NodeIndex,
    profile: &Profile,
) -> Option<(Vec<NodeIndex>, f64)> {
    if start == target {
        return Some((vec![start], 0.0));
    }

    let n = graph.node_count();
    let mut dist = vec![f64::INFINITY; n];
    let mut prev = vec![NodeIndex::MAX; n];
    let mut heap = BinaryHeap::new();

    dist[start as usize] = 0.0;
    heap.push(State {
        cost: 0.0,
        node: start,
    });

    while let Some(State { cost, node }) = heap.pop() {
        if node == target {
            let mut path = Vec::new();
            let mut curr = target;
            while curr != start {
                path.push(curr);
                curr = prev[curr as usize];
            }
            path.push(start);
            path.reverse();
            return Some((path, cost));
        }

        if cost > dist[node as usize] {
            continue;
        }

        for &edge_idx in &graph.out_edges[node as usize] {
            let edge = &graph.edges[edge_idx as usize];
            let next_node = edge.target;
            let weight = edge_cost(edge, profile);
            let next_cost = cost + weight;

            if next_cost < dist[next_node as usize] {
                dist[next_node as usize] = next_cost;
                prev[next_node as usize] = node;
                heap.push(State {
                    cost: next_cost,
                    node: next_node,
                });
            }
        }
    }

    None
}

/// Distance from all nodes to a set of destination nodes (computed via reversed graph).
pub fn remaining_distances_to_destinations(
    graph: &Graph,
    destinations: &[NodeIndex],
) -> Vec<f64> {
    let n = graph.node_count();
    let mut bounds = vec![f64::INFINITY; n];
    let mut heap = BinaryHeap::new();

    for &dest in destinations {
        bounds[dest as usize] = 0.0;
        heap.push(State {
            cost: 0.0,
            node: dest,
        });
    }

    // Traverse incoming edges (reverse direction) using edge length
    while let Some(State { cost, node }) = heap.pop() {
        if cost > bounds[node as usize] {
            continue;
        }

        for &edge_idx in &graph.in_edges[node as usize] {
            let edge = &graph.edges[edge_idx as usize];
            let prev_node = edge.source;
            let next_cost = cost + edge.length as f64;

            if next_cost < bounds[prev_node as usize] {
                bounds[prev_node as usize] = next_cost;
                heap.push(State {
                    cost: next_cost,
                    node: prev_node,
                });
            }
        }
    }

    bounds
}

/// Parallel precomputation of pairwise paths between anchors using Rayon.
pub fn compute_pairwise_paths(
    graph: &Graph,
    pairs: &[(NodeIndex, NodeIndex)],
    profile: &Profile,
) -> Vec<Option<(Vec<NodeIndex>, f64)>> {
    pairs
        .par_iter()
        .map(|&(start, target)| shortest_path(graph, start, target, profile))
        .collect()
}
