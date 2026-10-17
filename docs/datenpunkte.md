# Datenpunkte und Statistiken

Zusammengestellt aus den Schafkopf-Regeln (kurzer Stand der üblichen bayerischen Variante).
Nicht jede Runde spielt alles; Unbenutztes lässt man beim Eintragen einfach weg.

## Pro Spiel erfasst (`GameInput`)

| Datenpunkt | Beispiel | Pflicht |
| --- | --- | --- |
| Geber | Anna | ja |
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

Beim Sauspiel bekommt jeder Gewinner den Einzelwert und jeder Verlierer zahlt ihn. Beim Solo bekommt
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
- Ergebnis nach Platz relativ zum Geber (Vorhand, Mittelhand, Hinterhand, Geber)

**Pro Tisch**

- Anzahl Spiele, bewegtes Geld, teuerstes Spiel
- Verteilung der Spielarten und Siegquote je Spielart
- Welche Sau am häufigsten gerufen wird und wie oft sie gewinnt
- Anteil der Spiele mit Klopfen, Kontra, Re, Schneider, Schwarz
- Mitspieler-Matrix: wer hat mit wem wie oft gewonnen
- Kontoverlauf aller Spieler über die Spiele (für einen Graphen)

## Noch nicht erfasst

Schneider oder Schwarz angesagt, Sup/Supra (weitere Verdopplungen), Ramsch mit Jungfrau und
Durchmarsch-Details, Legen/Stoß-Varianten. Das lässt sich im Datenmodell nachrüsten, ohne alte
Dateien unlesbar zu machen, weil neue Felder mit Standardwerten gelesen werden.
