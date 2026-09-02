#!/bin/sh
set -eu

command_name="${1:-server}"
case "$command_name" in
    import)
        if [ ! -s /data/source/region.osm.pbf ]; then
            echo "missing prepared OSM input: /data/source/region.osm.pbf" >&2
            exit 1
        fi
        exec java ${JAVA_OPTS:--Xms1g -Xmx4g} -jar /app/graphhopper.jar import /app/config.yml
        ;;
    server)
        if [ ! -s /data/graph-cache/properties ]; then
            echo "missing prepared GraphHopper graph; run make build-routing" >&2
            exit 1
        fi
        exec java ${JAVA_OPTS:--Xms1g -Xmx4g} -jar /app/graphhopper.jar server /app/config.yml
        ;;
    *)
        echo "unsupported GraphHopper command: $command_name" >&2
        exit 2
        ;;
esac
