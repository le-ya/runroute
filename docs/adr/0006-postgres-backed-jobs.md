# ADR 0006: PostgreSQL-backed durable jobs

- Status: Accepted
- Date: 2026-09-01

## Context

Route searches outlive an HTTP request, need cancellation and recovery, and already require PostgreSQL/PostGIS for spatial business data. Adding a separate queue would create another failure domain before measured scale requires it.

## Decision

Store searches and jobs in PostgreSQL. Creating a search inserts `route_searches` and `generation_jobs` atomically. Workers claim available jobs with a short transaction using `FOR UPDATE SKIP LOCKED`, record attempt and lease metadata, commit, then perform gRPC work outside the transaction. Completion, failure, retry availability, and cancellation use conditional state transitions so stale workers cannot overwrite terminal state.

Events and candidates are written in bounded batches. Jobs have configured attempt and lease limits; process recovery reclaims expired non-terminal work. Cancellation marks durable intent and cancels an active in-process context when present. Database errors are surfaced truthfully rather than converted to successful empty results.

Use PostGIS only for application geometry such as candidate `LineStringZ` and later restrictions. Use GiST for spatial access and partial/B-tree indexes for available jobs and status/time queries. Do not store GraphHopper's graph or query per route point.

## Consequences

Search state, queue state, and business records share transactional semantics and backup/recovery. Throughput is bounded by PostgreSQL but avoids Redis/Kafka/RabbitMQ until measurements demonstrate a real bottleneck.
