# Schafkopf-Statistik

Eine Android-App, die beim Schafkopfen **jedes Spiel mitschreibt** und daraus Statistiken macht:
wer wie oft gespielt, gewonnen und gezahlt hat, wer am liebsten klopft, welche Sau am häufigsten
gerufen wird, wer mit wem gut zusammenspielt.

**Ziel: so viel wie möglich in Rust.** Die App ist am Ende ein Android-Paket (APK), die gesamte Logik
liegt in einem Rust-Kern.

## Use Case und Ablauf

Wir sitzen meist zu viert am Tisch, es können aber **2 bis 10 Spieler** sein. Ein Abend läuft so:

1. **Neuer Abend:** Die Mitspieler werden mit Namen in **Sitzreihenfolge** (Spielrichtung, im
   Uhrzeigersinn) eingetragen.
2. **Tarif festlegen** (Sauspiel, Solo, Schneider, Schwarz, Laufende). Der Tarif ist **jederzeit
   änderbar**. Neue Spiele rechnen mit dem neuen Tarif, bereits eingetragene Spiele behalten ihre
   Beträge.
3. **Spiel beginnt:** Der **Geber** wird angegeben. Es spielen mit: der Geber, der Spieler vor ihm
   (entgegen der Spielrichtung, er hebt ab) und die zwei Spieler nach ihm. Bei 2 oder 3 Spielern
   spielen alle mit; sitzen mehr als vier Leute am Tisch, setzen die übrigen aus. Die Auswahl lässt
   sich pro Spiel ändern.
4. **Nach dem Spiel eintragen:**
   - Spielart aus einer Auswahlbox (Sauspiel, Wenz, Solo, ...).
   - Eine Übersicht der vier Spieler: wer **gedoppelt** (geklopft) hat, wer **gewonnen** hat,
     **Schneider**, **Schwarz** und wie viele **Laufende**.
   - Der **Preis wird berechnet**. Wurde am Tisch etwas anderes verlangt, kann der **verlangte Preis
     von Hand** eingetragen werden. Er ersetzt dann die Rechnung, und die App merkt sich beide Werte.
5. **Speichern:** Das Spiel wird abgespeichert und **sofort in eine CSV-Datei geschrieben**.
6. **Graphen aktualisieren:** Im Vordergrund steht der **Kontoverlauf aller Spieler**. Er beginnt bei
   0 und kann ins Negative gehen.
7. **Der Geber rückt um einen Platz weiter**, und es geht mit dem nächsten Spiel weiter.

Die erfassten Datenpunkte und die Statistiken stehen in [`docs/datenpunkte.md`](docs/datenpunkte.md).

## Aufbau

```
schafkopf/
├── crates/schafkopf-core/   Spiellogik, Abrechnung, Statistiken (reines Rust, keine UI)
├── docs/datenpunkte.md      Welche Daten erfasst und ausgewertet werden
└── docs/ui-entscheidung.md  Vergleich der UI-Technik für Android und Empfehlung
```

`schafkopf-core` kennt weder Android noch eine UI, deshalb läuft es unverändert auf dem Handy und
lässt sich mit `cargo test` auf jedem Rechner testen:

| Modul | Aufgabe |
| --- | --- |
| `model` | Spielarten, Farben, `GameInput` (alles, was man pro Spiel einträgt) |
| `scoring` | `Tariff`, Plausibilitätsprüfung, Spielwert und Gewinn/Verlust pro Spieler |
| `table` | `Table` mit Spielern, Geber-Rotation, Spielen, Tarifänderung, Rückgängig und JSON-Speicherformat |
| `csv` | CSV-Kopfzeile und eine Zeile pro Spiel (zum Anhängen an die Datei), Kontostände pro Spieler |
| `stats` | Auswertung pro Spieler und pro Tisch, Kontoverlauf für Graphen |

Entscheidungen:

- **Geld in Cent als `i64`**, nie Fließkomma.
- **Eingetragen wird, was am Tisch passiert ist.** Der Betrag wird berechnet (ein anderer verlangter
  Preis lässt sich von Hand eintragen) und
  beim Eintragen im Spiel festgeschrieben, damit eine spätere Tarifänderung die Historie nicht ändert.
- **Zwei Dateien:** `Table::to_json` / `from_json` (mit Versionsfeld) speichert den ganzen Tisch samt
  Tarif. Dazu schreibt die App nach jedem Spiel `Table::csv_row` an eine **CSV-Datei**, die sich in
  Tabellenkalkulationen öffnen lässt (Beträge in Cent, pro Spieler Gewinn und Kontostand). Die CSV ist
  das Journal, das JSON der vollständige Stand mit Tarif. Wo die App die Dateien
  ablegt, entscheidet später die UI-Schicht.

## Status

- [x] Kern: Datenmodell, Abrechnung, Statistiken, JSON, Tests (`cargo test`)
- [x] UI-Technik recherchiert: Empfehlung Dioxus 0.7 (Oberfläche in Rust, Graphen als SVG), siehe
      [`docs/ui-entscheidung.md`](docs/ui-entscheidung.md)
- [ ] Prototyp auf dem Gerät: Eingabefeld, ein SVG-Diagramm, Datei speichern
- [ ] Android-Build (NDK, `cargo-ndk` oder `xbuild`), APK auf dem Gerät testen
- [ ] Bildschirme: Tisch anlegen, Spiel eintragen, Statistik, Kontoverlauf
- [ ] Speichern im App-Datenverzeichnis, Export/Import der JSON-Datei
- [ ] Spätere Ideen: Schneider/Schwarz angesagt, Ramsch mit Jungfrau, Hochzeit-Details, Sup/Supra,
      mehrere Tische, Abendabrechnung teilen

## Entwickeln

```sh
nix develop
cargo test
cargo clippy --all-targets
cargo fmt
```

Ohne Nix genügt eine aktuelle Rust-Toolchain. Der Android-Build braucht zusätzlich das Android SDK
und NDK und ist noch nicht eingerichtet.

## Regelvarianten

Schafkopf wird von Runde zu Runde verschieden gespielt (Tarife, Laufende ab 2 oder 3, Hochzeit,
Ramsch, Bettel). Der Kern nimmt die verbreitetste bayerische Variante als Voreinstellung und macht
alles Abweichende über den `Tariff` einstellbar. Was sich damit nicht abbilden lässt, steht in der
Liste der späteren Ideen oben.
