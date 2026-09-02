# ADR 0002: TypeScript, Go, and Rust boundaries

- Status: Accepted
- Date: 2026-09-01

## Context

The product combines browser interaction, durable job orchestration, spatial persistence, provider I/O, and CPU-heavy deterministic optimization. One language can implement all of it, but would either move heavy search into the browser or weaken the existing Svelte application and planned optimizer.

## Decision

TypeScript/Svelte owns browser forms, map interaction, visualization, comparison, GPX adaptation, and existing editing/export. Go is the only public backend and owns REST validation, problem responses, PostgreSQL transactions, jobs, persistence, history, SSE, and service orchestration. Rust owns provider normalization, deterministic generation, geometry/elevation processing, constraints, scoring, overlap, diversity, and bounded CPU work.

Go and Rust communicate exclusively through generated protobuf gRPC bindings. Rust owns no accounts or business database. Go implements no optimizer. Heavy optimization never runs in the browser.

## Consequences

Responsibilities are explicit and independently testable. Cross-service changes require a contract update and regenerated bindings. Some deployment complexity is accepted to keep public API/business state separate from the performance-sensitive engine.
