//! Configuration loader from TOML.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    #[serde(default = "default_bbox")]
    pub bbox: [f64; 4],

    #[serde(default = "default_points")]
    pub points: HashMap<String, (f64, f64)>,

    #[serde(default = "default_velov_max_km")]
    pub velov_max_km: f64,

    #[serde(default = "default_max_distance_km")]
    pub max_distance_km: f64,

    #[serde(default = "default_max_dplus_m")]
    pub max_dplus_m: f64,

    #[serde(default = "default_timeout_s")]
    pub vertical_search_timeout_s: f64,

    #[serde(default = "default_anchor_max_count")]
    pub anchor_max_count: usize,

    #[serde(default = "default_anchor_neighbor_count")]
    pub anchor_neighbor_count: usize,

    #[serde(default = "default_anchor_min_spacing_m")]
    pub anchor_min_spacing_m: f64,

    #[serde(default = "default_search_beam_width")]
    pub search_beam_width: usize,

    #[serde(default = "default_search_max_hops")]
    pub search_max_hops: usize,

    #[serde(default = "default_search_max_candidates")]
    pub search_max_candidates: usize,

    #[serde(default = "default_search_max_endpoints")]
    pub search_max_endpoints: usize,

    #[serde(default = "default_search_distance_slack")]
    pub search_distance_slack: f64,

    #[serde(default = "default_search_dplus_slack")]
    pub search_dplus_slack: f64,

    #[serde(default = "default_elevation_source")]
    pub elevation_source: String,

    #[serde(default = "default_sample_spacing")]
    pub elevation_sample_spacing_m: f64,

    #[serde(default = "default_export_spacing")]
    pub elevation_export_spacing_m: f64,

    #[serde(default = "default_median_window")]
    pub elevation_median_window: usize,

    #[serde(default = "default_noise_threshold")]
    pub elevation_noise_threshold_m: f64,
}

fn default_data_dir() -> PathBuf {
    PathBuf::from("data")
}
fn default_output_dir() -> PathBuf {
    PathBuf::from("gpx")
}
fn default_bbox() -> [f64; 4] {
    [45.70, 4.55, 45.95, 4.90]
}
fn default_points() -> HashMap<String, (f64, f64)> {
    let mut m = HashMap::new();
    m.insert("home".to_string(), (45.786902, 4.7896771));
    m.insert("ile_barbe".to_string(), (45.797415, 4.829038));
    m.insert("carret_sedallian".to_string(), (45.792478, 4.820567));
    m.insert("couzon".to_string(), (45.846034, 4.832409));
    m
}
fn default_velov_max_km() -> f64 {
    2.0
}
fn default_max_distance_km() -> f64 {
    25.0
}
fn default_max_dplus_m() -> f64 {
    1200.0
}
fn default_timeout_s() -> f64 {
    120.0
}
fn default_anchor_max_count() -> usize {
    36
}
fn default_anchor_neighbor_count() -> usize {
    5
}
fn default_anchor_min_spacing_m() -> f64 {
    300.0
}
fn default_search_beam_width() -> usize {
    240
}
fn default_search_max_hops() -> usize {
    10
}
fn default_search_max_candidates() -> usize {
    80
}
fn default_search_max_endpoints() -> usize {
    10
}
fn default_search_distance_slack() -> f64 {
    0.30
}
fn default_search_dplus_slack() -> f64 {
    0.40
}
fn default_elevation_source() -> String {
    "rge_alti_5m".to_string()
}
fn default_sample_spacing() -> f64 {
    10.0
}
fn default_export_spacing() -> f64 {
    25.0
}
fn default_median_window() -> usize {
    1
}
fn default_noise_threshold() -> f64 {
    1.0
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir: default_data_dir(),
            output_dir: default_output_dir(),
            bbox: default_bbox(),
            points: default_points(),
            velov_max_km: default_velov_max_km(),
            max_distance_km: default_max_distance_km(),
            max_dplus_m: default_max_dplus_m(),
            vertical_search_timeout_s: default_timeout_s(),
            anchor_max_count: default_anchor_max_count(),
            anchor_neighbor_count: default_anchor_neighbor_count(),
            anchor_min_spacing_m: default_anchor_min_spacing_m(),
            search_beam_width: default_search_beam_width(),
            search_max_hops: default_search_max_hops(),
            search_max_candidates: default_search_max_candidates(),
            search_max_endpoints: default_search_max_endpoints(),
            search_distance_slack: default_search_distance_slack(),
            search_dplus_slack: default_search_dplus_slack(),
            elevation_source: default_elevation_source(),
            elevation_sample_spacing_m: default_sample_spacing(),
            elevation_export_spacing_m: default_export_spacing(),
            elevation_median_window: default_median_window(),
            elevation_noise_threshold_m: default_noise_threshold(),
        }
    }
}

pub fn load_config<P: AsRef<Path>>(path: Option<P>) -> anyhow::Result<Config> {
    let mut config = Config::default();
    let config_path = if let Some(p) = path {
        Some(p.as_ref().to_path_buf())
    } else if Path::new("runroute.toml").exists() {
        Some(PathBuf::from("runroute.toml"))
    } else if Path::new("../runroute/runroute.toml").exists() {
        Some(PathBuf::from("../runroute/runroute.toml"))
    } else {
        None
    };

    if let Some(cp) = config_path {
        if cp.exists() {
            let content = std::fs::read_to_string(&cp)?;
            config = toml::from_str(&content)?;
        }
    }
    Ok(config)
}
