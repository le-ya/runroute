use serde::{Deserialize, Serialize};

pub type NodeIndex = u32;
pub type EdgeIndex = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub osm_id: u64,
    pub x: f64,
    pub y: f64,
    pub lat: f64,
    pub lon: f64,
    pub elevation: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SurfaceClass {
    Paved,
    Path,
    Trail,
    Steps,
    Unknown,
}

impl SurfaceClass {
    pub fn from_str(s: &str) -> Self {
        match s {
            "paved" => SurfaceClass::Paved,
            "path" => SurfaceClass::Path,
            "trail" => SurfaceClass::Trail,
            "steps" => SurfaceClass::Steps,
            _ => SurfaceClass::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            SurfaceClass::Paved => "paved",
            SurfaceClass::Path => "path",
            SurfaceClass::Trail => "trail",
            SurfaceClass::Steps => "steps",
            SurfaceClass::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WayClass {
    MainRoad,
    LocalRoad,
    QuietRoad,
    Pedestrian,
    Path,
    Unknown,
}

impl WayClass {
    pub fn from_str(s: &str) -> Self {
        match s {
            "main_road" => WayClass::MainRoad,
            "local_road" => WayClass::LocalRoad,
            "quiet_road" => WayClass::QuietRoad,
            "pedestrian" => WayClass::Pedestrian,
            "path" => WayClass::Path,
            _ => WayClass::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            WayClass::MainRoad => "main_road",
            WayClass::LocalRoad => "local_road",
            WayClass::QuietRoad => "quiet_road",
            WayClass::Pedestrian => "pedestrian",
            WayClass::Path => "path",
            WayClass::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: NodeIndex,
    pub target: NodeIndex,
    pub length: f32,
    pub slope_pct: f32,
    pub d_plus: f32,
    pub d_minus: f32,
    pub raw_d_plus: f32,
    pub raw_d_minus: f32,
    pub surface: SurfaceClass,
    pub way: WayClass,
    pub osm_name: Option<String>,
    pub elevation_profile: Vec<f32>,
    pub geometry_wkt: Option<String>,
}
