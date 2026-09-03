//! GeoJSON alternatives export.

use crate::graph::Graph;
use crate::multicriteria::RouteResult;
use anyhow::Result;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_alternatives_geojson(
    graph: &Graph,
    candidates: &[RouteResult],
    out_path: &Path,
) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let colors = ["#2563eb", "#10b981", "#f59e0b", "#ef4444", "#8b5cf6"];

    let mut features = Vec::new();
    for (i, c) in candidates.iter().take(5).enumerate() {
        let color = colors[i % colors.len()];
        let coordinates: Vec<[f64; 2]> = c
            .nodes
            .iter()
            .map(|&idx| {
                let n = &graph.nodes[idx as usize];
                [n.lon, n.lat]
            })
            .collect();

        features.push(json!({
            "type": "Feature",
            "properties": {
                "rank": i + 1,
                "distance_km": (c.distance_m / 1000.0 * 10.0).round() / 10.0,
                "dplus_m": c.dplus_m.round(),
                "stroke": color,
                "stroke-width": if i == 0 { 5 } else { 3 },
                "stroke-opacity": if i == 0 { 0.9 } else { 0.6 },
            },
            "geometry": {
                "type": "LineString",
                "coordinates": coordinates,
            }
        }));
    }

    let geojson = json!({
        "type": "FeatureCollection",
        "features": features,
    });

    let mut file = File::create(out_path)?;
    serde_json::to_writer_pretty(&mut file, &geojson)?;
    writeln!(file)?;

    Ok(())
}
