# Schafkopf-Statistik

Eine Android-App, die beim Schafkopfen **jedes Spiel mitschreibt** und daraus Statistiken macht:
wer wie oft gespielt, gewonnen und gezahlt hat, wer am liebsten klopft, welche Sau am häufigsten
gerufen wird, wer mit wem gut zusammenspielt.

**Ziel: so viel wie möglich in Rust.** Die App ist am Ende ein Android-Paket (APK), die gesamte Logik
liegt in einem Rust-Kern.

## Use Case

Wir sitzen zu viert am Tisch, es können aber **2 bis 10 Spieler** sein. **Einer trägt nach jedem Spiel ein**, was passiert ist:

- Was wurde gespielt (Sauspiel, Wenz, Solo, ...)?
- Wer hat gespielt, wer war Mitspieler?
- Wer hat geklopft (gedoppelt), wer hat Kontra oder Re gegeben?
- Wer hat gewonnen? Schneider, Schwarz, Laufende, Augen?
- Wie viel hat es gekostet? Das rechnet die App aus dem Tarif des Tisches aus.
- Wer hat gegeben?

Zu Beginn legt man einen **Tisch** an: Namen der Spieler, ihre **Sitzreihenfolge** (Spielrichtung, im
Uhrzeigersinn) und den ersten **Geber**. Danach wandert der Geber automatisch von Spiel zu Spiel
weiter, und die App schlägt ihn beim nächsten Eintrag vor. Gespielt wird immer zu viert (bei 2 oder 3
Spielern mit allen); sitzen mehr Leute am Tisch, setzen die anderen aus. Wer mitspielt, schlägt die
App vor (die vier links vom Geber), und es lässt sich pro Spiel ändern. Bei weniger als vier Spielern
gibt es kein Sauspiel und keine Hochzeit, weil das Zweierteams sind.
Der Tarif (Sauspiel, Solo, Schneider, Laufende, ...) ist pro Tisch einstellbar.

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
| `table` | `Table` mit Spielern, Geber-Rotation, Spielen, Rückgängig und JSON-Speicherformat |
| `stats` | Auswertung pro Spieler und pro Tisch, Kontoverlauf für Graphen |

Entscheidungen:

- **Geld in Cent als `i64`**, nie Fließkomma.
- **Eingetragen wird, was am Tisch passiert ist, nicht der Betrag.** Der Betrag wird berechnet und
  beim Eintragen im Spiel festgeschrieben, damit eine spätere Tarifänderung die Historie nicht ändert.
- **Speicherformat ist JSON** (`Table::to_json` / `from_json`) mit Versionsfeld. Wo die App die Datei
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
