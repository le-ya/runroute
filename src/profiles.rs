//! Profile definitions and cost preferences.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub up_weight: f64,
    pub down_weight: f64,
    pub surface_weights: HashMap<String, f64>,
    pub way_weights: HashMap<String, f64>,
    pub max_overlap: f64,
    pub repetitions: bool,
    pub description: String,
}

impl Profile {
    pub fn surface_weight(&self, surface: &str) -> f64 {
        *self.surface_weights.get(surface).unwrap_or(&1.1)
    }

    pub fn way_weight(&self, way: &str) -> f64 {
        let unknown = *self.way_weights.get("unknown").unwrap_or(&1.20);
        *self.way_weights.get(way).unwrap_or(&unknown)
    }
}

pub const ELEVATION_DENSITY_THRESHOLD: f64 = 25.0; // m D+ / km

pub fn routing_profile_for_target(
    profile: &Profile,
    target_m: f64,
    dplus_target_m: Option<f64>,
) -> Profile {
    let Some(dplus) = dplus_target_m else {
        return profile.clone();
    };
    if target_m <= 0.0 {
        return profile.clone();
    }
    let density = dplus / (target_m / 1000.0);
    if density < ELEVATION_DENSITY_THRESHOLD {
        profile.clone()
    } else {
        let mut p = profile.clone();
        p.up_weight = p.up_weight.min(0.20);
        p.down_weight = p.down_weight.min(0.35);
        p
    }
}

fn pleasant_ways() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("path".to_string(), 0.70);
    m.insert("pedestrian".to_string(), 0.80);
    m.insert("quiet_road".to_string(), 1.15);
    m.insert("local_road".to_string(), 1.60);
    m.insert("main_road".to_string(), 2.20);
    m.insert("unknown".to_string(), 1.20);
    m
}

fn paved_first() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("paved".to_string(), 1.0);
    m.insert("path".to_string(), 1.15);
    m.insert("trail".to_string(), 1.5);
    m.insert("steps".to_string(), 6.0);
    m.insert("unknown".to_string(), 1.1);
    m
}

fn trail_first() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("paved".to_string(), 1.4);
    m.insert("path".to_string(), 1.0);
    m.insert("trail".to_string(), 0.9);
    m.insert("steps".to_string(), 3.0);
    m.insert("unknown".to_string(), 1.1);
    m
}

fn balanced() -> HashMap<String, f64> {
    let mut m = HashMap::new();
    m.insert("paved".to_string(), 1.25);
    m.insert("path".to_string(), 0.95);
    m.insert("trail".to_string(), 1.0);
    m.insert("steps".to_string(), 5.0);
    m.insert("unknown".to_string(), 1.1);
    m
}

pub fn get_profile(name: &str) -> anyhow::Result<Profile> {
    let p = match name {
        "flat" => Profile {
            name: "flat".to_string(),
            up_weight: 8.0,
            down_weight: 5.0,
            surface_weights: paved_first(),
            way_weights: pleasant_ways(),
            max_overlap: 0.20,
            repetitions: false,
            description: "Footing plat: éviter au maximum le dénivelé, revêtement roulant.".to_string(),
        },
        "rolling" => Profile {
            name: "rolling".to_string(),
            up_weight: 2.0,
            down_weight: 1.5,
            surface_weights: balanced(),
            way_weights: pleasant_ways(),
            max_overlap: 0.20,
            repetitions: false,
            description: "Vallonné modéré: petites côtes acceptées, compromis sentier/route.".to_string(),
        },
        "threshold" => Profile {
            name: "threshold".to_string(),
            up_weight: 4.0,
            down_weight: 3.0,
            surface_weights: paved_first(),
            way_weights: pleasant_ways(),
            max_overlap: 0.25,
            repetitions: false,
            description: "Seuil / tempo: terrain plat et régulier, priorité au bitume propre.".to_string(),
        },
        "hills" => Profile {
            name: "hills".to_string(),
            up_weight: 0.4,
            down_weight: 0.6,
            surface_weights: balanced(),
            way_weights: pleasant_ways(),
            max_overlap: 0.80,
            repetitions: true,
            description: "Répétitions de côtes: aller-retour explicite sur une montée.".to_string(),
        },
        "trail_drills" => Profile {
            name: "trail_drills".to_string(),
            up_weight: 1.0,
            down_weight: 1.0,
            surface_weights: trail_first(),
            way_weights: pleasant_ways(),
            max_overlap: 0.25,
            repetitions: false,
            description: "Éducatifs trail: privilégier sentiers et chemins.".to_string(),
        },
        "long_run" => Profile {
            name: "long_run".to_string(),
            up_weight: 1.8,
            down_weight: 1.2,
            surface_weights: trail_first(),
            way_weights: pleasant_ways(),
            max_overlap: 0.15,
            repetitions: false,
            description: "Sortie longue trail prioritaire: grande boucle, overlap minimal.".to_string(),
        },
        _ => anyhow::bail!("Profil inconnu: '{}'. Profils disponibles: flat, hills, long_run, rolling, threshold, trail_drills", name),
    };
    Ok(p)
}
