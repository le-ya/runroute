# ADR 0005: Versioned canonical server elevation

- Status: Accepted
- Date: 2026-09-01

## Context

Browser statistics currently use Mapterhorn elevation plus simplification/smoothing. Route search, edited-route analysis, and target-ascent guarantees require one reproducible definition of D+/D- while retaining enough data to explain differences from watches or other providers.

## Decision

Rust owns canonical elevation analysis behind an `ElevationProvider`. The initial provider may use elevations returned from GraphHopper/prepared Mapterhorn data; later local DEM/COG providers implement the same contract. Optimization performs no external API call per point.

The versioned pipeline densifies geometry in the distance domain, interpolates provider elevations, retains raw samples, applies a median filter and weighted smoothing, then applies minimum meaningful vertical-change hysteresis before summing ascent/descent. Initial benchmark values are 10 m sampling, a 30 m smoothing window, and 3 m hysteresis; they remain configuration pending representative fixtures.

Every analysis persists provider, dataset, algorithm, and parameter versions plus raw and smoothed ascent/descent and altitude range. Shared golden fixtures assert reproducibility and acceptable ranges. After manual edits the UI marks server metrics stale until `/api/v1/routes/analyze` returns for the current geometry revision.

## Consequences

Displayed target compliance is consistent with optimization. Raw samples and versions make disagreements diagnosable. Browser-only metrics may remain available for immediate feedback but cannot claim canonical target compliance after Phase 4.
