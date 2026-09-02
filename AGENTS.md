# Repository Guidelines

## Architecture and project structure

This repository preserves gpx.studio at upstream commit `6a0d4343718e01637a8e301251977fb218cf88f8` and extends it through vertical route-generation slices. Read `docs/ARCHITECTURE.md`, `docs/UPSTREAM.md`, and the accepted records in `docs/adr/` before changing boundaries.

- `gpx/`: existing TypeScript GPX parser, model, statistics, and serializer.
- `website/`: existing Svelte 5/SvelteKit 2 editor, MapLibre map, Dexie persistence, and export UI.
- `services/api/`: Go public REST API, PostgreSQL jobs, persistence, SSE, and Rust orchestration.
- `services/route-engine/`: Rust gRPC optimizer, provider adapters, analysis, scoring, and diversity.
- `proto/`: the single versioned Go–Rust protobuf contract and Buf configuration.
- `migrations/`: ordered PostgreSQL/PostGIS schema changes.
- `deploy/`: GraphHopper, website proxy, and runtime configuration.
- `data/`: ignored local source/routing/elevation data; only small versioned fixtures may be committed.
- `docs/`: architecture, ADRs, baseline records, and operator/developer documentation.

Do not introduce a second GPX parser, editor, serializer, frontend framework, browser state store, public backend, or handwritten duplicate RPC DTO. Candidate previews remain transient GeoJSON; selection enters the existing `GPXFile` model through its action manager.

## Commands and toolchains

Use the repository `Makefile` command surface when present:

- `make proto` — lint contracts and regenerate checked Go/Rust bindings.
- `make build` — build all language components.
- `make test` — run unit and contract tests.
- `make lint` — run non-mutating formatters, linters, and static checks.
- `make data OSM_PBF_URL=<explicit-url>` — download/version source data explicitly.
- `make build-routing` — build prepared GraphHopper routing data explicitly.
- `make dev` — start the prepared Compose stack.

Until Phase 1 adds those targets, use:

- `npm ci --prefix gpx && npm run build --prefix gpx`
- `npm ci --prefix website && npm run check --prefix website`
- `npm run dev --prefix website`

The imported baseline failures and environment constraints are recorded in `docs/BASELINE.md`. Keep them visible; never obtain a green result by weakening strictness, excluding source broadly, or adding placeholder commands. Normal service startup must not download OSM/DEM data or rebuild a GraphHopper graph.

Pinned prerequisites are Node/npm lockfiles, Go modules, Cargo lockfile, Buf/protobuf tooling, Docker Compose, and PostgreSQL/PostGIS migrations. Update the relevant lock/configuration and this command list together when the tool surface changes.

## Style and boundaries

Use repository Prettier/ESLint settings for TypeScript and Svelte, `gofmt` plus the configured Go linter, `rustfmt` plus Clippy for Rust, `buf lint` for protobuf, and the configured formatter/linter for SQL and shell. Use spaces unless a formatter requires otherwise.

Prefer descriptive `camelCase` TypeScript functions, `PascalCase` TypeScript types/components, idiomatic Go exported names, and idiomatic Rust `snake_case` functions/modules with `PascalCase` types. Keep modules focused and avoid unrelated upstream formatting.

Respect ownership:

- Browser: interaction, visualization, GPX adaptation; no heavy optimization.
- Go: public HTTP, validation, durable jobs/persistence, SSE; no optimizer.
- Rust: deterministic optimization and provider normalization; no public HTTP, accounts, or business persistence.
- GraphHopper: access, snapping, low-level routing; its graph is not copied to PostGIS.

All work must be bounded and cancellable. Do not log precise personal coordinates. Access, continuity, privacy, closures, and safety are hard constraints and cannot be degraded.

## Testing

Every behavior change needs a test at the owning boundary. Name tests after observable behavior and cover normal paths, limits, cancellation, and expected failures.

- `gpx/` and `website/`: unit/component tests for conversion, validation, layers, selection, stale analysis, and export round-trip.
- Go: HTTP validation/problem responses, transactions/jobs, gRPC mapping, cancellation, SSE resume, and privacy-safe errors/logging.
- Rust: deterministic anchors, geometry/elevation, constraints, scoring, overlap/diversity, and degradation.
- Contracts: Buf lint, reproducible generation, and breaking checks.
- Integration: small versioned OSM/DEM fixtures through real GraphHopper → Rust → Go.
- Browser: Playwright generate/compare/select/edit/analyze/export flows.

Golden route tests assert ranges and invariants, not exact geometry. Mocks are test-only. Record benchmark inputs, versions, stage timings, budgets, route quality, CPU, and memory before changing performance architecture.

## Data, migrations, and operations

Prepared PBF, DEM, routing graph, and database data live in ignored persistent volumes. Commit checksums/manifests and small fixtures, not generated regional datasets.

Migrations are immutable after release and apply in order. Batch event/candidate writes; never query per route point. Health endpoints must probe and report each real dependency and dataset identity—never hard-code success.

## Commits and pull requests

Use short imperative subjects such as `Add route health contract` or `Reject inaccessible candidates`. Keep commits scoped to one vertical behavior. Pull requests explain the problem and solution, list exact verification, link issues/ADRs, call out configuration or migration changes, and include screenshots for visible changes.

Changes within `gpx/` or `website/` must remain easy to reconcile with upstream. Update `docs/UPSTREAM.md` when advancing the preserved base and add an ADR when replacing an accepted architectural decision.
