#!/usr/bin/env bash
# Idempotent rebrand of user-visible text (re-run after every upstream merge).
# Upstream's name/logo/icon are not open source (README "License"), so the
# managed build ships as "NGLB Dictate". Icons: nglb/make-icon.py, make-tray-icons.py.
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
python3 - <<'PY'
import glob, re
pat = re.compile(r"\bHandy\b(?!Keys)")
n = 0
for f in glob.glob("src/i18n/locales/*/translation.json"):
    s = open(f, encoding="utf-8").read()
    t = pat.sub("NGLB Dictate", s)
    if t != s:
        open(f, "w", encoding="utf-8").write(t)
        n += 1
print(f"rebranded {n} translation files")
PY
