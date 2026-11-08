# Web-Version (iPhone und andere über den Browser)

Für Geräte, auf denen sich keine APK installieren lässt (iPhone), gibt es dieselbe App als Web-Seite. Sie lässt sich in
Safari über **Teilen → Zum Home-Bildschirm** wie eine App ablegen (eigenes Icon, Vollbild, startet auch ohne Netz).
Es braucht kein Apple-Konto und keinen Mac.

## Bauen und veröffentlichen

```sh
./scripts/build-web.sh
```

Das Ergebnis liegt in `dist/web` (statische Dateien, alle Pfade relativ, läuft also auch in einem Unterordner).
Rust (wasm32), `dx`, `wasm-bindgen` und `binaryen` kommen aus `nix develop .#web`.
Lokal ansehen: in `dist/web` einen beliebigen statischen Webserver starten.

GitHub Pages: `.github/workflows/web.yml` baut bei jedem Push auf `main` und veröffentlicht. Einmalig unter
Settings → Pages als Quelle "GitHub Actions" wählen. Die Seite liegt dann unter
`https://<Benutzer>.github.io/<Repository>/`.

## Wie es funktioniert

- Derselbe Rust-Code, gebaut für `wasm32-unknown-unknown` (`dx build --platform web --profile web`). Das Profil `web` in
  `Cargo.toml` ist wie `release`, nur ohne `strip` (wasm-bindgen braucht die Abschnitte im Modul). Android nutzt das Profil nicht.
- **Dateien:** `src/vfs.rs` ersetzt `std::fs`. Auf Android und am Desktop reicht es `std::fs` unverändert durch. Im Browser
  liegen die Dateien in einer Tabelle im Speicher, die beim Start aus IndexedDB geladen und bei jeder Änderung zurückgeschrieben
  wird (Datenbank `schafkopf`, Speicher `files`). `main()` wartet im Browser, bis das Laden fertig ist.
- **Zeit:** `now_secs`/`now_millis` kommen im Browser von `Date.now()` (`SystemTime` hat dort keine Uhr).
- **Sicherung:** ZIP als Download (`src/web.rs`, Blob und Link), Einspielen wie bisher über das Dateifeld.
- **Offline:** `web/sw.js` ist ein Service Worker (mit Netz immer die neueste Fassung, ohne Netz die zuletzt geladene).
  `web/manifest.webmanifest` und `web/icons/` (aus `assets/android-icon.svg`) machen die Seite installierbar.
- `build-web.sh` macht aus dem dx-Ergebnis eine Seite für Unterordner: relative Pfade (auch im JavaScript für die
  `.wasm`-Datei), Manifest, Icons, iPhone-Angaben, Service Worker mit der Dateiliste.

## Grenzen

- **Daten liegen nur im Browser des Geräts.** Löscht man die Browserdaten oder entfernt die App vom Home-Bildschirm,
  sind sie weg: regelmäßig die Sicherung ziehen. Safari räumt Daten von Webseiten nach etwa 7 Tagen ohne Nutzung auf, nicht aber
  die einer App auf dem Home-Bildschirm; darum immer die Home-Bildschirm-Variante benutzen. Die App bittet den Browser
  zusätzlich um dauerhaften Speicher (`navigator.storage.persist()`).
- **Eigene Daten je Gerät.** Es gibt keinen Abgleich zwischen Geräten, nur die Sicherung per Datei.
- Android-Funktionen (Download-Ordner per JNI) gibt es im Browser nicht, dafür den Browser-Download.

## Getestet

Mit Chromium (headless, Puppeteer): Start, Abend einrichten, Neuladen (Stand bleibt über IndexedDB), Sicherung herunterladen,
Start ohne Netz, alles unter einem Unterordner wie bei GitHub Pages. **Nicht getestet:** Safari auf einem echten iPhone
(Zum Home-Bildschirm, Download der Sicherung, Dateiauswahl beim Einspielen, Tastatur, Wischen im Diagramm).
