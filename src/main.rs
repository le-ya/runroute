use anyhow::Result;
use clap::{Parser, Subcommand};
use runroute::config::{load_config, Config};
use runroute::geo::wgs_to_l93;
use runroute::graph::Graph;
use runroute::multicriteria::{search_anchor_routes, CompactGraph};
use runroute::profiles::get_profile;
use runroute::{anchors, export};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Parser)]
#[command(name = "runroute")]
#[command(about = "High-performance trail route generator for Lyon & Monts d'Or in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Génère un itinéraire de course à pied ou trail
    Route {
        /// Point de départ (nom dans config ou lat,lon)
        #[arg(long, default_value = "home")]
        start: String,

        /// Profil sportif (flat, hills, rolling, threshold, trail_drills, long_run)
        #[arg(long, default_value = "trail_drills")]
        profile: String,

        /// Distance cible en kilomètres
        #[arg(long, default_value_t = 12.0)]
        distance: f64,

        /// Dénivelé positif cible en mètres
        #[arg(long)]
        dplus: Option<f64>,

        /// Mode de génération: natural (sans répétitions artificielles) ou vertical (avec répétitions)
        #[arg(long, default_value = "natural")]
        route_mode: String,

        /// Nom ou libellé de la trace GPX générée
        #[arg(long)]
        name: Option<String>,

        /// Graine aléatoire déterministe
        #[arg(long)]
        seed: Option<u64>,

        /// Répertoire des données
        #[arg(long)]
        data_dir: Option<PathBuf>,

        /// Répertoire d'export GPX
        #[arg(long)]
        output_dir: Option<PathBuf>,

        /// Mode verbeux
        #[arg(short, long)]
        verbose: bool,
    },

    /// Convertit explicitement graph.graphml en graph.bin pour un chargement instantané
    Convert {
        #[arg(long, default_value = "data")]
        data_dir: PathBuf,
    },
}

fn resolve_start_node(
    graph: &Graph,
    config: &Config,
    name: &str,
) -> Result<(runroute::graph::types::NodeIndex, String)> {
    if let Some(&(lat, lon)) = config.points.get(name) {
        let (x, y) = wgs_to_l93(lat, lon);
        let node = graph.nearest_node(x, y);
        return Ok((node, name.to_string()));
    }

    // Try parsing lat,lon
    if let Some((lat_s, lon_s)) = name.split_once(',') {
        if let (Ok(lat), Ok(lon)) = (lat_s.trim().parse::<f64>(), lon_s.trim().parse::<f64>()) {
            let (x, y) = wgs_to_l93(lat, lon);
            let node = graph.nearest_node(x, y);
            return Ok((node, "custom".to_string()));
        }
    }

    anyhow::bail!(
        "Point inconnu: '{}'. Points configurés: {:?}",
        name,
        config.points.keys().collect::<Vec<_>>()
    )
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Convert { data_dir } => {
            let graphml_path = data_dir.join("graph.graphml");
            let bin_path = data_dir.join("graph.bin");
            println!("Conversion de {:?} vers {:?}...", graphml_path, bin_path);
            let t0 = Instant::now();
            let parsed = runroute::graph::graphml::parse_graphml(&graphml_path)?;
            println!("GraphML parsé en {:.2}s", t0.elapsed().as_secs_f64());
            runroute::graph::binary::save_binary(&bin_path, &parsed.nodes, &parsed.osm_to_idx, &parsed.edges)?;
            println!("Sauvegarde terminée en {:.2}s !", t0.elapsed().as_secs_f64());
        }

        Commands::Route {
            start,
            profile,
            distance,
            dplus,
            route_mode,
            name,
            seed,
            data_dir,
            output_dir,
            verbose,
        } => {
            let t_start = Instant::now();
            let cfg = load_config(None::<&str>)?;
            let effective_data_dir = data_dir.unwrap_or_else(|| {
                if Path::new("data").exists() {
                    PathBuf::from("data")
                } else if Path::new("../runroute/data").exists() {
                    PathBuf::from("../runroute/data")
                } else {
                    cfg.data_dir.clone()
                }
            });
            let effective_output_dir = output_dir.unwrap_or_else(|| {
                if Path::new("gpx").exists() {
                    PathBuf::from("gpx")
                } else if Path::new("../runroute/gpx").exists() {
                    PathBuf::from("../runroute/gpx")
                } else {
                    cfg.output_dir.clone()
                }
            });

            if verbose {
                eprintln!("[{:.2}s] Chargement du graphe depuis {:?}...", t_start.elapsed().as_secs_f64(), effective_data_dir);
            }

            let graph = Graph::load_or_convert(&effective_data_dir)?;
            if verbose {
                eprintln!(
                    "[{:.2}s] Graphe prêt: {} nœuds, {} arêtes",
                    t_start.elapsed().as_secs_f64(),
                    graph.node_count(),
                    graph.edge_count()
                );
            }

            let prof = get_profile(&profile)?;
            let target_m = distance * 1000.0;

            let (start_idx, start_name) = resolve_start_node(&graph, &cfg, &start)?;

            // Resolve destinations
            let mut destinations = Vec::new();
            let mut endpoint_names = hashbrown::HashMap::new();

            if let Some(&(h_lat, h_lon)) = cfg.points.get("home") {
                let (hx, hy) = wgs_to_l93(h_lat, h_lon);
                let home_node = graph.nearest_node(hx, hy);
                destinations.push(home_node);
                endpoint_names.insert(home_node, "home".to_string());
            }

            if destinations.is_empty() {
                destinations.push(start_idx);
                endpoint_names.insert(start_idx, start_name.clone());
            }

            let mut mandatory = vec![start_idx];
            for &d in &destinations {
                if !mandatory.contains(&d) {
                    mandatory.push(d);
                }
            }

            if verbose {
                eprintln!(
                    "[{:.2}s] Sélection des ancres pour cible {:.1}km / D+ {}m...",
                    t_start.elapsed().as_secs_f64(),
                    distance,
                    dplus.map(|d| format!("{:.0}", d)).unwrap_or_else(|| "auto".to_string())
                );
            }

            let anchors = anchors::select_anchor_nodes(
                &graph,
                &mandatory,
                target_m,
                48,
                300.0,
                dplus,
            );

            if verbose {
                eprintln!(
                    "[{:.2}s] Construction du graphe compact ({}) ancres en parallèle...",
                    t_start.elapsed().as_secs_f64(),
                    anchors.len()
                );
            }

            let compact = CompactGraph::build(
                &graph,
                &anchors,
                &prof,
                target_m,
                dplus,
                cfg.anchor_neighbor_count,
                &mandatory,
            );

            if verbose {
                eprintln!(
                    "[{:.2}s] Graphe compact: {} segments précalculés. Lancement du beam search...",
                    t_start.elapsed().as_secs_f64(),
                    compact.number_of_edges()
                );
            }

            let candidates = search_anchor_routes(
                &graph,
                &compact,
                start_idx,
                &destinations,
                &endpoint_names,
                &start_name,
                target_m,
                dplus,
                &prof,
                &route_mode,
                &cfg,
            );

            let Some(best_route) = candidates.first() else {
                anyhow::bail!("Aucun itinéraire conforme trouvé pour les critères demandés.");
            };

            let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
            let label = name.unwrap_or_else(|| format!("{}_{:.0}km", profile, distance));
            let file_prefix = format!("{}_{}", date_str, label);

            let gpx_file = effective_output_dir.join(format!("{}.gpx", file_prefix));
            let json_file = effective_output_dir.join(format!("{}.json", file_prefix));
            let geojson_file = effective_output_dir.join(format!("{}.alternatives.geojson", file_prefix));

            export::export_gpx(&graph, best_route, &gpx_file, &format!("{} {:.1} km", profile, best_route.distance_m / 1000.0))?;
            export::export_report(
                best_route,
                &candidates,
                target_m,
                dplus,
                &profile,
                &route_mode,
                seed.unwrap_or(42),
                &json_file,
            )?;
            export::export_alternatives_geojson(&graph, &candidates, &geojson_file)?;

            let status_icon = if best_route.compliant { "✔" } else { "⚠" };
            println!(
                "{} {}: {:.2} km, D+ {:.0} m, D- {:.0} m, overlap {:.0}%",
                status_icon,
                profile,
                best_route.distance_m / 1000.0,
                best_route.dplus_m,
                best_route.dminus_m,
                best_route.overlap_ratio * 100.0
            );
            println!("  départ: {} -> arrivée: {}", best_route.start_name, best_route.end_name);
            println!("  GPX: {:?}", gpx_file);
            println!("  rapport: {:?}", json_file);
            println!("  comparaison: {:?}", geojson_file);
            println!("  temps d'exécution total: {:.2}s", t_start.elapsed().as_secs_f64());
        }
    }

    Ok(())
}
