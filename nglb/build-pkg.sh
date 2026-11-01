#!/usr/bin/env bash
# Build, sign, (notarize) and package the NGLB Dictate build (rebranded Handy fork) as a Jamf-ready .pkg.
#
# Env:
#   APP_SIGN_ID     Developer ID Application identity (else ad-hoc signed: test only)
#   PKG_SIGN_ID     Developer ID Installer identity   (else unsigned pkg)
#   NOTARY_PROFILE  notarytool keychain profile       (else not notarized)
#   MODELS_DIR      directory of approved models to stage (optional but recommended)
#   NGLB_REV        packaging revision, default 1
#   SKIP_BUILD=1    reuse an existing .app bundle
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

REV="${NGLB_REV:-1}"
APP_SRC="src-tauri/target/release/bundle/macos/NGLB Dictate.app"
OUT_DIR="${OUT_DIR:-$PWD/dist-pkg}"
STAGE_DIR_ABS="/Library/Application Support/NGLB/Dictate/models"
PKG_ID="com.neugelb.dictate.managed"

if [[ "${SKIP_BUILD:-0}" != "1" ]]; then
  ./nglb/check-invariants.sh
  ./nglb/rebrand.sh
  HANDY_FORCE_AI_STUB="${HANDY_FORCE_AI_STUB:-0}" bun run tauri build --bundles app
fi
[[ -d "$APP_SRC" ]] || { echo "Missing $APP_SRC" >&2; exit 1; }

VERSION="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP_SRC/Contents/Info.plist")"
BUNDLE_ID="$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$APP_SRC/Contents/Info.plist")"
[[ "$BUNDLE_ID" == "com.neugelb.dictate" ]] || { echo "Unexpected bundle id $BUNDLE_ID (policy domain mismatch)" >&2; exit 1; }

WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
ROOT="$WORK/root"; SCRIPTS="$WORK/scripts"
mkdir -p "$ROOT/Applications" "$SCRIPTS" "$OUT_DIR"
ditto "$APP_SRC" "$ROOT/Applications/NGLB Dictate.app"
# MIT requires the upstream copyright and permission notice to travel with copies.
cp LICENSE "$ROOT/Applications/NGLB Dictate.app/Contents/Resources/LICENSE-Handy-MIT.txt"
cp NGLB-NOTICE.md "$ROOT/Applications/NGLB Dictate.app/Contents/Resources/NGLB-NOTICE.md"

if [[ -n "${APP_SIGN_ID:-}" ]]; then
  codesign --force --deep --options runtime --timestamp \
    --entitlements src-tauri/Entitlements.plist --sign "$APP_SIGN_ID" "$ROOT/Applications/NGLB Dictate.app"
else
  echo "WARNING: APP_SIGN_ID not set; ad-hoc signing (not for production)" >&2
  codesign --force --deep --sign - "$ROOT/Applications/NGLB Dictate.app"
fi
codesign --verify --deep --strict "$ROOT/Applications/NGLB Dictate.app"

if [[ -n "${MODELS_DIR:-}" ]]; then
  mkdir -p "$ROOT$STAGE_DIR_ABS"
  ditto "$MODELS_DIR" "$ROOT$STAGE_DIR_ABS"
  (cd "$ROOT$STAGE_DIR_ABS" && find . -type f ! -name MANIFEST.sha256 -print0 | sort -z | xargs -0 shasum -a 256 > MANIFEST.sha256)
fi

cat > "$SCRIPTS/postinstall" <<'EOS'
#!/bin/bash
# Root-owned, read-only model stage: users cannot swap models.
STAGE="/Library/Application Support/NGLB/Dictate"
if [ -d "$STAGE" ]; then
  /usr/sbin/chown -R root:wheel "$STAGE"
  /bin/chmod -R go-w "$STAGE"
fi
/usr/sbin/chown -R root:wheel "/Applications/NGLB Dictate.app"
/bin/chmod -R go-w "/Applications/NGLB Dictate.app"
/usr/bin/xattr -dr com.apple.quarantine "/Applications/NGLB Dictate.app" 2>/dev/null || true
exit 0
EOS
chmod +x "$SCRIPTS/postinstall"

PKG_NAME="NGLB-Dictate-${VERSION}-nglb${REV}.pkg"
RAW="$WORK/raw.pkg"
# Non-relocatable, so Installer never "finds" and updates a copy elsewhere.
COMPONENTS="$WORK/components.plist"
pkgbuild --analyze --root "$ROOT" "$COMPONENTS" >/dev/null
/usr/libexec/PlistBuddy -c "Add :0:BundleIsRelocatable bool false" "$COMPONENTS"
pkgbuild --root "$ROOT" --component-plist "$COMPONENTS" --identifier "$PKG_ID" \
  --version "${VERSION}.${REV}" --install-location / --scripts "$SCRIPTS" \
  --ownership recommended "$RAW"

if [[ -n "${PKG_SIGN_ID:-}" ]]; then
  productsign --sign "$PKG_SIGN_ID" "$RAW" "$OUT_DIR/$PKG_NAME"
else
  echo "WARNING: PKG_SIGN_ID not set; pkg is unsigned" >&2
  cp "$RAW" "$OUT_DIR/$PKG_NAME"
fi

if [[ -n "${NOTARY_PROFILE:-}" && -n "${PKG_SIGN_ID:-}" ]]; then
  xcrun notarytool submit "$OUT_DIR/$PKG_NAME" --keychain-profile "$NOTARY_PROFILE" --wait
  xcrun stapler staple "$OUT_DIR/$PKG_NAME"
fi

echo "--- $OUT_DIR/$PKG_NAME"
shasum -a 256 "$OUT_DIR/$PKG_NAME"
pkgutil --check-signature "$OUT_DIR/$PKG_NAME" || true
pkgutil --payload-files "$OUT_DIR/$PKG_NAME" | grep -cE '.' | xargs echo "payload entries:"
