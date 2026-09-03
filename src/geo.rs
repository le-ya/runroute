//! Geometric utilities and Lambert-93 (EPSG:2154) projection.

use std::f64::consts::PI;

pub const DEFAULT_METRIC_CRS: &str = "EPSG:2154";

// Constants for GRS 1980 / Lambert-93
const A: f64 = 6378137.0; // semi-major axis
const E: f64 = 0.08181919104281579; // first eccentricity
const PHI1: f64 = 44.0 * PI / 180.0;
const PHI2: f64 = 49.0 * PI / 180.0;
const PHI0: f64 = 46.5 * PI / 180.0;
const LAMBDA0: f64 = 3.0 * PI / 180.0;
const X0: f64 = 700000.0;
const Y0: f64 = 6600000.0;

/// Convert WGS84 (lat, lon) in degrees to Lambert-93 (x, y) in meters.
pub fn wgs_to_l93(lat: f64, lon: f64) -> (f64, f64) {
    let m1 = PHI1.cos() / (1.0 - E * E * PHI1.sin().powi(2)).sqrt();
    let m2 = PHI2.cos() / (1.0 - E * E * PHI2.sin().powi(2)).sqrt();

    let t0 = (PI / 4.0 - PHI0 / 2.0).tan()
        / ((1.0 - E * PHI0.sin()) / (1.0 + E * PHI0.sin())).powf(E / 2.0);
    let t1 = (PI / 4.0 - PHI1 / 2.0).tan()
        / ((1.0 - E * PHI1.sin()) / (1.0 + E * PHI1.sin())).powf(E / 2.0);
    let t2 = (PI / 4.0 - PHI2 / 2.0).tan()
        / ((1.0 - E * PHI2.sin()) / (1.0 + E * PHI2.sin())).powf(E / 2.0);

    let n = (m1.ln() - m2.ln()) / (t1.ln() - t2.ln());
    let f = m1 / (n * t1.powf(n));
    let rho0 = A * f * t0.powf(n);

    let phi = lat * PI / 180.0;
    let lam = lon * PI / 180.0;

    let t = (PI / 4.0 - phi / 2.0).tan()
        / ((1.0 - E * phi.sin()) / (1.0 + E * phi.sin())).powf(E / 2.0);
    let rho = A * f * t.powf(n);
    let theta = n * (lam - LAMBDA0);

    let x = X0 + rho * theta.sin();
    let y = Y0 + rho0 - rho * theta.cos();
    (x, y)
}

/// Convert Lambert-93 (x, y) in meters to WGS84 (lat, lon) in degrees.
pub fn l93_to_wgs(x: f64, y: f64) -> (f64, f64) {
    let m1 = PHI1.cos() / (1.0 - E * E * PHI1.sin().powi(2)).sqrt();
    let m2 = PHI2.cos() / (1.0 - E * E * PHI2.sin().powi(2)).sqrt();

    let t0 = (PI / 4.0 - PHI0 / 2.0).tan()
        / ((1.0 - E * PHI0.sin()) / (1.0 + E * PHI0.sin())).powf(E / 2.0);
    let t1 = (PI / 4.0 - PHI1 / 2.0).tan()
        / ((1.0 - E * PHI1.sin()) / (1.0 + E * PHI1.sin())).powf(E / 2.0);
    let t2 = (PI / 4.0 - PHI2 / 2.0).tan()
        / ((1.0 - E * PHI2.sin()) / (1.0 + E * PHI2.sin())).powf(E / 2.0);

    let n = (m1.ln() - m2.ln()) / (t1.ln() - t2.ln());
    let f = m1 / (n * t1.powf(n));
    let rho0 = A * f * t0.powf(n);

    let dx = x - X0;
    let dy = Y0 + rho0 - y;

    let rho = (dx * dx + dy * dy).sqrt().copysign(n);
    let theta = dx.atan2(dy);

    let lam = LAMBDA0 + theta / n;
    let t = (rho / (A * f)).powf(1.0 / n);

    let mut phi = PI / 2.0 - 2.0 * t.atan();
    for _ in 0..6 {
        let esin = E * phi.sin();
        let next_phi = PI / 2.0 - 2.0 * (t * ((1.0 - esin) / (1.0 + esin)).powf(E / 2.0)).atan();
        if (next_phi - phi).abs() < 1e-12 {
            phi = next_phi;
            break;
        }
        phi = next_phi;
    }

    let lat = phi * 180.0 / PI;
    let lon = lam * 180.0 / PI;
    (lat, lon)
}

/// Euclidean distance between two projected (x, y) coordinates in meters.
#[inline]
pub fn euclidean_dist(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    (x1 - x2).hypot(y1 - y2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambert93_roundtrip() {
        let lat = 45.792478;
        let lon = 4.820567;
        let (x, y) = wgs_to_l93(lat, lon);
        assert!((x - 841417.8557).abs() < 1e-3);
        assert!((y - 6523058.7999).abs() < 1e-3);

        let (lat2, lon2) = l93_to_wgs(x, y);
        assert!((lat2 - lat).abs() < 1e-6);
        assert!((lon2 - lon).abs() < 1e-6);
    }
}
