#!/usr/bin/env bash
# Baut die Android-APK für das Handy, sucht sie und schickt sie per wormhole-rs.
#
#   ./scripts/build-apk.sh              Debug-Build (schnell gebaut, große und langsame App), prüfen und senden
#   ./scripts/build-apk.sh --release    Release-Build (optimierter Rust-Code, rund 10x kleiner und viel schneller)
#   ./scripts/build-apk.sh --no-send    nur bauen und den Pfad ausgeben
#   ./scripts/build-apk.sh --send-only  die zuletzt gebaute APK erneut senden, ohne zu bauen (mit --release: die Release-APK)
#
# Voraussetzung ist Nix mit Flakes. SDK, NDK, Java und dx kommen aus `nix develop .#android`.
# wormhole-rs wird benutzt, wenn es installiert ist, sonst über `nix run nixpkgs#wormhole-rs`.
#
# Auch die Release-APK heißt bei dx `app-debug.apk` (Gradle baut die Debug-Variante um die optimierte Rust-Bibliothek)
# und ist mit dem Android-Debug-Schlüssel signiert. Sie liegt unter target/dx/<app>/release/ und lässt sich über eine
# Debug-Version installieren, ohne die App vorher zu löschen (die Daten bleiben).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT/crates/schafkopf-app"
TARGET_TRIPLE="aarch64-linux-android"

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

# Die neueste gebaute APK des Profils unter target/dx/.
find_apk() {
  find "$ROOT/target/dx" -name 'app-debug.apk' -path "*/$profile/android/*/outputs/apk/*" -printf '%T@ %p\n' 2>/dev/null \
    | sort -rn | head -n 1 | cut -d' ' -f2- || true
}

if [ "$build" -eq 1 ]; then
  echo "==> Baue die $profile-APK für $TARGET_TRIPLE (das dauert beim ersten Mal)"
  release_flag=()
  if [ "$profile" = release ]; then release_flag=(--release); fi
  # Ohne --target baut dx für die Architektur des PCs, und das Handy lehnt die APK ab.
  (cd "$APP_DIR" && nix_cmd develop "$ROOT#android" --command \
    dx build --platform android --target "$TARGET_TRIPLE" ${release_flag[@]+"${release_flag[@]}"})

  # dx schreibt bei jedem Build sein Platzhalter-Icon ins Gradle-Projekt (Dioxus 0.7 kennt kein eigenes Android-Icon).
  # Darum das eigene (android/icon, erzeugt mit ../tools/android-icon.sh) darüberkopieren und Gradle noch einmal
  # laufen lassen: Das übersetzt nur die Ressourcen neu und packt die APK mit derselben Rust-Bibliothek.
  gradle_root="$ROOT/target/dx/schafkopf-app/$profile/android/app"
  cp -r "$APP_DIR/android/icon/." "$gradle_root/app/src/main/res/"
  echo "==> Setze das App-Icon ein"
  (cd "$gradle_root" && nix_cmd develop "$ROOT#android" --command ./gradlew --quiet assembleDebug)
fi

apk="$(find_apk)"
if [ -z "$apk" ]; then
  echo "Keine $profile-APK unter $ROOT/target/dx gefunden. Erst bauen: ./scripts/build-apk.sh --$profile" >&2
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

# Updates klappen nur mit demselben Schlüssel wie die installierte App (sonst deinstallieren, Daten weg).
nix_cmd develop "$ROOT#android" --command "$ROOT/../tools/android-signing.sh" "$apk"

if [ "$send" -eq 0 ]; then
  exit 0
fi

echo "==> Sende mit wormhole-rs. Auf dem Handy den angezeigten Code eingeben."
if command -v wormhole-rs > /dev/null 2>&1; then
  exec wormhole-rs send "$apk"
else
  nix_cmd run nixpkgs#wormhole-rs -- send "$apk"
fi
