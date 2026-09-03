use super::label::SearchLabel;
use crate::profiles::Profile;
use hashbrown::HashMap;

pub fn surface_penalty(surface_distances: &HashMap<String, f64>, profile: &Profile) -> f64 {
    let distance: f64 = surface_distances.values().sum();
    if distance <= 0.0 {
        return 0.0;
    }
    let baseline = profile
        .surface_weights
        .values()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let unknown_weight = *profile.surface_weights.get("unknown").unwrap_or(&1.1);

    let penalty_sum: f64 = surface_distances
        .iter()
        .map(|(surf, &len)| {
            let w = *profile.surface_weights.get(surf).unwrap_or(&unknown_weight);
            len * (w - baseline).max(0.0)
        })
        .sum();

    penalty_sum / distance
}

pub fn way_penalty(way_distances: &HashMap<String, f64>, profile: &Profile) -> f64 {
    let distance: f64 = way_distances.values().sum();
    if distance <= 0.0 {
        return 0.0;
    }
    let baseline = profile
        .way_weights
        .values()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let unknown_weight = *profile.way_weights.get("unknown").unwrap_or(&1.2);

    let penalty_sum: f64 = way_distances
        .iter()
        .map(|(way, &len)| {
            let w = *profile.way_weights.get(way).unwrap_or(&unknown_weight);
            len * (w - baseline).max(0.0)
        })
        .sum();

    penalty_sum / distance
}

pub fn label_priority(
    label: &SearchLabel,
    target_m: f64,
    dplus_target_m: Option<f64>,
    profile: &Profile,
    route_mode: &str,
    lower_bounds: &[f64],
) -> f64 {
    let min_rem = lower_bounds
        .get(label.anchor as usize)
        .copied()
        .unwrap_or(0.0);
    let min_rem = if min_rem.is_infinite() { 0.0 } else { min_rem };

    let proj_dist = label.distance_m + min_rem;
    let distance_error = if proj_dist > target_m {
        (proj_dist - target_m) / target_m.max(1.0)
    } else {
        (target_m - proj_dist) / target_m.max(1.0) * 0.4
    };

    let density = match dplus_target_m {
        Some(dp) if target_m > 0.0 => dp / (target_m / 1000.0),
        _ => 0.0,
    };

    let dplus_penalty = match dplus_target_m {
        Some(dp) if dp > 0.0 => {
            let expected_dplus = dp * (label.distance_m / target_m.max(1.0)).min(1.0);
            let dplus_deficit = (expected_dplus - label.dplus_m).max(0.0) / dp;
            let dplus_advance = (label.dplus_m - expected_dplus).max(0.0) / dp;
            let dplus_surplus = (label.dplus_m - dp * 1.15).max(0.0) / dp;

            let dplus_weight = if route_mode == "vertical" && density >= 40.0 {
                6.0
            } else if density >= 30.0 {
                4.5
            } else {
                2.5
            };

            let advance_weight = if density >= 35.0 { 3.0 } else { 1.5 };
            dplus_weight * dplus_deficit - advance_weight * dplus_advance + 2.0 * dplus_surplus
        }
        _ => 0.0,
    };

    let repetition_penalty = if route_mode == "vertical" {
        0.0
    } else {
        label.repeated_m / target_m.max(1.0) * 2.5
    };

    let surf_pen = surface_penalty(&label.surface_distances_m, profile);
    let way_pen = way_penalty(&label.way_distances_m, profile);

    2.5 * distance_error
        + dplus_penalty
        + repetition_penalty
        + 0.05 * label.turn_penalty
        + surf_pen
        + 1.5 * way_pen
}
