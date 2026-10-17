# Datenpunkte und Statistiken

Zusammengestellt aus den Schafkopf-Regeln (kurzer Stand der üblichen bayerischen Variante).
Nicht jede Runde spielt alles; Unbenutztes lässt man beim Eintragen einfach weg.

## Pro Spiel erfasst (`GameInput`)

| Datenpunkt | Beispiel | Pflicht |
| --- | --- | --- |
| Geber | Anna | ja |
| Mitspielende | 2 bis 4 Plätze, die in diesem Spiel dabei sind (Voreinstellung: Geber, der Spieler vor ihm und die zwei nach ihm) | nein |
| Verlangter Preis | Einzelwert in Cent, falls am Tisch etwas anderes verlangt wurde als der Tarif ergibt | nein |
| Spielart | Sauspiel, Hochzeit, Wenz, Geier, Farbsolo, Farbwenz, Farbgeier, Bettel, Ramsch | ja |
| Farbe | beim Sauspiel die gerufene Sau (nicht Herz), beim Farbsolo die Trumpffarbe | je nach Spielart |
| Spieler | Ben (beim Ramsch: der Verlierer) | ja |
| Mitspieler | Dora (Sauspiel, Hochzeit) | je nach Spielart |
| Klopfer | Cem, Dora (jeder Klopfer verdoppelt) | nein |
| Kontra von | Cem (verdoppelt) | nein |
| Re | ja/nein (verdoppelt, nur nach Kontra) | nein |
| Tout | ja/nein (verdoppelt, nur bei Solospielen) | nein |
| Gewonnen | Spielerpartei ja/nein | ja |
| Schneider | Verlierer hat höchstens 30 Augen | nein |
| Schwarz | Verlierer hat keinen Stich, zählt zusätzlich zu Schneider | nein |
| Laufende | Anzahl, gezählt ab 3 (Wenz ab 2) | nein |
| Augen | Augen der Spielerpartei, daraus lassen sich Sieg und Schneider ableiten | nein |
| Zeitpunkt, Notiz | frei | nein |

**Berechnet** wird daraus der Einzelwert des Spiels und der Gewinn/Verlust jedes Spielers:

```
Einzelwert = (Grundtarif + Schneider + Schwarz + Laufende × Laufendensatz) × 2^(Klopfer + Kontra + Re + Tout)
```

Zweierteams (Sauspiel, Hochzeit) gibt es nur mit genau vier Mitspielenden. Beim Sauspiel bekommt jeder Gewinner den Einzelwert und jeder Verlierer zahlt ihn. Beim Solo bekommt
oder zahlt der Spieler den Einzelwert von bzw. an alle drei Gegner. Die Summe aller Spieler ist
immer null.

Voreingestellter Tarif (pro Tisch änderbar): Sauspiel 0,20 €, Solo 0,50 €, Schneider +0,10 €,
Schwarz +0,10 €, jedes Laufende +0,10 €.

## Statistiken

**Pro Spieler**

- Kontostand, bestes und schlechtestes Spiel
- Spiele gespielt, ausgesetzt, gegeben
- Siege und Siegquote als Spieler, als Mitspieler und als Gegenspieler
- Ergebnis pro Spielart (wie oft Wenz gespielt, wie oft gewonnen, was es gebracht hat)
- Wie oft geklopft und wie oft danach gewonnen, Kontra und Re ebenso
- Schneider- und Schwarz-Siege
- Längste Sieges- und Verlustserie, aktuelle Serie
- Ergebnis nach Platz ab links vom Geber (Vorhand, Mittelhand, Hinterhand, Letzter)

**Pro Tisch**

- Anzahl Spiele, bewegtes Geld, teuerstes Spiel
- Verteilung der Spielarten und Siegquote je Spielart
- Welche Sau am häufigsten gerufen wird und wie oft sie gewinnt
- Anteil der Spiele mit Klopfen, Kontra, Re, Schneider, Schwarz und mit korrigiertem Preis
- Mitspieler-Matrix: wer hat mit wem wie oft gewonnen
- Kontoverlauf aller Spieler über die Spiele (für einen Graphen), beginnt bei 0 und kann negativ werden

## Noch nicht erfasst

Schneider oder Schwarz angesagt, Sup/Supra (weitere Verdopplungen), Ramsch mit Jungfrau und
Durchmarsch-Details, Legen/Stoß-Varianten. Das lässt sich im Datenmodell nachrüsten, ohne alte
Dateien unlesbar zu machen, weil neue Felder mit Standardwerten gelesen werden.

## CSV-Datei

Eine Zeile pro Spiel, Beträge in Cent, Kopfzeile mit Spaltennamen. Spalten: `Nr`, `Zeit`, `Geber`,
`Spielart`, `Farbe`, `Spieler`, `Mitspieler`, `Mitspielende`, `Geklopft`, `Kontra`, `Re`, `Tout`,
`Gewonnen`, `Schneider`, `Schwarz`, `Laufende`, `Augen`, `Preis_Cent`, `Preis_berechnet_Cent`, `Notiz`,
danach pro Spieler `<Name> Gewinn_Cent` und `<Name> Stand_Cent`. `Preis_Cent` ist der abgerechnete
Einzelwert, `Preis_berechnet_Cent` der nach Tarif berechnete. Felder mit Komma oder Anführungszeichen
werden nach RFC 4180 maskiert. Der Tarif steht nicht in der CSV, sondern in der JSON-Datei des Tisches.

### Export

"CSV exportieren" speichert die Spiele eines Abends im Ordner Downloads, in zwei Formaten:

- **Für Excel:** Semikolon als Trennzeichen und UTF-8 mit Kennung (BOM). So öffnet das deutsche Excel
  Spalten und Umlaute richtig.
- **Allgemein:** Komma als Trennzeichen, ohne BOM, wie es Programme und Tabellenkalkulationen mit
  englischen Einstellungen erwarten.

Der Dateiname enthält das Datum des Abends (`schafkopf-2026-10-17.csv`). Eine bestehende Datei wird
nie überschrieben, es kommt eine Nummer dazu.
