# ADR 0007: Deterministic constraints, scoring, and diversity

- Status: Accepted
- Date: 2026-09-01

## Context

A useful training route is not merely the shortest path. Results must satisfy non-negotiable access/safety rules, explain target tradeoffs, produce genuinely different candidates, and reproduce under controlled inputs.

## Decision

Separate hard constraints from soft scoring. Pedestrian access, private/forbidden routing, continuity, active closures, privacy, safety, and configured absolute limits reject a route and are never relaxed. Degraded mode may relax only requested distance/ascent tolerances and must return one warning per violated target.

Normalize each soft component to an integer 0–100 score: distance, ascent, surface, trail share, safety, self-overlap, continuity, intersections, training-profile suitability, and geometry. Versioned profile configuration supplies integer weights; accumulate with fixed ordering and integer/fixed-point arithmetic, then apply a documented deterministic tie-break order. Return every component, weight, total, and version.

Derive anchor ordering solely from the explicit seed and versioned search configuration. Sort provider outputs into canonical order before parallel analysis can affect ranking. Persist dataset, provider, elevation, scoring, and configuration versions with results.

Podium diversity uses shared stable routed-segment distance divided by the shorter route distance. Initial limits are 60% for Natural and 75% for Vertical. Relax only through versioned documented increments when three candidates are otherwise impossible, and emit the final threshold as a warning.

## Consequences

Equal request, seed, dataset, algorithms, and configuration reproduce the same ordered podium regardless of task completion order. Scores remain explainable. Floating-point geometry/provider behavior still requires platform-aware golden ranges, but cannot silently decide ties.
