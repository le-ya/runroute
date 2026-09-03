# runroute (v2 - Rust)

> Générateur d'itinéraires de course à pied et trail haute performance pour Lyon et les Monts d'Or en **Rust**.

Version réécrite de `runroute` offrant un gain de vitesse de **50x à 60x** par rapport à l'implémentation Python historique (génération en **~1.5 à 2 secondes** contre 80–90 secondes).

---

## Fonctionnalités

- ⚡ **Ultra-rapide** : chargement du graphe (158 704 nœuds, 455 003 arêtes) en **< 200 ms** via cache binaire (`graph.bin`), calcul Dijkstra multi-threadé avec **Rayon**, beam search instantané.
- ⛰️ **Multi-critères Altimétrique & Sentiers** : optimisation simultanée de la distance cible, du dénivelé ($D^+$, $D^-$), de la part de sentiers/chemins (profils *trail-first*), et minimisation stricte du recouvrement (*overlap*).
- 📍 **Points d'intérêt & Gares/Stations** : support des départs/arrivées prédéfinis (`home`, `carret_sedallian`, `couzon`, `ile_barbe`) ou coordonnées GPS libres `lat,lon`.
- 📊 **Parité complète des exports** :
  - Trace **GPX 1.1** enrichie (waypoints de départ/arrivée, altitudes lissées, compatible montres COROS/Garmin/Strava).
  - Rapport complet **JSON (schéma 2.3)** avec distribution des revêtements (*trail*, *path*, *paved*, *steps*) et types de voies (*path*, *pedestrian*, *quiet_road*, etc.).
  - Comparatif **GeoJSON** des alternatives générées.

---

## Profils sportifs disponibles

| Profil | Poids D+ | Poids D- | Revêtement prioritaire | Recouvrement max |
| :--- | :---: | :---: | :--- | :---: |
| `trail_drills` | 1.0 | 1.0 | Sentiers & chemins (*trail-first*) | 25% |
| `long_run` | 1.8 | 1.2 | Sentiers & chemins | 15% |
| `rolling` | 2.0 | 1.5 | Équilibré sentier / bitume | 20% |
| `flat` | 8.0 | 5.0 | Bitume roulant (*paved-first*) | 20% |
| `threshold` | 4.0 | 3.0 | Bitume roulant | 25% |
| `hills` | 0.4 | 0.6 | Répétitions autorisées | 80% |

---

## Installation & Compilation

Nécessite Rust 1.82+ (recommandé 1.85+) :

```bash
cargo build --release
```

Le binaire optimisé est produit dans `./target/release/runroute`.

---

## Utilisation

### Générer un parcours trail 21 km / 900 m D+ depuis la station Vélo'v Carret / Sédallian :
```bash
./target/release/runroute route \
  --start carret_sedallian \
  --profile trail_drills \
  --distance 21 \
  --dplus 900 \
  -v
```

### Autres exemples :
```bash
# Sortie longue 25 km depuis Couzon avec D+ libre
./target/release/runroute route --start couzon --profile long_run --distance 25

# Boucle vallonnée 15 km depuis le domicile
./target/release/runroute route --start home --profile rolling --distance 15 --dplus 450
```

### Conversion explicite du graphe :
```bash
./target/release/runroute convert --data-dir data
```
*(Remarque : la conversion se fait automatiquement au premier lancement si `graph.bin` n'existe pas).*

---

## Tests

```bash
cargo test
```
