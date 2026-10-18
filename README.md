# Schafkopf-Statistik

Eine Android-App, die beim Schafkopfen **jedes Spiel mitschreibt** und daraus Statistiken macht:
wer wie oft gespielt, gewonnen und gezahlt hat, wer am liebsten klopft, welche Sau am häufigsten
gerufen wird, wer mit wem gut zusammenspielt.

**Ziel: so viel wie möglich in Rust.** Die App ist am Ende ein Android-Paket (APK), die gesamte Logik
liegt in einem Rust-Kern.

## Use Case und Ablauf

Wir sitzen meist zu viert am Tisch, es können aber **2 bis 10 Spieler** sein. Ein Abend läuft so:

1. **Neuer Abend:** Die Mitspieler werden in **Sitzreihenfolge** (Spielrichtung, im
   Uhrzeigersinn) **aus der Liste der gespeicherten Spieler gewählt**. Einen neuen Namen tippst du nur
   einmal, danach steht er zur Auswahl. Die Spieler des letzten Abends sind vorbelegt.
2. **Regelwerk und Tarif wählen.** Ein **Regelwerk** legt fest, welche Spielarten ihr spielt (zum
   Beispiel nur Sauspiel, Wenz, Geier, Farbsolo und Ramsch). Ein **Tarif** legt fest, was es kostet
   (Sauspiel, Solo, Schneider, Schwarz, Laufende). Beide legst du **mit Namen** in der Verwaltung an,
   es gibt jeweils einen **Standard**, und beim neuen Abend wählst du aus den gespeicherten oder
   passt den Tarif nur für diesen Abend an. Der Tarif ist während des Abends **jederzeit änderbar**.
   Neue Spiele rechnen mit dem neuen Tarif, bereits eingetragene Spiele behalten ihre Beträge.
3. **Spiel beginnt:** Der **Geber** wird angegeben. Es spielen mit: der Geber, der Spieler vor ihm
   (entgegen der Spielrichtung, er hebt ab) und die zwei Spieler nach ihm. Bei 2 oder 3 Spielern
   spielen alle mit; sitzen mehr als vier Leute am Tisch, setzen die übrigen aus. Die Auswahl lässt
   sich pro Spiel ändern.
4. **Nach dem Spiel eintragen:**
   - Spielart aus einer Auswahlbox, mit den Spielarten des Regelwerks.
   - Eine Übersicht der vier Spieler: wer **gedoppelt** (geklopft) hat, wer **gewonnen** hat (beim
     Ramsch: wer verloren hat, oder mit "Durchmarsch" wer ihn gemacht hat),
     **Schneider**, **Schwarz** und wie viele **Laufende**.
   - Der **Preis wird berechnet**. Wurde am Tisch etwas anderes verlangt, kann der **verlangte Preis
     von Hand** eingetragen werden. Er ersetzt dann die Rechnung, und die App merkt sich beide Werte.
5. **Speichern:** Das Spiel wird abgespeichert und **sofort in eine CSV-Datei geschrieben**. Ein
   früheres Spiel lässt sich später **bearbeiten oder löschen**, es wird dann mit dem Tarif von damals
   neu gerechnet.
6. **Graphen aktualisieren:** Im Vordergrund steht der **Kontoverlauf aller Spieler**. Er beginnt bei
   0 und kann ins Negative gehen.
7. **Der Geber rückt um einen Platz weiter**, und es geht mit dem nächsten Spiel weiter.
8. **Später ansehen:** Beginnst du einen neuen Abend, kommt der alte **ins Archiv auf dem Handy** (ein
   Abend ohne ein einziges Spiel wird nicht aufgehoben). Unter **Frühere Abende** siehst du jeden
   Abend noch einmal mit Diagramm, Statistik und Spielen, kannst ihn als CSV exportieren oder löschen.
   Ein Tipp auf einen Namen unter dem Diagramm öffnet gleich die Statistik dieses Spielers.

Die erfassten Datenpunkte und die Statistiken stehen in [`docs/datenpunkte.md`](docs/datenpunkte.md).

## Aufbau

```
schafkopf/
├── crates/schafkopf-core/   Spiellogik, Abrechnung, Statistiken (reines Rust, keine UI)
├── crates/schafkopf-charts/ Diagramme als SVG-Text (reines Rust, keine UI)
├── crates/schafkopf-app/    Die App (Dioxus) für Android und Desktop
├── scripts/build-apk.sh     APK bauen, prüfen und mit wormhole-rs aufs Handy schicken
├── docs/datenpunkte.md      Welche Daten erfasst und ausgewertet werden
├── docs/ui-entscheidung.md  Vergleich der UI-Technik für Android und Empfehlung
└── docs/android-app.md      App auf dem Handy installieren und testen (Anleitung, Checkliste)
```

`schafkopf-core` kennt weder Android noch eine UI, deshalb läuft es unverändert auf dem Handy und
lässt sich mit `cargo test` auf jedem Rechner testen:

| Modul | Aufgabe |
| --- | --- |
| `model` | Spielarten, Farben, `GameInput` (alles, was man pro Spiel einträgt) |
| `scoring` | `Tariff`, Plausibilitätsprüfung, Spielwert und Gewinn/Verlust pro Spieler |
| `table` | `Table` mit Spielern, Geber-Rotation, Spielen, Tarif und Spielarten, Bearbeiten, Rückgängig und JSON-Speicherformat |
| `library` | `Library` mit benannten Regelwerken und Tarifen, je ein Standard |
| `csv` | CSV-Kopfzeile und eine Zeile pro Spiel (zum Anhängen an die Datei), Kontostände pro Spieler |
| `stats` | Auswertung pro Spieler und pro Tisch, Kontoverlauf für Graphen |

Entscheidungen:

- **Geld in Cent als `i64`**, nie Fließkomma.
- **Eingetragen wird, was am Tisch passiert ist.** Der Betrag wird berechnet (ein anderer verlangter
  Preis lässt sich von Hand eintragen) und
  beim Eintragen im Spiel festgeschrieben, damit eine spätere Tarifänderung die Historie nicht ändert.
- **Drei Dateien** im privaten App-Ordner:
  - `schafkopf-tisch.json` ist der vollständige Stand des laufenden Abends mit Tarif und Spielarten
    (`Table::to_json` / `from_json`, mit Versionsfeld).
  - `schafkopf-spiele.csv` ist das Journal, das die App nach jedem Spiel fortschreibt (`Table::csv_row`),
    Beträge in Cent, pro Spieler Gewinn und Kontostand.
  - `schafkopf-bibliothek.json` enthält die gespeicherten Spieler und die benannten Regelwerke und Tarife
    mit ihren Standards.
  Ein Abend **kopiert** Spielarten und Tarif beim Anlegen. Spätere Änderungen in der Bibliothek
  verändern deshalb nie einen laufenden oder schon gespielten Abend. Wird ein neuer Abend begonnen,
  kommen Tisch und CSV des alten ins Unterverzeichnis `archiv`.
- **Jedes Spiel merkt sich seinen Tarif.** Beim Bearbeiten wird mit dem Tarif von damals neu
  gerechnet, nicht mit dem heutigen.

## Status

- [x] Kern: Datenmodell, Abrechnung, Statistiken, JSON, Tests (`cargo test`)
- [x] UI-Technik recherchiert: Empfehlung Dioxus 0.7 (Oberfläche in Rust, Graphen als SVG), siehe
      [`docs/ui-entscheidung.md`](docs/ui-entscheidung.md)
- [x] Prototyp geschrieben (Eingabe, SVG-Geldstand, JSON und CSV speichern und laden)
- [x] Prototyp auf dem Handy geprüft (Android 16): Eingabe, Diagramm, Darstellung und Laden nach Neustart
      klappen. Entscheidung: **Dioxus bleibt**. Siehe [`docs/android-app.md`](docs/android-app.md)
- [x] CSV-Export in den Ordner Downloads (Excel mit Semikolon oder allgemein mit Komma), am Desktop
      getestet, auf dem Handy noch zu prüfen
- [x] Spiele nachträglich bearbeiten und löschen
- [x] Regelwerke (aktive Spielarten) und benannte Tarife mit Standard und Auswahl, Verwaltung in der App
- [x] Spielerliste: Namen einmal eingeben und danach auswählen, neue Namen werden gemerkt, Verwaltung in der App
- [x] Lauf und Gegenlauf eintragen, Statistik dazu (je Spielart und je Spieler)
- [x] Frühere Abende ansehen (Diagramm, Statistik, Spiele), exportieren und löschen; Namen unter dem Diagramm
- [x] Bock (kein Bock, nächstes Spiel, nächste Runde je Regelwerk; Bockspiel und Bockrunde-Knopf, Schwarz löst aus)
- [x] Ramsch: Klopfen wirkt wahlweise nicht auf den Preis, wird aber erfasst
- [x] Icons für Eichel, Gras, Herz und Schellen bei Sau und Farbsolos
- [x] Spieler kommen und gehen im Laufe des Abends (neuer Platz nach Wahl, gehen, zurückkommen)
- [x] Spieler umsetzen, Abend beenden, Beginn und Ende, Protokoll (kommt, geht, setzt sich um) als eigene CSV
- [x] Mehrere Bockrunden pro Spiel; Regelwerk: hintereinander oder übereinander
- [x] Ramsch: Bock verdoppelt wahlweise nicht, und die Bockrunde zählt, wartet oder verfällt
- [x] Falsch verlangte Preise: wer, und welche Fehlerart (Grundtarif, Doppler, Bock, Laufende, Schneider/Schwarz, Partei, verrechnet), mit Statistik
- [x] Zu viel verlangt: Verlierer können zurückverlangen (Tarif-Option), mit Statistik zu Zahl und Geld
- [x] Gesamtstatistik über mehrere Abende (frei wählbar, mit Doppler-Anteilen und Spielerwerten) und Testabende (Branch feature/gesamtstatistik)
- [x] Sicherung aller Daten als ZIP in Downloads und Einspielen (auswählbar: Spieler, Regelwerke, Tarife, Abende)
      führen zur Statistik des Spielers
- [x] Korrekturen aus dem Handytest: Laufende bis 14 beim Sauspiel (Grenze je Spielart), Zurück-Taste
      führt in der App zurück statt in den Hintergrund; davor: Geier zählt Laufende ab 2 (ein Feld mit Wenz), Ramsch mit
      Durchmarsch-Knopf, Umlaute in beiden CSV-Formaten
- [x] Fehlerarten richten sich nach der Richtung des Preises (zu viel: dazugedichtet, zu wenig: vergessen)
- [x] Sie als Steigerung des Tout beim Farbsolo (8 Laufende, kein Schneider/Schwarz, keine zusätzliche Verdopplung)
- [x] Vergehen und Neumischen mit Strafzahlungen (Tarif-Option), Statistik, Verlauf und CSV
- [x] Spieleliste farbig nach Sauspiel, Solo und Ramsch; Schellen wie auf dem Blatt
- [ ] JSON-Sicherung exportieren und wieder importieren
- [x] Diagramm bedienbar: Tippen zeigt die Werte eines Spiels, Ziehen verschiebt, Zwei-Finger-Zoom,
      Doppeltippen und Knöpfe setzen zurück oder zeigen die letzten Spiele. Gesten sind getestet und am
      Desktop mit der Maus probiert, Zoom und Doppeltippen mit dem Finger noch nicht auf dem Handy
- [x] Android-Build mit `dx` und APK auf dem Gerät (Anleitung in `docs/android-app.md`)
- [x] Eintragen-Ablauf: Abend einrichten, Tarif, Spiel eintragen mit Preisvorschau und Korrekturfeld,
      Tischansicht mit Geldstand und Spieleliste, Rückgängig, Neuer Abend mit Archiv. Auf dem Handy
      noch zu prüfen, siehe [`docs/android-app.md`](docs/android-app.md)
- [x] Statistik-Bildschirm mit Grafiken (Ringe für Siege und Niederlagen, Balken, Ergebnisreihe Spiel für
      Spiel): Rangliste, Überblick über den Abend, Spielarten, gerufene Sau, Paare und jeder
      Spieler im Detail (Rollen, Ansagen, Serien, Spielarten, Plätze). Am Desktop geprüft, auf dem Handy
      noch zu prüfen
- [ ] Spätere Ideen: Schneider/Schwarz angesagt, Ramsch mit Jungfrau, Hochzeit-Details, Sup/Supra,
      mehrere Tische, Abendabrechnung teilen

## Entwickeln

```sh
nix develop
cargo test
cargo clippy --all-targets
cargo fmt
```

Die APK für das Handy baut und verschickt `./scripts/build-apk.sh`.

Ohne Nix genügt eine aktuelle Rust-Toolchain. Der Android-Build braucht zusätzlich das Android SDK
und NDK; `nix develop .#android` bringt alles mit, siehe [`docs/android-app.md`](docs/android-app.md).

## Versionen

Wichtige Stände sind mit einem **Tag** markiert, einer dauerhaften Marke auf einem Commit. Zu ihnen
kann man jederzeit zurückkehren.

| Tag | Inhalt |
| --- | --- |
| `schafkopf-v0.1.0` | Erste Fassung, die auf dem Handy gut läuft: Abend einrichten mit Spielerliste, Regelwerken und benannten Tarifen, Eintragen mit Preisvorschau, Bearbeiten, Diagramm zum Bedienen, Statistik mit Grafiken, CSV-Export, Zurück-Taste. |

Befehle im Ordner `sandbox` (das Repo liegt eine Ebene über `schafkopf/`):

```sh
git tag -n
git diff schafkopf-v0.1.0 --stat
git switch -c zurueck-auf-0.1 schafkopf-v0.1.0
git worktree add ../sandbox-v0.1.0 schafkopf-v0.1.0
```

- `git tag -n` listet die Tags mit ihrer Beschreibung.
- `git diff schafkopf-v0.1.0 --stat` zeigt, was sich seitdem geändert hat.
- `git switch -c zurueck-auf-0.1 schafkopf-v0.1.0` legt einen neuen Zweig auf diesem Stand an und wechselt
  dorthin. Der Tag selbst bleibt dabei unverändert.
- `git worktree add ../sandbox-v0.1.0 schafkopf-v0.1.0` legt den alten Stand in einen **zweiten Ordner**,
  ohne den laufenden zu berühren. Dort lässt sich die alte Fassung bauen, zum Beispiel um die alte APK
  noch einmal aufs Handy zu schicken.

Ein normales `git push` überträgt Tags nicht mit. Auf dem Host schickt man sie so zum Remote:

```sh
git push sync schafkopf-v0.1.0
git push sync main --follow-tags
```

Der zweite Befehl überträgt außerdem alle Tags, die auf den gepushten Commits sitzen.

## Regelvarianten

Schafkopf wird von Runde zu Runde verschieden gespielt (Tarife, Laufende ab 2 oder 3, Hochzeit,
Ramsch, Bettel). Der Kern nimmt die verbreitetste bayerische Variante als Voreinstellung und macht
alles Abweichende über den `Tariff` einstellbar. Was sich damit nicht abbilden lässt, steht in der
Liste der späteren Ideen oben.
