# ADR 0003: gRPC internally and resumable SSE publicly

- Status: Accepted
- Date: 2026-09-01

## Context

Search produces ordered progress and candidate events, must propagate cancellation, and later supports live visualization and replay. Browsers consume unidirectional updates; Go and Rust need typed streaming and deadlines.

## Decision

Define one versioned protobuf package with unary `Check`, server-streaming `GenerateRoutes`, and unary `AnalyzeRoute`. Generated Go and Rust messages are the sole internal RPC DTOs. Every search event contains its search ID, monotonic sequence, occurrence time, and a typed `oneof` payload. Cancellation and deadlines flow through the gRPC context to provider work.

Go persists meaningful events before exposing them over `GET /api/v1/searches/{id}/events`. The endpoint uses SSE. `Last-Event-ID` resumes strictly after the durable sequence, terminal events close the stream, and heartbeat comments keep idle connections alive without consuming sequence numbers. Go bounds subscriber buffers; slow clients reconnect from durable history rather than blocking the worker.

## Consequences

Browser support stays simple and HTTP-friendly. Durable ordering supports resume and replay. Bidirectional browser control is unnecessary; cancellation remains an ordinary `DELETE`. Event schemas and retention require deliberate versioning and compact payloads.
