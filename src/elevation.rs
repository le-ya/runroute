//! Elevation smoothing and analysis (RDP 20m + 100m moving average).

/// Point with (distance_m, elevation_m).
#[derive(Debug, Clone, Copy)]
pub struct ElevPoint {
    pub dist: f64,
    pub elev: f64,
}

/// Perpendicular distance from point p to segment (p1, p2).
fn perpendicular_dist(p: ElevPoint, p1: ElevPoint, p2: ElevPoint) -> f64 {
    let dx = p2.dist - p1.dist;
    let dy = p2.elev - p1.elev;

    if dx == 0.0 && dy == 0.0 {
        return (p.dist - p1.dist).hypot(p.elev - p1.elev);
    }

    let numerator = (dy * p.dist - dx * p.elev + p2.dist * p1.elev - p2.elev * p1.dist).abs();
    let denominator = dx.hypot(dy);
    numerator / denominator
}

/// Ramer-Douglas-Peucker simplification for elevation profile.
pub fn rdp_simplify(points: &[ElevPoint], epsilon: f64) -> Vec<ElevPoint> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut dmax = 0.0;
    let mut index = 0;
    let end = points.len() - 1;

    for i in 1..end {
        let d = perpendicular_dist(points[i], points[0], points[end]);
        if d > dmax {
            index = i;
            dmax = d;
        }
    }

    if dmax > epsilon {
        let mut rec1 = rdp_simplify(&points[..=index], epsilon);
        let rec2 = rdp_simplify(&points[index..], epsilon);
        rec1.pop();
        rec1.extend(rec2);
        rec1
    } else {
        vec![points[0], points[end]]
    }
}

/// Compute rolling average of elevations with a window of `window_m` (e.g. 100m).
pub fn moving_average_elevation(points: &[ElevPoint], window_m: f64) -> Vec<ElevPoint> {
    if points.len() <= 1 {
        return points.to_vec();
    }

    let half = window_m / 2.0;
    let mut smoothed = Vec::with_capacity(points.len());

    for p in points {
        let min_d = p.dist - half;
        let max_d = p.dist + half;

        let mut sum_elev = 0.0;
        let mut count = 0;

        for other in points {
            if other.dist >= min_d && other.dist <= max_d {
                sum_elev += other.elev;
                count += 1;
            }
        }

        let avg = if count > 0 { sum_elev / count as f64 } else { p.elev };
        smoothed.push(ElevPoint {
            dist: p.dist,
            elev: avg,
        });
    }

    smoothed
}

/// Calculate positive ($D^+$) and negative ($D^-$) elevation gain from a profile.
pub fn compute_dplus_dminus(points: &[ElevPoint]) -> (f64, f64) {
    let mut dplus = 0.0;
    let mut dminus = 0.0;

    for i in 0..points.len().saturating_sub(1) {
        let diff = points[i + 1].elev - points[i].elev;
        if diff > 0.0 {
            dplus += diff;
        } else {
            dminus += -diff;
        }
    }

    (dplus, dminus)
}

/// Full gpx.studio smoothing metric: RDP 20m + 100m moving average.
pub fn smooth_elevation_gpx_studio(raw_points: &[ElevPoint]) -> (f64, f64, Vec<ElevPoint>) {
    if raw_points.len() <= 2 {
        let (dp, dm) = compute_dplus_dminus(raw_points);
        return (dp, dm, raw_points.to_vec());
    }

    let simplified = rdp_simplify(raw_points, 20.0);
    let smoothed = moving_average_elevation(&simplified, 100.0);
    let (dplus, dminus) = compute_dplus_dminus(&smoothed);

    (dplus, dminus, smoothed)
}
