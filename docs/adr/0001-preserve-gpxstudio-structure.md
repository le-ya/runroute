# ADR 0001: Preserve gpx.studio structure and model

- Status: Accepted
- Date: 2026-09-01

## Context

gpx.studio already supplies the GPX parser/model, SvelteKit editor, MapLibre rendering, routing anchors, undo/redo, Dexie persistence, elevation display, and export. Replacing any of these creates duplicate state and makes upstream synchronization costly.

## Decision

Keep `gpx/` and `website/` in their upstream layout. New services and contracts live beside them. Candidate previews are bounded transient GeoJSON, not GPX files. Selecting a candidate creates an ordinary `GPXFile` using the existing action manager; all later edits, persistence, undo/redo, and export follow upstream paths.

Changes inside preserved directories must be narrow and must not introduce a second parser, serializer, editor, framework, or state store. Repository-wide formatting is prohibited during feature work.

## Consequences

Frontend integration must adapt backend results to current GPX/action contracts. Upstream merges remain tractable. Candidate-specific state disappears safely when comparison ends and cannot pollute persistent files.
