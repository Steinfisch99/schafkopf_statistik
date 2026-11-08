#!/usr/bin/env bash
# Baut die Web-Version (läuft im Browser, auf dem iPhone über "Zum Home-Bildschirm") nach dist/web.
#
#   ./scripts/build-web.sh        bauen
#
# Voraussetzung ist Nix mit Flakes. Rust (wasm32), dx, wasm-bindgen und binaryen kommen aus `nix develop .#web`.
# Das Ergebnis sind statische Dateien, die auf jeden Webserver passen (zum Beispiel GitHub Pages in einem Unterordner:
# alle Pfade sind relativ). Android und Desktop berührt dieses Skript nicht.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_DIR="$ROOT/crates/schafkopf-app"
OUT="$ROOT/dist/web"

nix_cmd() { nix --extra-experimental-features 'nix-command flakes' "$@"; }

echo "==> Baue die Web-Version (das dauert beim ersten Mal)"
# dx lässt Dateien früherer Builds (mit anderem Hash im Namen) liegen, darum den alten Web-Ausgabeordner entfernen.
rm -rf "${CARGO_TARGET_DIR:-$ROOT/target}"/dx/schafkopf-app/*/web
(cd "$APP_DIR" && nix_cmd develop "$ROOT#web" --command dx build --platform web --profile web)

# dx legt das Ergebnis unter <target>/dx/schafkopf-app/<profil>/web/public ab.
target="${CARGO_TARGET_DIR:-$ROOT/target}"
built="$(find "$target/dx/schafkopf-app" -type d -path '*/web/public' | head -n 1)"
if [ -z "$built" ] || [ ! -f "$built/index.html" ]; then
  echo "FEHLER: Keine Web-Ausgabe unter $target/dx/schafkopf-app gefunden." >&2
  exit 1
fi

rm -rf "$OUT"
mkdir -p "$OUT"
cp -r "$built/." "$OUT/"
cp -r "$ROOT/web/icons" "$OUT/icons"
cp "$ROOT/web/manifest.webmanifest" "$OUT/manifest.webmanifest"

version="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -n 1)"

# index.html: relative Pfade (für Unterordner), Titel, Manifest, Icon und iPhone-Angaben, Service Worker.
sed -i \
  -e 's#"/\./assets/#"./assets/#g' \
  -e 's#<title>[^<]*</title>#<title>Schafkopf</title>#' \
  "$OUT/index.html"
head_extra='<link rel="icon" href="icons/icon-192.png"><link rel="manifest" href="manifest.webmanifest"><link rel="apple-touch-icon" href="icons/icon-180.png"><meta name="apple-mobile-web-app-capable" content="yes"><meta name="mobile-web-app-capable" content="yes"><meta name="apple-mobile-web-app-title" content="Schafkopf"><meta name="apple-mobile-web-app-status-bar-style" content="default"><meta name="theme-color" content="#1f6f3f"><meta name="format-detection" content="telephone=no">'
sw_register="<script>if ('serviceWorker' in navigator) { window.addEventListener('load', function () { navigator.serviceWorker.register('./sw.js'); }); }</script>"
sed -i \
  -e "s|</head>|${head_extra}</head>|" \
  -e "s|</body>|${sw_register}</body>|" \
  "$OUT/index.html"

# dx trägt den Pfad zur .wasm-Datei absolut ("/./assets/...") ins JavaScript ein. Unter einem Unterordner (GitHub Pages)
# stimmt das nicht, darum relativ zum Skript selbst.
sed -i 's#"/\./assets/\(schafkopf-app_bg-[^"]*\.wasm\)"#new URL("\1",import.meta.url)#g' "$OUT"/assets/*.js
if grep -q '"/\./assets/' "$OUT"/assets/*.js; then
  echo "FEHLER: Im JavaScript steht noch ein absoluter Pfad." >&2
  exit 1
fi

# Service Worker mit der Dateiliste (alles, was die App zum Starten braucht).
files="$(cd "$OUT" && find . -type f ! -name sw.js | sort | sed 's#^\./#./#' | sed 's#.*#"&"#' | paste -sd, -)"
sed -e "s#__VERSION__#${version}#" -e "s#__FILES__#[\"./\", ${files}]#" "$ROOT/web/sw.js" > "$OUT/sw.js"

echo "==> Fertig: $OUT ($(du -sh "$OUT" | cut -f1))"
echo "    Lokal ansehen: cd $OUT && python3 -m http.server 8080   (oder ein anderer statischer Webserver)"
