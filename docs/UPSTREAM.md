# Upstream provenance

The preserved application base is [`gpxstudio/gpx.studio`](https://github.com/gpxstudio/gpx.studio) `main` at commit `6a0d4343718e01637a8e301251977fb218cf88f8`.

The snapshot was imported on 2026-09-01. The existing `gpx/` and `website/` directory layout, root MIT `LICENSE`, GPX model, SvelteKit editor, and upstream assets are retained. Trail-route services must be added beside these directories rather than by rewriting the upstream application.

This workspace currently has no `.git` metadata, so the commit identity above is the authoritative local provenance record. When a writable checkout is available:

```bash
git remote add upstream https://github.com/gpxstudio/gpx.studio.git
git fetch upstream
git merge-base --is-ancestor 6a0d4343718e01637a8e301251977fb218cf88f8 upstream/main
```

Upstream synchronization rules:

1. Fetch and inspect upstream changes before merging them.
2. Keep changes to `gpx/` and `website/` narrow; do not apply repository-wide formatting during feature work.
3. Keep new backend, contract, database, and data-preparation code outside the preserved directories.
4. Resolve upstream conflicts in favor of retaining the existing GPX parser/editor/action-manager contracts unless a deliberate ADR replaces them.
5. Record the new upstream commit here whenever the base is advanced.
