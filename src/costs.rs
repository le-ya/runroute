use crate::graph::types::Edge;
use crate::profiles::Profile;

pub const OVERLAP_PENALTY: f64 = 4.0;

#[inline]
pub fn edge_cost(edge: &Edge, profile: &Profile) -> f64 {
    let length = edge.length as f64;
    let slope = edge.slope_pct as f64;

    let up = slope.max(0.0);
    let down = (-slope).max(0.0);

    let slope_factor = 1.0
        + profile.up_weight * (up / 10.0).powi(2)
        + profile.down_weight * (down / 10.0).powi(2);

    let surface_str = edge.surface.as_str();
    let surface_factor = profile.surface_weight(surface_str);

    let way_str = edge.way.as_str();
    let way_factor = profile.way_weight(way_str);

    length * slope_factor * surface_factor * way_factor
}
