# Schafkopf-Statistik

Eine Android-App, die beim Schafkopfen **jedes Spiel mitschreibt** und daraus Statistiken macht:
wer wie oft gespielt, gewonnen und gezahlt hat, wer am liebsten klopft, welche Sau am häufigsten
gerufen wird, wer mit wem gut zusammenspielt.

**Ziel: so viel wie möglich in Rust.** Die App ist am Ende ein Android-Paket (APK), die gesamte Logik
liegt in einem Rust-Kern.

## Use Case

Wir sitzen zu viert (oder zu fünft) am Tisch. **Einer trägt nach jedem Spiel ein**, was passiert ist:

- Was wurde gespielt (Sauspiel, Wenz, Solo, ...)?
- Wer hat gespielt, wer war Mitspieler?
- Wer hat geklopft (gedoppelt), wer hat Kontra oder Re gegeben?
- Wer hat gewonnen? Schneider, Schwarz, Laufende, Augen?
- Wie viel hat es gekostet? Das rechnet die App aus dem Tarif des Tisches aus.
- Wer hat gegeben?

Zu Beginn legt man einen **Tisch** an: Namen der Spieler, ihre **Sitzreihenfolge** (Spielrichtung, im
Uhrzeigersinn) und den ersten **Geber**. Danach wandert der Geber automatisch von Spiel zu Spiel
weiter, und die App schlägt ihn beim nächsten Eintrag vor. Bei fünf Spielern setzt der Geber aus.
Der Tarif (Sauspiel, Solo, Schneider, Laufende, ...) ist pro Tisch einstellbar.

Die erfassten Datenpunkte und die Statistiken stehen in [`docs/datenpunkte.md`](docs/datenpunkte.md).

## Aufbau

```
schafkopf/
├── crates/schafkopf-core/   Spiellogik, Abrechnung, Statistiken (reines Rust, keine UI)
└── docs/datenpunkte.md      Welche Daten erfasst und ausgewertet werden
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
- [ ] UI-Technik wählen und Prototyp: Kandidaten sind [Slint](https://slint.dev) (native Oberfläche,
      Android-Backend) und [Dioxus](https://dioxuslabs.com) (Rust, Oberfläche in WebView). Beides
      nutzt den Kern unverändert.
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
