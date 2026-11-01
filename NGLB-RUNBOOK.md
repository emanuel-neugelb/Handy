# NGLB Dictate – Operations Runbook

Owner: IT (Emanuel). Repo: https://github.com/emanuel-neugelb/NGLB-Dictate (default branch `nglb/main`).
Upstream: https://github.com/cjpais/Handy (MIT). Current baseline: **v0.9.8**.

## 1. What is where
| Item | Location |
|---|---|
| Source / patches | `nglb/main`; patch map in `NGLB-PATCHES.md` |
| Pure upstream copy | `upstream-mirror` (fast-forward only, never edited) |
| Build + package | `nglb/build-pkg.sh` |
| Policy / PPPC profile generator | `Handy-Local-Only/jamf/make-profiles.py` (documentation folder) |
| Jamf objects | Package `NGLB-Dictate-<ver>-nglb<rev>.pkg`; profiles "NGLB Dictate - Managed Policy" and "NGLB Dictate - Privacy Permissions"; smart group "NGLB Dictate - Pilot"; Self Service policy "NGLB Dictate - Install" |
| Models (staged, not downloaded) | `Parakeet Ultra Q8_0` + `VoxLingua107` language ID, SHA-256 in `MANIFEST.sha256` inside the pkg |

## 2. Upstream update (when Handy releases a new version)

### 2.1 Detect
- Weekly (Monday) run `nglb/check-upstream-release.sh`. Exit `10` = newer upstream release.
- Automation: `nglb-upstream-sync.yml` runs every Monday 06:00 UTC and opens a sync PR via `sync-upstream.sh --pr` (can also be started manually: Actions → nglb-upstream-sync → Run workflow). The PR is never merged automatically.
- Also subscribe to releases: GitHub → cjpais/Handy → Watch → Custom → Releases.

### 2.2 Prepare (never skip a step)
1. `nglb/sync-upstream.sh --pr` – fast-forwards `upstream-mirror`, merges it into `sync/<date>`, prints changed files and **every new network-related line**, and opens a PR into `nglb/main`.
2. Review the printed "New network-related lines". Anything new that talks to a host, adds a Tauri plugin, a git dependency or an updater path is a **stop-and-review** item. Update `nglb/allowed-plugins.txt` / `allowed-git-sources.txt` only consciously.
3. Conflicts: resolve only in files listed in `NGLB-PATCHES.md`; keep the NGLB behaviour.
4. Run: `cd src-tauri && cargo test --lib` (≈330 tests) and `nglb/check-invariants.sh`. Both must pass.
5. Check the model catalog: if upstream changed the revision/hash of Parakeet Ultra or the language detector, re-stage the models from the new revision and verify SHA-256 against `catalog.json`.
6. Re-check the rebrand: window title, tray tooltip, About page, onboarding (no Handy name/logo). `nglb/rebrand.sh` is idempotent and runs during the build.
7. Merge the PR (merge commit, keep history), then tag `vX.Y.Z-nglbN`.

### 2.3 Build and package
```
bun install
MODELS_DIR=<verified models dir> OUT_DIR=<out> \
APP_SIGN_ID="Developer ID Application: …" PKG_SIGN_ID="Developer ID Installer: …" \
NOTARY_PROFILE=<notarytool profile> nglb/build-pkg.sh
```
Validate: `pkgutil --check-signature`, `spctl -a -vv` on the app, package payload count, and `codesign -dr -` (needed for PPPC).
Until the Developer ID certificates exist the build is ad-hoc signed (pilot only; cdhash changes every build).

### 2.4 Jamf rollout
1. Regenerate the profiles with the new code requirement: `make-profiles.py --policy-version <n> --llm-mode off --code-requirement "<designated requirement>"`. With Developer ID the requirement is stable across versions, so PPPC does not need to change per release.
2. Upload the package: `jamf-cli pro packages upload --yes <pkg>`.
3. Update the PPPC and policy profiles (only if the requirement or policy changed).
4. Ring rollout: pilot group (smart group) → IT → all staff. Keep the install policy on Self Service until the pilot passes.
5. Compliance extension attribute: raise `MIN_VERSION`; policy drift is reported as non-compliant.
6. Record the release: SBOM / Jira asset (version, source commit, model hashes), release note, date.

### 2.5 Rollback
Re-scope the previous package/policy to the affected group; the app data directory is independent of the install. Policy and PPPC profiles are unchanged by a rollback.

## 3. Day-to-day operations
- **Settings are enforced**, not configurable by users: updater off, start hidden, autostart, tray icon, experimental off, history limit 0 (recording deleted before paste), LLM post-processing off. A changed value is reverted on the next read.
- **Fail closed**: missing or tampered policy plist (not root-owned or group/world-writable) = local-only defaults, downloads off.
- **Uninstall for testing**: `jamf/local-test.sh uninstall` (leaves Jamf-managed profiles in place; re-run the install policy afterwards).
- **PPPC**: Accessibility is pre-approved by profile. Microphone cannot be pre-approved by MDM, the user approves once. Input Monitoring is user-approvable (`AllowStandardUserToSetSystemService`) by design.
- **Troubleshooting a Mac**: app log `~/Library/Logs/com.neugelb.dictate/handy.log` (timestamps are UTC); policy plist `/Library/Managed Preferences/com.neugelb.dictate.plist`; MDM overrides `/Library/Application Support/com.apple.TCC/MDMOverrides.plist`; profile failures: `log show --last 15m --predicate 'process == "mdmclient"'`.

## 4. Known limitations (accepted)
- Opening web links from the app (About page etc.) is allowed.
- Input Monitoring is approved by the user, which is the intended onboarding path.
- A local administrator can still tamper with the machine; the compliance EA and Jamf check-in detect and remediate drift.
