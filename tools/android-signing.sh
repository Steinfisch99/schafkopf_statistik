#!/usr/bin/env bash
# Prüft, ob eine APK mit dem gewohnten Schlüssel signiert ist.
#
#   tools/android-signing.sh <apk>
#
# Android installiert ein Update nur, wenn es mit demselben Schlüssel signiert ist wie die installierte App.
# Sonst kommt nur "App nicht installiert", man muss deinstallieren, und die Daten der App sind weg. Gradle signiert
# mit ~/.android/debug.keystore und legt die Datei still neu an, wenn sie fehlt (anderer Rechner, gelöschter
# Ordner). Darum merkt sich dieses Skript den Fingerabdruck in tools/android-signing.sha256 (eingecheckt)
# und bricht ab, wenn eine APK anders signiert ist.
#
# Läuft in der Android-Shell einer App (`nix develop .#android`), weil apksigner aus dem SDK kommt.

set -euo pipefail

apk="${1:?Aufruf: tools/android-signing.sh <apk>}"
known="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/android-signing.sha256"

apksigner="$(ls "${ANDROID_HOME:?ANDROID_HOME fehlt, bitte in nix develop .#android aufrufen}"/build-tools/*/apksigner | sort | tail -n 1)"
digest="$("$apksigner" verify --print-certs "$apk" | sed -n 's/^Signer #1 certificate SHA-256 digest: //p')"
if [ -z "$digest" ]; then
  echo "FEHLER: Konnte die Signatur von $apk nicht lesen." >&2
  exit 1
fi

if [ ! -f "$known" ]; then
  echo "$digest" > "$known"
  echo "==> Schlüssel gemerkt in $known: $digest"
  echo "    Bitte die Datei committen und ~/.android/debug.keystore sicher aufbewahren (ohne sie gehen keine Updates)."
  exit 0
fi

if [ "$digest" = "$(tr -d '[:space:]' < "$known")" ]; then
  echo "==> Signiert mit dem gewohnten Schlüssel: Update ohne Deinstallieren möglich"
  exit 0
fi

cat >&2 << EOF
FEHLER: Die APK ist mit einem anderen Schlüssel signiert als bisher.
  diese APK: $digest
  gewohnt:   $(cat "$known")
Das Handy würde das Update ablehnen, und nach dem Deinstallieren wären die Daten der App weg.
Lösung: die alte ~/.android/debug.keystore zurückholen und neu bauen.
Nur wenn der alte Schlüssel wirklich verloren ist: $known löschen, dann merkt sich das nächste Bauen den neuen
(vorher in jeder App eine Sicherung anlegen, falls sie das kann).
EOF
exit 1
