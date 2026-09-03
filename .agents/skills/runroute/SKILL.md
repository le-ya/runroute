---
name: runroute
description: Autonomous operations, route generation, and configuration guide for runroute (v2 Rust), a high-performance trail running route generator. Use when generating, analyzing, benchmarking, or configuring running and trail routes in Lyon and Monts d'Or.
---

# `runroute` — AI Agent Skill & Autonomous Operations Manual

## 1. Overview & Purpose

`runroute` is a high-performance multi-criteria trail running route generator written in **Rust**.
It plans natural running loops and point-to-point routes across complex topography (e.g., Monts d'Or & Lyon metropolitan area) by optimizing simultaneously:
- Target distance ($km \pm 5\%$)
- Target vertical elevation gain and loss ($D^+$, $D^-$)
- Trail vs. road surface ratio (*trail-first* vs. *paved-first*)
- Strict non-overlap constraints (preventing artificial U-turns and dead-ends)
- Landmark snapping (Vélo'v stations, transit hubs, home coordinates)

### Speed Benchmark
- **Graph size**: 158,704 nodes, 455,003 edges.
- **Execution time**: **1.3 to 2.3 seconds** total (compared to 75–90 seconds in legacy Python engines).
- **Graph loading**: $< 200\text{ ms}$ via memory-efficient binary cache (`data/graph.bin`).

---

## 2. Fast-Start Execution Protocol

Any AI agent interacting with this repository should run the following commands:

### Build Release Binary
```bash
cargo build --release
```
The optimized binary is located at `./target/release/runroute`.

### Primary Route Generation Command
```bash
./target/release/runroute route \
  --start <START_POINT> \
  --profile <PROFILE> \
  --distance <DISTANCE_KM> \
  [--dplus <DPLUS_METERS>] \
  [--route-mode <natural|vertical>] \
  [--name <OUTPUT_LABEL>] \
  [-v]
```

### Quick Verification Test
```bash
./target/release/runroute route --start carret_sedallian --profile trail_drills --distance 21 --dplus 900 -v
```
Expected output:
- Generates a route in ~1.5–2.3 seconds.
- Exports `.gpx`, `.json`, and `.alternatives.geojson` into the `gpx/` directory.
- Verifies `compliant: true`, `overlap <= 10%`, `dead_ends: 0`, `u_turns: 0`.

---

## 3. Profiles & Parameters Matrix

| Profile | Up Weight | Down Weight | Surface Preference | Max Overlap | Best For |
| :--- | :---: | :---: | :--- | :---: | :--- |
| `trail_drills` | 1.0 | 1.0 | **Trail First** (singletrack, dirt path) | 25% | Technical trail workouts, ridge traverses |
| `long_run` | 1.8 | 1.2 | **Trail First** (large scenic loop) | 15% | Long endurance runs, minimal repetition |
| `rolling` | 2.0 | 1.5 | **Balanced** (trails & quiet roads) | 20% | Moderate hilly training |
| `flat` | 8.0 | 5.0 | **Paved First** (asphalt, riverbanks) | 20% | Recovery runs, flat pace maintenance |
| `threshold` | 4.0 | 3.0 | **Paved First** (regular gradient) | 25% | Tempo & threshold intervals |
| `hills` | 0.4 | 0.6 | **Balanced** (hill repeats allowed) | 80% | Short hill repeats / vertical drills |

### CLI Options

- `--start <ID_OR_COORDS>`: Start anchor name from `runroute.toml` (`home`, `carret_sedallian`, `couzon`, `ile_barbe`) OR raw WGS84 `latitude,longitude` (e.g. `45.7924,4.8205`).
- `--profile <NAME>`: One of the 6 profiles above (default: `trail_drills`).
- `--distance <KM>`: Target distance in kilometers (e.g. `21.0`). Target tolerance window is $\pm 5\%$.
- `--dplus <METERS>`: Desired vertical elevation gain in meters.
  - If omitted: automatically balanced by profile.
  - To **maximize $D^+$**: pass a high target (e.g. `--dplus 1100`). The engine rewards steep trail ascents and ranks higher-gain routes first.
- `--route-mode <natural|vertical>`:
  - `natural` (default): True trail loop or point-to-point with 0 immediate U-turns and 0 dead-ends.
  - `vertical`: Allows climb repeats on steep slopes for focused uphill conditioning.
- `--name <STRING>`: Custom prefix for output filenames.
- `--data-dir <PATH>`: Path to graph data directory (default: `data/`).
- `--output-dir <PATH>`: Output directory for generated GPX and reports (default: `gpx/`).
- `-v, --verbose`: Prints timestamped operational milestones.

---

## 4. Standard Operating Procedures (SOPs)

### SOP 1: Generating a Workout for an Athlete
1. Check athlete requirements:
   - Starting point (e.g., station Vélo'v n°9041 `carret_sedallian`, train station `couzon`, or `home`).
   - Distance (e.g. 21 km) and target $D^+$ (e.g. 900 m).
   - Sport modality (trail, recovery, hills).
2. Execute the CLI command with `--start`, `--profile`, `--distance`, `--dplus`, and `-v`.
3. Inspect the console summary:
   - Ensure the route is marked `✔` (compliant).
   - Check `distance_km`, `dplus_m`, and `overlap`.
4. Inspect the generated JSON report in `gpx/`:
   - Verify `"compliant": true`.
   - Check `"surface_ratios"`: verify trail/path ratio $\ge 60\%$ for trail profiles.

### SOP 2: Maximizing Vertical Gain on Fixed Distance
When the user requests maximum possible $D^+$ on a given distance (e.g. 21 km):
1. Target profile `trail_drills`.
2. Set `--dplus` to an ambitious threshold (e.g. `1050` or `1100`).
3. The engine activates high-density heuristics:
   - Dynamic 14-hop exploration traversing multiple Monts d'Or peaks (Mont Cindre, Mont Thou, Mont Verdun, Croix Rampau).
   - Elevation advance bonus in beam search frontier.
   - Highest compliant $D^+$ ranked as Rank 1.
4. Report both:
   - **D+ RGE ALTI lissé (gpx.studio)**: standard benchmark.
   - **D+ brut**: expected readout on watch GPS (COROS, Garmin, Strava).

### SOP 3: Adding or Modifying Landmarks / Points
To register a new landmark or home address:
1. Open [`runroute.toml`](file:///home/yannick/runrouteV2/runroute.toml).
2. Under `[points]`, add `name = [latitude, longitude]`:
   ```toml
   [points]
   home = [45.786902, 4.7896771]
   new_spot = [45.812345, 4.823456]
   ```
3. The coordinate is automatically projected into Lambert-93 and snapped to the nearest graph node at runtime.

### SOP 4: Regenerating the Binary Graph Cache
If `data/graph.graphml` is updated or replaced:
```bash
./target/release/runroute convert --data-dir data
```
- Reads XML in ~3.1 seconds.
- Writes optimized binary cache `data/graph.bin` in ~3.3 seconds.
- All subsequent runs resume sub-second startup (<200ms).

---

## 5. Output Schemas & Diagnostics

Every execution generates three paired files in `output_dir` (default: `gpx/`):

### 1. `YYYY-MM-DD_<name>.gpx`
Standard GPX 1.1 file containing:
- `<metadata>` with route summary and distance/elevation.
- `<wpt>` waypoints for Departure (`Départ — <start>`) and Arrival (`Arrivée — <end>`).
- `<trk>` track points with latitude, longitude, and RGE ALTI smoothed elevation `<ele>`.
- Fully compatible with COROS, Garmin Connect, Suunto, Strava, and gpx.studio.

### 2. `YYYY-MM-DD_<name>.json`
Structured analytical report complying with **Schema 2.3**:
- `route`: Key metrics (`distance_km`, `dplus_m`, `raw_dplus_m`, `dminus_m`, `overlap_ratio`, `score`, `compliant`).
- `surface_distances_m` & `surface_ratios`: Distance and percentage on `trail`, `path`, `paved`, `steps`.
- `way_distances_m` & `way_ratios`: Distance and percentage on `path`, `pedestrian`, `quiet_road`, `local_road`, `main_road`.
- `candidates`: Top evaluated alternative candidates for comparative inspection.

### 3. `YYYY-MM-DD_<name>.alternatives.geojson`
Standard GeoJSON `FeatureCollection` showing the top candidate trajectories with distinct styling and colors (`#2563eb`, `#10b981`, etc.) for visual inspection on Leaflet, Mapbox, or QGIS.

---

## 6. Verification Checklist for Agents

Before presenting a generated route to the athlete/user, verify:
- [ ] Distance error is within $\pm 5\%$ of requested distance.
- [ ] $D^+$ is within target tolerance (or maximized if requested).
- [ ] Overlap ratio is strictly within profile allowance ($\le 25\%$ for trail drills, $\le 15\%$ for long run, $\approx 0-8\%$ achieved).
- [ ] Departure matches requested location; Arrival terminates at specified destination (or `home`).
- [ ] GPX and JSON files exist on disk and are non-empty.
