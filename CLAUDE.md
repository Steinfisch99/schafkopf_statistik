# Schafkopf-Statistik: Arbeitsnotizen für Claude

Diese Datei ist das Gedächtnis zwischen Sitzungen (die VM vergisst alles). Produktbeschreibung: `README.md`.
Datenmodell und Regeln: `docs/datenpunkte.md`. Handy-Checkliste: `docs/android-app.md`.
Entscheidung zur Oberfläche: `docs/ui-entscheidung.md`.

## Arbeitsweise mit dem Nutzer

- **Antworten auf Deutsch.** Am Ende einer Arbeit: kurz, was gemacht ist, was ungetestet ist, der Befehl zum Bauen.
- **Direkt auf `main` arbeiten und committen, keine Branches** (Nutzer hat das ausdrücklich gesagt). Wenn das
  Arbeitsverzeichnis des Nutzers währenddessen stabil bleiben soll, ginge ein Worktree unter `/scratch` mit einem
  Wegwerf-Branch, am Ende `git merge --ff-only` und aufräumen. Das habe ich zuletzt so gemacht.
- Der Nutzer **baut und installiert auf dem Handy selbst**: `./scripts/build-apk.sh` (Kabellos über wormhole-rs).
  Ich kann die APK in der VM nicht installieren und das Handy nicht testen.
- Pushen geht von der VM nicht (keine SSH-Schlüssel). Tags und Commits liegen nur lokal, der Nutzer pusht auf dem
  Host (`git push sync <tag>`). Aktuelle Marken: `schafkopf-v0.1.0`, `v0.2.0`, `v0.3.0` (Stand `b8725b9`).
- **Bestehende Statistiken nie entfernen**, Grafiken kommen dazu. Haltbarkeit der Daten ist wichtig: Neue Felder
  immer mit `#[serde(default)]`, alte Dateien müssen weiter laden (dafür gibt es in `tests.rs` Tests).
- Jede neue Funktion bekommt: Tests, einen Abschnitt in `docs/datenpunkte.md`, einen Abschnitt in der Checkliste
  `docs/android-app.md` (fortlaufend nummeriert, "Darstellung" ist immer der letzte) und eine Zeile in der
  README-Liste.
- Commit-Nachrichten Englisch, mit den Trailern `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`
  und `Claude-Session: ...` (siehe System-Hinweis der Sitzung).

## Aufbau

Rust-Workspace unter `crates/`:

- `schafkopf-core`: Datenmodell (`model.rs`), Abrechnung (`scoring.rs`), Tisch/Abend (`table.rs`), Statistik
  (`stats.rs`, `overall.rs` für mehrere Abende), CSV (`csv.rs`), Bibliothek mit Spielern, Regelwerken, Tarifen
  (`library.rs`), Sicherung als ZIP (`backup.rs`), Uhrzeiten (`clock.rs`). Alle Tests in `tests.rs`. Kein UI-Code.
- `schafkopf-charts`: SVG-Texte für Verlaufsdiagramm, Torten, Farb-Icons.
- `schafkopf-app`: Dioxus 0.7 (WebView). `draft.rs` (Eingabeentwurf fürs Formular), `views/` (Bildschirme),
  `storage.rs` (Dateien), `navigation.rs` (Zurück-Taste), `summary.rs`/`stats_text.rs` (Texte), `export.rs`.

Daten liegen im Datenordner (Android: `/data/data/de.schafkopf.statistik/files`, sonst `~/schafkopf` oder
`SCHAFKOPF_DATA_DIR`): `schafkopf-tisch.json`, `schafkopf-bibliothek.json`, CSVs, `archiv/`.

## Wichtige Fachentscheidungen (nicht aus dem Code ersichtlich)

- Preis je Gegner (Einzelwert); Solo: der Spieler bekommt/zahlt ihn dreifach. Verdopplungen: Klopfen, Kontra, Re,
  Tout, Bock. **Beim Tout zählen Schneider und Schwarz nicht zum Preis**, Laufende schon.
- Ramsch: Klopfen und Bock verdoppeln wahlweise (Tarif); die Bockrunde zählt, wartet oder verfällt (Regelwerk).
- Bock wird **aus den Spielen abgeleitet** (`bock_layers_before`), nie mitgezählt, damit Bearbeiten, Löschen und
  Rückgängig stimmen. Mehrere Bockrunden: hintereinander oder übereinander (Regelwerk).
- Spieler kommen/gehen: neue bekommen hinten einen neuen Sitzplatz, die Reihenfolge steht getrennt in `seating`,
  Gegangene in `absent`; jedes Spiel merkt sich `at_table`. Protokoll in `events`, eigene CSV.
- **Falsch verlangt:** nur die **Gewinnerseite** verlangt (beide beim Sauspiel), automatisch vorgewählt. Zu viel
  verlangt: Verlierer dürfen das **Doppelte des Mehrbetrags** zurückverlangen, immer in voller Höhe
  (`reclaimed_value`, Preis = verlangt − 2·Mehrbetrag, nie unter 0). Tarif-Option schaltet es ab.
- **Falsch-verlangt-Fehlerarten** hängen von der Richtung ab (`PriceMistake::for_price`): zu viel = "dazugedichtet",
  zu wenig = "vergessen/nicht verlangt". Alte Varianten bleiben im Enum, damit gespeicherte Daten laden.
- **Sie** ist **keine Spielart**, sondern `GameInput.sie`: Steigerung des Tout, nur beim Farbsolo. Immer 8 Laufende, Schneider/
  Schwarz zählen nicht, **keine** Verdopplung über Tout hinaus (der Nutzer hat das ausdrücklich so gesagt).
- **Vergehen/Neumischen** sind `Table.offences` (`Offence`, Auszahlung wird beim Eintragen festgeschrieben). Täter zahlt
  nur den **Mitspielern der laufenden Runde** (`default_penalty_recipients`), nicht allen am Tisch. Neumischen = `OffenceKind::Misdeal`, der Geber bleibt (er ist aus den Spielen abgeleitet).
  Tarif: `penalties_enabled`, `offence_penalty`, `misdeal_penalty` (Standard 0). Strafen stecken im Stand nach
  `after_games` (`balance_history`) und in `PlayerStats::balance`. In der CSV als Zeilen in der Ereignis-Datei.
- **Vergleich** (`views/compare.rs`): `Metric` + `PeerRow` (Zahlen je Spieler, aus `PlayerStats` oder `OverallPlayer`), `Tap`/`TapKv`
  machen Zeilen in den Spielerkarten antippbar und zeigen die Rangliste. Neue Zahl in der Karte = Metric ergänzen und beide
  `*_peer`-Funktionen füllen. Doppler-Stufen exakt in `by_doublings` (`PlayerStats`, `TableStats`, `OverallPlayer`).
- Spieleliste färbt nach `GameKind::is_team_game`/`is_solo` (CSS `kind-team`, `kind-solo`, `kind-ramsch`).
- Gesamtstatistik führt Spieler über den **Namen** zusammen. Testabende (`Table.debug`) zählen nie mit.

## Bauen und prüfen in der VM

Alles in `nix-shell` (Flake `nixpkgs` aus dem Registry, Netz nur eingeschränkt, siehe `~/.claude/CLAUDE.md`):

```sh
export CARGO_TARGET_DIR=/scratch/target-schafkopf
nix-shell -I nixpkgs=flake:nixpkgs -p cargo rustc gcc pkg-config webkitgtk_4_1 gtk3 libsoup_3 xdotool openssl glib clippy rustfmt \
  --run "cargo fmt --all && cargo clippy --workspace --features schafkopf-app/desktop --all-targets && cargo test --workspace --features schafkopf-app/desktop"
```

Für das reine Kern-Crate reichen `cargo rustc gcc pkg-config openssl`. Stand: 255 Tests, alle grün.

Die Oberfläche lässt sich am Desktop prüfen: App mit `cargo build -p schafkopf-app --features desktop` bauen, unter
`Xvfb` (z. B. 440 Pixel breit, 2000 hoch) starten, mit `xdotool` klicken, mit ImageMagick (`import -window root`,
`magick ... -crop`) Bildschirmfotos machen und ansehen. Daten mit `SCHAFKOPF_DATA_DIR` und `HOME` auf Wegwerf-Ordner
setzen. Stolpersteine: `xdotool type` verschluckt schnell getippte Zeichen in Dioxus-Feldern (mit `key --delay 400`
tippen); der GTK-Dateidialog braucht `XDG_DATA_DIRS` mit den Schemas von `gtk3` und `gsettings-desktop-schemas`
(Pfade unter `/nix/store/...`), und in ihm den Pfad per `xclip` einfügen statt zu tippen (Autovervollständigung
verdoppelt Zeichen). Es gibt hier kein Python, Änderungen am besten mit dem Edit-Werkzeug (sed mit `#` oder `|` im
Muster geht oft schief; rustfmt bricht Zeilen um und macht spätere `sed`-Treffer ungültig).

## Offene Punkte und Ungetestetes (Stand: Ende der Sitzung)

- **Auf dem Handy ungetestet:** Dateiauswahl beim Einspielen der Sicherung (`views/backup_screen.rs`). Das Feld
  wird bewusst außerhalb von Dioxus im `body` angelegt (Dioxus fängt Klicks auf eigene Datei-Felder ab und öffnet
  einen Desktop-Dialog, den es auf Android nicht gibt) und per `label` ausgelöst. Am Desktop funktioniert der
  Weg. Falls der Dialog auf Android nicht aufgeht, braucht es einen anderen Weg (zum Beispiel einen Android-
  Auswahldialog per JNI wie beim Speichern in Downloads, `android.rs`).
- Auch sonst wurde fast alles nur am Desktop und mit Tests geprüft. Die Checklisten in `docs/android-app.md` sind
  für den Nutzer gedacht und noch nicht abgehakt.
- ZIP-Dateien haben als Zeitstempel 1980 (keine Zeitfunktion in der ZIP-Bibliothek eingeschaltet), harmlos.
- Die Frage "falsch verlangen aktiviert?" wurde als Option **fürs Zurückverlangen** umgesetzt (Tarif). Der Nutzer hat
  nicht bestätigt, ob er stattdessen das ganze Falsch-verlangen-Feature abschaltbar haben wollte.
- Neu und nur per Test geprüft, nicht angesehen: Antippen der Zahlen mit Rangliste (`views/compare.rs`, Tipps in `<details>`-Karten).
- Neu und nur am Desktop per Test geprüft, nicht angesehen: Bildschirm "Vergehen oder Neumischen" (`views/offence_screen.rs`),
  Farben der Spieleliste, neue Schellen-Zeichnung (`schafkopf-charts/src/suits.rs`, nach dem Bild des Nutzers vereinfacht, vorher geraten, keine
  Recherche möglich: Netz gesperrt).
- Ideen, die nie verlangt wurden: nichts offen. Wunschliste kommt vom Nutzer.
