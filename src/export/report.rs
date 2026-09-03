//! JSON report export conforming to schema 2.3.

use crate::multicriteria::RouteResult;
use anyhow::Result;
use hashbrown::HashMap;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub const REPORT_SCHEMA_VERSION: &str = "2.3";

pub fn export_report(
    result: &RouteResult,
    candidates: &[RouteResult],
    target_distance_m: f64,
    target_dplus_m: Option<f64>,
    profile_name: &str,
    route_mode: &str,
    seed: u64,
    out_path: &Path,
) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let route_payload = |r: &RouteResult, rank: usize| {
        let dist_err = (r.distance_m - target_distance_m) / target_distance_m * 100.0;
        let dplus_err = target_dplus_m.map(|dp| (r.dplus_m - dp) / dp * 100.0);

        let mut surf_ratios = HashMap::new();
        let mut way_ratios = HashMap::new();
        if r.distance_m > 0.0 {
            for (k, &v) in &r.surface_distances_m {
                surf_ratios.insert(k.clone(), v / r.distance_m);
            }
            for (k, &v) in &r.way_distances_m {
                way_ratios.insert(k.clone(), v / r.distance_m);
            }
        }

        json!({
            "rank": rank,
            "logistic_level": "direct",
            "logistic_rank": rank,
            "profile": profile_name,
            "requested_start": r.start_name,
            "start": r.start_name,
            "end": r.end_name,
            "seed": seed,
            "route_mode": route_mode,
            "compliant": r.compliant,
            "degraded": !r.compliant,
            "score": r.score,
            "distance_km": (r.distance_m / 1000.0 * 10000.0).round() / 10000.0,
            "target_distance_km": target_distance_m / 1000.0,
            "distance_error_pct": dist_err,
            "dplus_m": (r.dplus_m * 10.0).round() / 10.0,
            "raw_dplus_m": r.raw_dplus_m,
            "elevation_source": "rge_alti_5m",
            "elevation_metric": "gpx.studio: RDP 20 m + moyenne glissante 100 m",
            "target_dplus_m": target_dplus_m,
            "dplus_error_pct": dplus_err,
            "dminus_m": (r.dminus_m * 10.0).round() / 10.0,
            "raw_dminus_m": r.raw_dminus_m,
            "surface_distances_m": r.surface_distances_m,
            "surface_ratios": surf_ratios,
            "way_distances_m": r.way_distances_m,
            "way_ratios": way_ratios,
            "overlap_ratio": r.overlap_ratio,
            "longest_repeated_m": r.longest_repeated_m,
            "immediate_u_turns": r.immediate_u_turns,
            "dead_end_visits": r.dead_end_visits,
            "node_count": r.nodes.len(),
        })
    };

    let candidates_payload: Vec<_> = candidates
        .iter()
        .enumerate()
        .take(10)
        .map(|(i, c)| route_payload(c, i + 1))
        .collect();

    let root = json!({
        "schema_version": REPORT_SCHEMA_VERSION,
        "date": chrono::Local::now().format("%Y-%m-%d").to_string(),
        "route": route_payload(result, 1),
        "candidates": candidates_payload,
    });

    let mut file = File::create(out_path)?;
    serde_json::to_writer_pretty(&mut file, &root)?;
    writeln!(file)?;

    Ok(())
}
