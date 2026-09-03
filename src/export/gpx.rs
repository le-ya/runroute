//! GPX 1.1 exporter compatible with COROS / Garmin / Strava.

use crate::graph::Graph;
use crate::multicriteria::RouteResult;
use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::Path;

pub fn export_gpx(
    graph: &Graph,
    result: &RouteResult,
    out_path: &Path,
    track_name: &str,
) -> Result<()> {
    if let Some(parent) = out_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = File::create(out_path)?;

    writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        file,
        r#"<gpx version="1.1" creator="runroute" xmlns="http://www.topografix.com/GPX/1/1" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd">"#
    )?;

    writeln!(
        file,
        r#"  <metadata><name>{}</name><desc>{:.2} km, D+ {:.0} m, D- {:.0} m, overlap {:.0}%</desc></metadata>"#,
        quick_xml::escape::escape(track_name),
        result.distance_m / 1000.0,
        result.dplus_m,
        result.dminus_m,
        result.overlap_ratio * 100.0
    )?;

    // Waypoints
    if let (Some(&first), Some(&last)) = (result.nodes.first(), result.nodes.last()) {
        let f_node = &graph.nodes[first as usize];
        let l_node = &graph.nodes[last as usize];

        writeln!(
            file,
            r#"  <wpt lat="{:.7}" lon="{:.7}"><ele>{:.1}</ele><name>Départ — {}</name></wpt>"#,
            f_node.lat, f_node.lon, f_node.elevation, quick_xml::escape::escape(&result.start_name)
        )?;
        writeln!(
            file,
            r#"  <wpt lat="{:.7}" lon="{:.7}"><ele>{:.1}</ele><name>Arrivée — {}</name></wpt>"#,
            l_node.lat, l_node.lon, l_node.elevation, quick_xml::escape::escape(&result.end_name)
        )?;
    }

    // Track
    writeln!(file, r#"  <trk>"#)?;
    writeln!(file, r#"    <name>{}</name>"#, quick_xml::escape::escape(track_name))?;
    writeln!(file, r#"    <trkseg>"#)?;

    let mut prev_lat = 0.0;
    let mut prev_lon = 0.0;

    for &node_idx in &result.nodes {
        let node = &graph.nodes[node_idx as usize];
        if (node.lat - prev_lat).abs() < 1e-7 && (node.lon - prev_lon).abs() < 1e-7 {
            continue;
        }
        writeln!(
            file,
            r#"      <trkpt lat="{:.7}" lon="{:.7}"><ele>{:.1}</ele></trkpt>"#,
            node.lat, node.lon, node.elevation
        )?;
        prev_lat = node.lat;
        prev_lon = node.lon;
    }

    writeln!(file, r#"    </trkseg>"#)?;
    writeln!(file, r#"  </trk>"#)?;
    writeln!(file, r#"</gpx>"#)?;

    Ok(())
}
