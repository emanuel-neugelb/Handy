#!/usr/bin/env bash
# Reports whether upstream Handy has a release newer than our baseline.
# Exit 0 = up to date, 10 = newer upstream release available, 1 = error.
# Read-only: it only calls the public GitHub API.
set -euo pipefail

UPSTREAM_REPO="cjpais/Handy"
cd "$(git rev-parse --show-toplevel)"

BASELINE="v$(grep -m1 '^version' src-tauri/Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')"
LATEST="$(gh api "repos/${UPSTREAM_REPO}/releases/latest" --jq .tag_name)"
PUBLISHED="$(gh api "repos/${UPSTREAM_REPO}/releases/latest" --jq .published_at)"

echo "NGLB baseline : ${BASELINE}"
echo "Upstream      : ${LATEST} (published ${PUBLISHED})"

if [[ "$(printf '%s\n%s\n' "${BASELINE}" "${LATEST}" | sort -V | tail -1)" == "${BASELINE}" ]]; then
  echo "UP TO DATE"
  exit 0
fi

echo "NEWER UPSTREAM RELEASE AVAILABLE -> follow NGLB-RUNBOOK.md section 'Upstream update'"
exit 10
