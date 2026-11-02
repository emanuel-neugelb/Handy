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
- Current baseline: upstream **v0.10.0** (merged 2026-11-02, release `0.10.0-nglb1`).

## Patched upstream files

Status: `done` = merged on `nglb/main` and in the pilot build.

| File | Change | Ticket | Status |
|------|--------|--------|--------|
| `src-tauri/Cargo.toml` | feature `nglb-managed`; dependency pins | HND-003, HND-007 | done |
| `src-tauri/src/settings.rs` | managed-value overlay (locked settings) | HND-003, HND-004 | done |
| `src-tauri/src/llm_client.rs` | host allowlist, no redirects (re-applied on the shared client in v0.10.0) | HND-004 | done |
| `src-tauri/src/managers/model.rs`, `managers/model/download.rs` | block HuggingFace downloads, use staged models, pinned hashes | HND-005 | done |
| `src-tauri/src/managers/history.rs` | retention: recording and transcript deleted at save (`history_limit=0`), startup sweep of leftover wavs | HND-006 | done |
| `src-tauri/src/transcription_hook.rs` | upstream user hook (program receiving transcripts) disabled in the managed build; invariant 8 | v0.10.0 sync | done |
| `src-tauri/src/lib.rs` | managed layer loaded; updater and opener plugins removed; window title "NGLB Dictate" | HND-003, HND-005 | done |
| `src-tauri/src/tray.rs` | tray tooltip "NGLB Dictate v..." | HND-013 | done |
| `src-tauri/tauri.conf.json` | CSP, updater block removed, product name and identifier `com.neugelb.dictate` | HND-004, HND-005 | done |
| `src-tauri/capabilities/default.json`, `desktop.json` | `opener` and `updater` permissions dropped | HND-004 | done |
| `index.html`, `.env.production` | document title, production flags | HND-013 | done |
| `src/components/icons/*` (`HandyHand`, `HandyMascot`, `HandyTextLogo`, `TrayIcon`) | NGLB branding replaces Handy name and logo | HND-013 | done |
| `src/components/onboarding/OnboardingFlow.tsx` | branding | HND-013 | done |
| `src/components/settings/about/AboutSettings.tsx` | source link to `emanuel-neugelb/NGLB-Dictate`; donate button removed | HND-013 | done |
| `src/i18n/locales/*` | Handy name replaced by NGLB Dictate (`nglb/rebrand.sh`, idempotent) | HND-013 | done |
| `src-tauri/icons/*` | placeholder NGLB icons (`nglb/make-icon.py`, `make-tray-icons.py`) | HND-013 | done, real artwork open |

After each upstream sync, re-run `nglb/rebrand.sh` (also run by `build-pkg.sh`) and check `src/i18n/locales/*` for new "Handy" strings.

## New NGLB-only files

| File | Purpose | Ticket |
|------|---------|--------|
| `src-tauri/src/nglb/mod.rs` | managed policy, allowlist, fail-closed plist handling, guards | HND-003, HND-004 |
| `nglb/sync-upstream.sh` | upstream merge workflow (`--pr`) | HND-002 |
| `nglb/check-upstream-release.sh` | reports whether upstream has a newer release | HND-014 |
| `nglb/check-invariants.sh`, `allowed-plugins.txt`, `allowed-git-sources.txt` | CI invariants | HND-007 |
| `nglb/build-pkg.sh`, `rebrand.sh`, `make-icon.py`, `make-tray-icons.py` | packaging and branding | HND-008, HND-013 |
| `.github/workflows/nglb-ci.yml`, `nglb-upstream-sync.yml` | CI invariants and the weekly upstream sync | HND-007, HND-014 |
| `.github/CODEOWNERS` | review ownership | HND-007 |
| `NGLB-NOTICE.md`, `NGLB-RUNBOOK.md`, `NGLB-PATCHES.md` | attribution, operations, patch registry | HND-009, HND-014 |

## Disabled upstream workflows
All inherited upstream workflows (build, release, playwright, nix and others) are disabled on the fork through the GitHub API. Re-disable any workflow that a sync reintroduces.
