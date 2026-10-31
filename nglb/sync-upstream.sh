#!/usr/bin/env bash
# Merge the latest upstream Handy into an NGLB sync branch and (optionally) open a PR.
# Never merges into nglb/main automatically: a human reviews every upstream change.
set -euo pipefail

UPSTREAM_URL="https://github.com/cjpais/Handy.git"
BASE_BRANCH="nglb/main"
MIRROR_BRANCH="upstream-mirror"
OPEN_PR=0
[[ "${1:-}" == "--pr" ]] && OPEN_PR=1

cd "$(git rev-parse --show-toplevel)"
git diff --quiet && git diff --cached --quiet || { echo "Working tree not clean" >&2; exit 1; }

git remote get-url upstream >/dev/null 2>&1 || git remote add upstream "$UPSTREAM_URL"
git fetch upstream --tags

# The mirror must always be a pure copy of upstream (fast-forward only).
git checkout -q "$MIRROR_BRANCH"
git merge --ff-only upstream/main

STAMP="$(date +%Y-%m-%d)"
SYNC_BRANCH="sync/${STAMP}"
git checkout -q "$BASE_BRANCH"
git checkout -q -B "$SYNC_BRANCH"

if git merge --no-edit "$MIRROR_BRANCH"; then
  echo "Merged cleanly into $SYNC_BRANCH"
else
  echo "CONFLICTS: resolve using NGLB-PATCHES.md as the map of touched files." >&2
  exit 2
fi

UPSTREAM_SHA="$(git rev-parse --short "$MIRROR_BRANCH")"
echo "--- Files changed vs $BASE_BRANCH (REVIEW for new network code or dependencies) ---"
git diff --stat "$BASE_BRANCH"..."$SYNC_BRANCH" | tail -n 20
echo "--- New network-related lines ---"
git diff "$BASE_BRANCH"..."$SYNC_BRANCH" -- '*.rs' '*.ts' '*.tsx' '*.json' '*.toml' \
  | grep -E '^\+' | grep -E 'https?://|reqwest|hf_hub|tauri-plugin-|updater|fetch\(' || echo "(none)"

if [[ $OPEN_PR -eq 1 ]]; then
  git push -u origin "$SYNC_BRANCH"
  gh pr create --base "$BASE_BRANCH" --head "$SYNC_BRANCH" \
    --title "Upstream sync ${STAMP} (${UPSTREAM_SHA})" \
    --body "Upstream sync to \`${UPSTREAM_SHA}\`. Review the network diff above and CI invariants before merging."
fi
