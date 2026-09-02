# ADR 0004: GraphHopper-first routing provider

- Status: Accepted
- Date: 2026-09-01

## Context

The preserved frontend already uses GraphHopper for foot and bike routing and requests useful path details. The optimizer needs pedestrian access, snapping, geometry, surfaces, trail classifications, and stable segment identity without maintaining a second routing graph.

## Decision

Use a pinned local GraphHopper build as the first low-level routing provider. Rust accesses it through a narrow `RoutingProvider` abstraction and normalizes all responses at that boundary. Requests use a shared HTTP client, bounded concurrency, deadlines, cancellation, and explicit path-detail capability probes.

Prefer GraphHopper `edge_id` details only after the pinned build proves their presence and stability; namespace identities by dataset version. Otherwise derive tested direction-independent hashes from stable provider/OSM metadata. The GraphHopper graph remains outside PostGIS.

Prepared PBF, elevation inputs, graph, profile/configuration, and dataset identity are persistent versioned data. Data download and graph construction are explicit commands; application startup validates prepared data and never downloads or rebuilds it.

## Consequences

Existing manual routing and optimizer behavior share access semantics. Provider JSON cannot leak into core scoring. Changing providers remains possible after comparative quality and operational benchmarks, but is not preemptively generalized beyond the narrow interface.
