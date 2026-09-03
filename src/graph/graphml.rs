use super::types::{Edge, Node, NodeIndex, SurfaceClass, WayClass};
use anyhow::{Context, Result};
use hashbrown::HashMap;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub struct ParsedGraphML {
    pub nodes: Vec<Node>,
    pub osm_to_idx: HashMap<u64, NodeIndex>,
    pub edges: Vec<Edge>,
}

pub fn parse_graphml<P: AsRef<Path>>(path: P) -> Result<ParsedGraphML> {
    let file = File::open(path.as_ref())
        .with_context(|| format!("Impossible d'ouvrir le fichier GraphML: {:?}", path.as_ref()))?;
    let mut reader = Reader::from_reader(BufReader::with_capacity(1024 * 1024, file));
    reader.config_mut().trim_text(true);

    let mut key_attr_names: HashMap<String, String> = HashMap::new();

    let mut nodes: Vec<Node> = Vec::with_capacity(160_000);
    let mut osm_to_idx: HashMap<u64, NodeIndex> = HashMap::with_capacity(160_000);
    let mut edges: Vec<Edge> = Vec::with_capacity(460_000);

    let mut buf = Vec::with_capacity(8192);

    // Temp state
    let mut in_node = false;
    let mut in_edge = false;
    let mut current_key: Option<String> = None;

    let mut cur_node_id: u64 = 0;
    let mut cur_node_x = 0.0;
    let mut cur_node_y = 0.0;
    let mut cur_node_lat = 0.0;
    let mut cur_node_lon = 0.0;
    let mut cur_node_elev = 0.0f32;

    let mut cur_edge_src: u64 = 0;
    let mut cur_edge_tgt: u64 = 0;
    let mut cur_edge_len = 0.0f32;
    let mut cur_edge_slope = 0.0f32;
    let mut cur_edge_dp = 0.0f32;
    let mut cur_edge_dm = 0.0f32;
    let mut cur_edge_raw_dp = 0.0f32;
    let mut cur_edge_raw_dm = 0.0f32;
    let mut cur_edge_surface = SurfaceClass::Unknown;
    let mut cur_edge_way = WayClass::Unknown;
    let mut cur_edge_name: Option<String> = None;
    let mut cur_edge_elev_profile: Vec<f32> = Vec::new();
    let mut cur_edge_wkt: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => match e.name().as_ref() {
                b"key" => {
                    let mut id = String::new();
                    let mut name = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"id" {
                            id = String::from_utf8_lossy(&attr.value).to_string();
                        } else if attr.key.as_ref() == b"attr.name" {
                            name = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                    if !id.is_empty() && !name.is_empty() {
                        key_attr_names.insert(id, name);
                    }
                }
                b"node" => {
                    in_node = true;
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"id" {
                            if let Ok(id_val) = std::str::from_utf8(&attr.value) {
                                cur_node_id = id_val.parse().unwrap_or(0);
                            }
                        }
                    }
                    cur_node_x = 0.0;
                    cur_node_y = 0.0;
                    cur_node_lat = 0.0;
                    cur_node_lon = 0.0;
                    cur_node_elev = 0.0;
                }
                b"edge" => {
                    in_edge = true;
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"source" {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                cur_edge_src = val.parse().unwrap_or(0);
                            }
                        } else if attr.key.as_ref() == b"target" {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                cur_edge_tgt = val.parse().unwrap_or(0);
                            }
                        }
                    }
                    cur_edge_len = 0.0;
                    cur_edge_slope = 0.0;
                    cur_edge_dp = 0.0;
                    cur_edge_dm = 0.0;
                    cur_edge_raw_dp = 0.0;
                    cur_edge_raw_dm = 0.0;
                    cur_edge_surface = SurfaceClass::Unknown;
                    cur_edge_way = WayClass::Unknown;
                    cur_edge_name = None;
                    cur_edge_elev_profile.clear();
                    cur_edge_wkt = None;
                }
                b"data" => {
                    current_key = None;
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"key" {
                            if let Ok(k) = std::str::from_utf8(&attr.value) {
                                current_key = Some(k.to_string());
                            }
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"key" {
                    let mut id = String::new();
                    let mut name = String::new();
                    for attr in e.attributes().flatten() {
                        if attr.key.as_ref() == b"id" {
                            id = String::from_utf8_lossy(&attr.value).to_string();
                        } else if attr.key.as_ref() == b"attr.name" {
                            name = String::from_utf8_lossy(&attr.value).to_string();
                        }
                    }
                    if !id.is_empty() && !name.is_empty() {
                        key_attr_names.insert(id, name);
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if let Some(ref k) = current_key {
                    let attr_name = key_attr_names.get(k).map(|s| s.as_str()).unwrap_or("");
                    let text = e.unescape().unwrap_or_default();
                    if in_node {
                        match attr_name {
                            "x" => cur_node_x = text.parse().unwrap_or(0.0),
                            "y" => cur_node_y = text.parse().unwrap_or(0.0),
                            "lat" => cur_node_lat = text.parse().unwrap_or(0.0),
                            "lon" => cur_node_lon = text.parse().unwrap_or(0.0),
                            "elevation" => cur_node_elev = text.parse().unwrap_or(0.0),
                            _ => {}
                        }
                    } else if in_edge {
                        match attr_name {
                            "length" => cur_edge_len = text.parse().unwrap_or(0.0),
                            "slope_pct" => cur_edge_slope = text.parse().unwrap_or(0.0),
                            "d_plus" => cur_edge_dp = text.parse().unwrap_or(0.0),
                            "d_minus" => cur_edge_dm = text.parse().unwrap_or(0.0),
                            "raw_d_plus" => cur_edge_raw_dp = text.parse().unwrap_or(0.0),
                            "raw_d_minus" => cur_edge_raw_dm = text.parse().unwrap_or(0.0),
                            "surface_class" => cur_edge_surface = SurfaceClass::from_str(&text),
                            "way_class" => cur_edge_way = WayClass::from_str(&text),
                            "osm_name" => {
                                if !text.is_empty() {
                                    cur_edge_name = Some(text.to_string());
                                }
                            }
                            "elevation_profile_json" => {
                                if text.starts_with('[') && text.ends_with(']') {
                                    let inner = &text[1..text.len() - 1];
                                    cur_edge_elev_profile = inner
                                        .split(',')
                                        .filter_map(|s| s.trim().parse::<f32>().ok())
                                        .collect();
                                }
                            }
                            "geometry_wkt" => {
                                if !text.is_empty() {
                                    cur_edge_wkt = Some(text.to_string());
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.name().as_ref() {
                b"data" => {
                    current_key = None;
                }
                b"node" => {
                    let idx = nodes.len() as NodeIndex;
                    nodes.push(Node {
                        osm_id: cur_node_id,
                        x: cur_node_x,
                        y: cur_node_y,
                        lat: cur_node_lat,
                        lon: cur_node_lon,
                        elevation: cur_node_elev,
                    });
                    osm_to_idx.insert(cur_node_id, idx);
                    in_node = false;
                }
                b"edge" => {
                    if let (Some(&src_idx), Some(&tgt_idx)) =
                        (osm_to_idx.get(&cur_edge_src), osm_to_idx.get(&cur_edge_tgt))
                    {
                        edges.push(Edge {
                            source: src_idx,
                            target: tgt_idx,
                            length: cur_edge_len,
                            slope_pct: cur_edge_slope,
                            d_plus: cur_edge_dp,
                            d_minus: cur_edge_dm,
                            raw_d_plus: cur_edge_raw_dp,
                            raw_d_minus: cur_edge_raw_dm,
                            surface: cur_edge_surface,
                            way: cur_edge_way,
                            osm_name: cur_edge_name.take(),
                            elevation_profile: std::mem::take(&mut cur_edge_elev_profile),
                            geometry_wkt: cur_edge_wkt.take(),
                        });
                    }
                    in_edge = false;
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(err) => anyhow::bail!("Erreur parsing GraphML à la position {}: {:?}", reader.buffer_position(), err),
            _ => {}
        }
        buf.clear();
    }

    Ok(ParsedGraphML {
        nodes,
        osm_to_idx,
        edges,
    })
}
