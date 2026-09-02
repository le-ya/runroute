#!/bin/sh
set -eu

: "${OSM_PBF_URL:?OSM_PBF_URL is required}"
: "${OSM_PBF_SHA256:?OSM_PBF_SHA256 is required}"

repository_root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
data_directory="$repository_root/data"
temporary_file="$data_directory/region.osm.pbf.part"
output_file="$data_directory/region.osm.pbf"
manifest_file="$data_directory/manifest.env"

mkdir -p "$data_directory"
trap 'rm -f "$temporary_file"' EXIT INT TERM
curl --fail --location --output "$temporary_file" "$OSM_PBF_URL"
printf '%s  %s\n' "$OSM_PBF_SHA256" "$temporary_file" | sha256sum --check --strict
mv "$temporary_file" "$output_file"
trap - EXIT INT TERM

{
    printf 'OSM_PBF_URL=%s\n' "$OSM_PBF_URL"
    printf 'OSM_PBF_SHA256=%s\n' "$OSM_PBF_SHA256"
} > "$manifest_file"

printf 'Prepared %s using checksum %s\n' "$output_file" "$OSM_PBF_SHA256"
