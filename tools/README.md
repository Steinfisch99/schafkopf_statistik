# Werkzeuge

## `android-icon.sh`: App-Icon für Android

Dioxus 0.7 (`dx`) schreibt bei jedem Android-Build sein Platzhalter-Icon ins Gradle-Projekt und hat keine
Einstellung für ein eigenes. Darum gibt es:

- `assets/android-icon.svg`: die Quelle, viewBox `0 0 108 108`, ein Element pro Zeile (`rect`, `circle`, `path`),
  ohne `transform`. Das Element mit `id="bg"` ist der Hintergrund. Das Motiv bleibt im Kreis mit Radius 33 um die
  Mitte, sonst schneiden runde Icon-Masken es ab.
- `crates/schafkopf-app/android/icon/`: daraus erzeugt (Vektoren für Android 8+, WebP-Bilder für Android 7), mit genau den
  Dateinamen, die dx schreibt. Erzeugt, aber eingecheckt, damit der Build keine weiteren Werkzeuge braucht.

`scripts/build-apk.sh` kopiert `android/icon/` nach `dx build` über die Dateien von dx und ruft `./gradlew assembleDebug`
noch einmal auf (nur Ressourcen und Packen, schnell). Nach einer Änderung an der Quelle neu erzeugen:

```sh
nix-shell -p resvg libwebp --run "tools/android-icon.sh assets/android-icon.svg crates/schafkopf-app/android/icon"
```

## `android-signing.sh`: Updates ohne Deinstallieren

Android installiert ein Update nur, wenn die APK mit demselben Schlüssel signiert ist wie die installierte App. Sonst
meldet das Handy nur "App nicht installiert", und nach dem Deinstallieren sind die Daten weg. Gradle signiert mit
`~/.android/debug.keystore` und legt die Datei **still neu an**, wenn sie fehlt.

`scripts/build-apk.sh` ruft darum `tools/android-signing.sh <apk>` auf (in `nix develop .#android`, wegen `apksigner`).
Beim ersten Lauf merkt es sich den Fingerabdruck in `tools/android-signing.sha256`, danach bricht es bei einem anderen
Schlüssel ab. **`~/.android/debug.keystore` sichern**: Ohne diese Datei lassen sich installierte Apps nicht mehr aktualisieren.
