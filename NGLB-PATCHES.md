# NGLB-PATCHES.md

Registry of every NGLB deviation from upstream `cjpais/Handy`. This is the merge-conflict map: keep it current in every PR that touches an upstream file.

Rules:
- New files go under `nglb/` or are clearly NGLB-named. They never conflict.
- Changes to upstream files must be behind `#[cfg(feature = "nglb-managed")]` (Rust) or a build-time flag (frontend), and kept as small as possible.
- Upstream sync: `nglb/sync-upstream.sh` (a human reviews the network diff).

## Branches
- `upstream-mirror`: fast-forward-only copy of upstream `main`. Never commit here.
- `nglb/main`: upstream plus the patches below. Protected. Releases are tagged `nglb-v<upstream>-<N>`.

## Baseline
- Forked from upstream commit `7bfb2e2c934b798bcf6e3627653aed2439996f89` (v0.9.8, 2026-10-27).

## Patched upstream files

| File | Change | Ticket | Status |
|------|--------|--------|--------|
| `src-tauri/Cargo.toml` | add feature `nglb-managed`; vendor `hf-hub` | HND-003, HND-007 | planned |
| `src-tauri/src/settings.rs` | managed-value overlay; remove cloud provider presets | HND-003, HND-004 | planned |
| `src-tauri/src/llm_client.rs` | host allowlist, no redirects | HND-004 | planned |
| `src-tauri/src/catalog/mod.rs` | replace `MIRRORS` | HND-005 | planned |
| `src-tauri/src/managers/model.rs`, `managers/model/download.rs` | block HuggingFace, pinned hashes | HND-005 | planned |
| `src-tauri/src/lib.rs` | drop the updater and opener plugins; load the managed layer | HND-003, HND-005 | planned |
| `src-tauri/tauri.conf.json` | CSP, updater block removed | HND-004, HND-005 | planned |
| `src-tauri/capabilities/*.json` | drop `opener`, `updater`; trim `fs` | HND-004 | planned |
| `src/components/**` | hide provider UI, update checker and external links; show locked state | HND-003, HND-005 | planned |

## New NGLB-only files

| File | Purpose | Ticket |
|------|---------|--------|
| `nglb/sync-upstream.sh` | upstream merge workflow | HND-002 |
| `src-tauri/src/nglb/` | managed policy and the allowlist | HND-003, HND-004 |
| `.github/workflows/nglb-*.yml` | CI invariants and the weekly sync | HND-007 |
