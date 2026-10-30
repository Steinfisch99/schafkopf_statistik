#!/usr/bin/env bash
# Baut die Android-APK für das Handy, sucht sie und schickt sie per wormhole-rs.
#
#   ./scripts/build-apk.sh              Debug-Build (schnell gebaut, langsame App), prüfen und senden
#   ./scripts/build-apk.sh --release    Release-Build (optimiert, kleiner, viel schneller), signieren, prüfen, senden
#   ./scripts/build-apk.sh --no-send    nur bauen und den Pfad ausgeben
#   ./scripts/build-apk.sh --send-only  die zuletzt gebaute APK erneut senden, ohne zu bauen (mit --release: die Release-APK)
#
# Voraussetzung ist Nix mit Flakes. SDK, NDK, Java und dx kommen aus `nix develop .#android`.
# wormhole-rs wird benutzt, wenn es installiert ist, sonst über `nix run nixpkgs#wormhole-rs`.
#
# Release-APKs kommen von Gradle unsigniert und lassen sich nicht installieren. Das Skript richtet sie mit
# zipalign aus und signiert sie mit dem Android-Debug-Schlüssel (~/.android/debug.keystore, wird bei Bedarf
# angelegt). Das ist derselbe Schlüssel wie bei Debug-Builds: Eine Release-APK lässt sich also über eine
# Debug-Version installieren, ohne die App vorher zu löschen (die Daten bleiben). Eigener Schlüssel: ANDROID_KEYSTORE.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT/crates/schafkopf-app"
TARGET_TRIPLE="aarch64-linux-android"
BUILD_TOOLS="34.0.0"

build=1
send=1
profile=debug
for arg in "$@"; do
  case "$arg" in
    --release) profile=release ;;
    --no-send) send=0 ;;
    --send-only) build=0 ;;
    -h | --help)
      sed -n '2,15p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
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

# Die neueste gebaute APK des Profils unter target/ (bei Release ist die signierte die neueste).
find_apk() {
  find "$ROOT/target" -name "app-$profile*.apk" -path "*/outputs/apk/$profile/*" -printf '%T@ %p\n' 2>/dev/null \
    | sort -rn | head -n 1 | cut -d' ' -f2- || true
}

# Die unsignierte Release-APK von Gradle.
find_unsigned_apk() {
  find "$ROOT/target" -name 'app-release*.apk' ! -name '*-signed.apk' -path '*/outputs/apk/release/*' -printf '%T@ %p\n' 2>/dev/null \
    | sort -rn | head -n 1 | cut -d' ' -f2- || true
}

# Richtet die APK aus, signiert sie (v2) und prüft die Signatur. Läuft in der Nix-Shell wegen SDK und Java.
sign_apk() {
  nix_cmd develop "$ROOT#android" --command bash -c '
    set -euo pipefail
    in="$1"; out="$2"; tools="$ANDROID_HOME/build-tools/$3"
    ks="${ANDROID_KEYSTORE:-$HOME/.android/debug.keystore}"
    if [ ! -f "$ks" ]; then
      echo "==> Lege den Schlüssel $ks an"
      mkdir -p "$(dirname "$ks")"
      keytool -genkeypair -keystore "$ks" -storepass android -keypass android -alias androiddebugkey \
        -keyalg RSA -keysize 2048 -validity 10000 -dname "CN=Android Debug,O=Android,C=US"
    fi
    "$tools/zipalign" -f -p 4 "$in" "$out.aligned"
    "$tools/apksigner" sign --ks "$ks" --ks-pass pass:android --key-pass pass:android \
      --ks-key-alias androiddebugkey --out "$out" "$out.aligned"
    "$tools/apksigner" verify "$out"
  ' _ "$1" "$2" "$BUILD_TOOLS"
}

if [ "$build" -eq 1 ]; then
  echo "==> Baue die $profile-APK für $TARGET_TRIPLE (das dauert beim ersten Mal)"
  release_flag=()
  if [ "$profile" = release ]; then release_flag=(--release); fi
  # Ohne --target baut dx für die Architektur des PCs, und das Handy lehnt die APK ab.
  (cd "$APP_DIR" && nix_cmd develop "$ROOT#android" --command \
    dx build --platform android --target "$TARGET_TRIPLE" ${release_flag[@]+"${release_flag[@]}"})

  if [ "$profile" = release ]; then
    unsigned="$(find_unsigned_apk)"
    if [ -z "$unsigned" ]; then
      echo "Keine Release-APK unter $ROOT/target gefunden (erwartet: .../outputs/apk/release/app-release*.apk)." >&2
      exit 1
    fi
    signed="$(dirname "$unsigned")/app-release-signed.apk"
    echo "==> Signiere $unsigned"
    sign_apk "$unsigned" "$signed"
  fi
fi

apk="$(find_apk)"
if [ -z "$apk" ]; then
  echo "Keine $profile-APK unter $ROOT/target gefunden. Erst bauen: ./scripts/build-apk.sh --$profile" >&2
  exit 1
fi
if [ "$profile" = release ] && [[ "$apk" != *-signed.apk ]]; then
  echo "Die neueste Release-APK ist nicht signiert ($apk). Erst bauen: ./scripts/build-apk.sh --release" >&2
  exit 1
fi
size="$(du -h "$apk" | cut -f1)"
echo "==> APK ($profile): $apk ($size)"

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
