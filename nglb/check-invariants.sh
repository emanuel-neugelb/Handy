#!/usr/bin/env bash
# NGLB policy invariants. Run in CI on every PR (including upstream-sync PRs).
# Fails the build if an upstream merge re-introduces an unmanaged network path.
set -uo pipefail
cd "$(git rev-parse --show-toplevel)"
fail=0
bad() { echo "INVARIANT VIOLATION: $*" >&2; fail=1; }

# 1. Managed feature is on by default.
grep -qE '^default = \[.*"nglb-managed".*\]' src-tauri/Cargo.toml \
  || bad "nglb-managed is not a default feature in src-tauri/Cargo.toml"

# 2. Tauri config: strict CSP, no updater endpoint, no updater artifacts.
python3 - <<'PY' || fail=1
import json, sys
c = json.load(open("src-tauri/tauri.conf.json"))
errs = []
csp = c.get("app", {}).get("security", {}).get("csp")
if not csp or "default-src 'self'" not in csp:
    errs.append("CSP missing or not restrictive")
if csp and any(x in csp for x in ("http://*", "https://*", " *;", "unsafe-eval")):
    errs.append("CSP contains wildcard or unsafe-eval")
if c.get("plugins", {}).get("updater", {}).get("endpoints"):
    errs.append("updater endpoints must be empty")
if c.get("bundle", {}).get("createUpdaterArtifacts"):
    errs.append("createUpdaterArtifacts must be false")
if c.get("identifier") != "com.neugelb.dictate":
    errs.append("bundle identifier changed; update the Jamf profile preference domain too")
for e in errs:
    print("INVARIANT VIOLATION:", e, file=sys.stderr)
sys.exit(1 if errs else 0)
PY

# 3. No frontend permission for the updater.
grep -rn '"updater:' src-tauri/capabilities && bad "updater permission present in capabilities"

# 4. HTTP clients only where the guard is applied.
allowed_clients='src-tauri/src/llm_client.rs src-tauri/src/managers/model/download.rs'
while IFS= read -r f; do
  case " $allowed_clients " in *" $f "*) ;; *) bad "new HTTP client in $f (must go through nglb guards)";; esac
done < <(grep -rlE 'reqwest::(Client|get|blocking)|Client::new\(|ureq::|isahc::|hyper::client' src-tauri/src --include=*.rs | grep -v '/nglb/')

grep -q 'guard_llm_url' src-tauri/src/llm_client.rs || bad "llm_client.rs no longer calls nglb::guard_llm_url"
grep -q 'guard_download_url' src-tauri/src/managers/model/download.rs || bad "download.rs no longer calls nglb::guard_download_url"
grep -q 'enforce_settings' src-tauri/src/settings.rs || bad "settings.rs no longer calls nglb::enforce_settings"
grep -q 'Hugging Face downloads are disabled' src-tauri/src/managers/model.rs || bad "model.rs no longer blocks Hugging Face downloads"

# 5. hf_hub confined to model code.
for f in $(grep -rl 'hf_hub' src-tauri/src --include=*.rs); do
  case "$f" in src-tauri/src/managers/model.rs|src-tauri/src/managers/model/download.rs) ;; *) bad "hf_hub used in $f";; esac
done

# 6. Dependency allowlists (plugins and git sources). Review, then update the lists.
diff <(grep -oE '^tauri-plugin-[a-z-]+' src-tauri/Cargo.toml | sort -u) nglb/allowed-plugins.txt >/dev/null \
  || bad "tauri plugin set changed; review and update nglb/allowed-plugins.txt"
diff <(grep -E '^source = "git' src-tauri/Cargo.lock | sort -u) nglb/allowed-git-sources.txt >/dev/null \
  || bad "git dependency set changed; review and update nglb/allowed-git-sources.txt"

# 7. No cloud-provider endpoints or telemetry in shipped code (tests excluded).
if grep -rnE 'api\.openai\.com|api\.anthropic\.com|openrouter\.ai|api\.groq\.com|api\.cerebras\.ai|api\.z\.ai|bedrock-mantle|sentry\.io|posthog|aptabase|googletagmanager' \
     src-tauri/src src --include=*.rs --include=*.ts --include=*.tsx 2>/dev/null \
   | grep -vE '/nglb/|test|\.test\.|#\[cfg\(test\)\]|^src-tauri/src/settings\.rs|^src-tauri/src/llm_client\.rs' | grep .; then
  bad "cloud provider or telemetry host found (above)"
fi

# 8. No user-supplied program may receive transcripts (upstream transcription hook).
grep -q 'cfg(feature = "nglb-managed")' src-tauri/src/transcription_hook.rs \
  || bad "transcription hook is not disabled in the managed build"

[ $fail -eq 0 ] && echo "NGLB invariants OK"
exit $fail
