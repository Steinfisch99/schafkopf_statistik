#!/usr/bin/env bash
# Baut die Android-APK für das Handy, sucht sie und schickt sie per wormhole-rs.
#
#   ./scripts/build-apk.sh              bauen, prüfen und senden
#   ./scripts/build-apk.sh --no-send    nur bauen und den Pfad ausgeben
#   ./scripts/build-apk.sh --send-only  die zuletzt gebaute APK erneut senden, ohne zu bauen
#
# Voraussetzung ist Nix mit Flakes. SDK, NDK, Java und dx kommen aus `nix develop .#android`.
# wormhole-rs wird benutzt, wenn es installiert ist, sonst über `nix run nixpkgs#wormhole-rs`.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT/crates/schafkopf-app"
TARGET_TRIPLE="aarch64-linux-android"

build=1
send=1
for arg in "$@"; do
  case "$arg" in
    --no-send) send=0 ;;
    --send-only) build=0 ;;
    -h | --help)
      sed -n '2,10p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "Unbekannte Option: $arg (siehe --help)" >&2
      exit 2
      ;;
  esac
done

nix_cmd() {
  nix --extra-experimental-features 'nix-command flakes' "$@"
}

# Die neueste gebaute Debug-APK unter target/.
find_apk() {
  find "$ROOT/target" -name 'app-debug.apk' -path '*/outputs/apk/debug/*' -printf '%T@ %p\n' 2>/dev/null \
    | sort -rn | head -n 1 | cut -d' ' -f2- || true
}

if [ "$build" -eq 1 ]; then
  echo "==> Baue die APK für $TARGET_TRIPLE (das dauert beim ersten Mal)"
  # Ohne --target baut dx für die Architektur des PCs, und das Handy lehnt die APK ab.
  (cd "$APP_DIR" && nix_cmd develop "$ROOT#android" --command \
    dx build --platform android --target "$TARGET_TRIPLE")
fi

apk="$(find_apk)"
if [ -z "$apk" ]; then
  echo "Keine APK unter $ROOT/target gefunden. Erst bauen: ./scripts/build-apk.sh" >&2
  exit 1
fi
size="$(du -h "$apk" | cut -f1)"
echo "==> APK: $apk ($size)"

# Die APK muss die Bibliothek für das Handy (arm64) enthalten, sonst sagt Android nur
# "App nicht installiert".
if command -v unzip > /dev/null 2>&1; then
  # Erst in eine Variable: `grep -q` schließt die Leitung früh, und `unzip` meldet dann unter `pipefail` einen Fehler.
  listing="$(unzip -l "$apk")"
  if grep -q 'lib/arm64-v8a/' <<< "$listing"; then
    echo "==> Enthält lib/arm64-v8a: passt zu aktuellen Handys"
  else
    echo "WARNUNG: Die APK enthält kein lib/arm64-v8a, das Handy wird sie ablehnen." >&2
    exit 1
  fi
fi

if [ "$send" -eq 0 ]; then
  exit 0
fi

echo "==> Sende mit wormhole-rs. Auf dem Handy den angezeigten Code eingeben."
if command -v wormhole-rs > /dev/null 2>&1; then
  exec wormhole-rs send "$apk"
else
  nix_cmd run nixpkgs#wormhole-rs -- send "$apk"
fi
