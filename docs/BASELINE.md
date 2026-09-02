# Preserved upstream baseline

Status recorded on 2026-09-01 for upstream commit `6a0d4343718e01637a8e301251977fb218cf88f8`.

## Supported local commands

Dependencies are pinned by `gpx/package-lock.json` and `website/package-lock.json`; use `npm ci`, not `npm install`, for a reproducible install.

| Area                      | Command                                                                                                                                     | Baseline result                                                                                                                                      |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| GPX build                 | `npm run build --prefix gpx`                                                                                                                | Passes                                                                                                                                               |
| GPX formatting            | `npm exec --prefix gpx prettier -- --check . --config ../.prettierrc`                                                                       | Passes                                                                                                                                               |
| GPX lint                  | `npm run lint --prefix gpx`                                                                                                                 | Blocked by the upstream ESLint 9 configuration mismatch: no flat `eslint.config.*` exists                                                            |
| Website dependencies      | `npm ci --prefix website`                                                                                                                   | Passes                                                                                                                                               |
| Website type/Svelte check | `npm run check --prefix website`                                                                                                            | Upstream baseline: 25 errors and 12 warnings, including missing environment and third-party declarations plus stricter Svelte/TypeScript diagnostics |
| Website formatting        | `npm exec --prefix website prettier -- --check . --config ../.prettierrc --ignore-path ../.prettierignore --ignore-path website/.gitignore` | Upstream differences in `website/components.json` and `website/src/app.css`                                                                          |
| Website build             | `npm run build --prefix website`                                                                                                            | Environment-blocked: `tsx` cannot create its IPC socket under `/tmp` in this sandbox                                                                 |

These failures are isolated to the imported snapshot or execution environment. New code must not add diagnostics. A later baseline-repair change may make the checks green, but it must be narrow and must not hide pre-existing errors with broad exclusions, relaxed compiler settings, or disabled lint rules.

## Environment

Observed tools:

- Node.js 22 and npm 10
- Go 1.22

Unavailable at baseline:

- Rust/Cargo
- Docker Compose
- `protoc` and Buf
- PostgreSQL client tools

The planned Compose/toolchain work therefore cannot be verified in this environment until those prerequisites are installed or supplied by a container runtime. Phase 1 must provide pinned container/tool versions and explicit data preparation; normal service startup must never download or rebuild routing data.

## Dependency advisories

The imported lockfiles report 10 advisories in `gpx` and 23 in `website`. Do not run `npm audit fix --force`. Triage each reachable production dependency, upgrade compatibly, and verify the affected application behavior.

## Baseline policy

- Preserve upstream behavior while adding vertical route-generation slices.
- Treat the server analysis algorithm as canonical only after Phase 4; until then, the browser's current elevation behavior remains unchanged.
- Keep known baseline failures visible. CI should distinguish them from regressions rather than claiming a false green result.
- Re-record counts and commands whenever dependencies or diagnostics change.
