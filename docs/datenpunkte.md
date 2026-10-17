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
| Spieler | Ben. Beim Ramsch: der Verlierer, oder bei "Durchmarsch" der, der alle Stiche gemacht hat | ja |
| Mitspieler | Dora (Sauspiel, Hochzeit) | je nach Spielart |
| Klopfer | Cem, Dora (jeder Klopfer verdoppelt) | nein |
| Kontra von | Cem (verdoppelt) | nein |
| Re | ja/nein (verdoppelt, nur nach Kontra) | nein |
| Tout | ja/nein (verdoppelt, nur bei Solospielen) | nein |
| Gewonnen | Spielerpartei ja/nein | ja |
| Schneider | Verlierer hat höchstens 30 Augen | nein |
| Schwarz | Verlierer hat keinen Stich, zählt zusätzlich zu Schneider | nein |
| Laufende | Anzahl; gezählt beim Sauspiel und Farbsolo ab 3, bei Wenz und Geier ab 2 (im Tarif einstellbar). Höchstens 14 beim Sauspiel, der Hochzeit und dem Farbsolo, 4 beim Wenz und Geier, 11 beim Farbwenz und Farbgeier | nein |
| Gegenlauf | Ja, wenn die **Gegner** die Laufenden hatten (Gegenlauf) statt der Spielerpartei (Lauf). Der Preis ist in beiden Fällen derselbe, es zählt für die Statistik | nein |
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

## Statistiken (in der App unter "Statistik")

**Pro Spieler**

- Kontostand, bestes und schlechtestes Spiel
- Spiele gespielt, ausgesetzt, gegeben
- Siege und Siegquote als Spieler, als Mitspieler und als Gegenspieler
- Ergebnis pro Spielart (wie oft Wenz gespielt, wie oft gewonnen, was es gebracht hat)
- Wie oft geklopft und wie oft danach gewonnen, Kontra und Re ebenso
- Schneider- und Schwarz-Siege
- Längste Sieges- und Verlustserie, aktuelle Serie
- In wie vielen Spielen die eigene Seite die Laufenden hatte (Lauf oder Gegenlauf) und in wie vielen die andere
- Ergebnis nach Platz ab links vom Geber (Vorhand, Mittelhand, Hinterhand, Letzter)

**Pro Tisch**

- Anzahl Spiele, bewegtes Geld, teuerstes Spiel
- Verteilung der Spielarten und Siegquote je Spielart
- Welche Sau am häufigsten gerufen wird und wie oft sie gewinnt
- Anteil der Spiele mit Klopfen, Kontra, Re, Schneider, Schwarz und mit korrigiertem Preis
- Mitspieler-Matrix: wer hat mit wem wie oft gewonnen
- **Lauf und Gegenlauf** je Spielart: wie oft die Spielerpartei die Laufenden hatte, wie oft die Gegner, wie oft keine, und die durchschnittliche Zahl Laufende. Gezählt wird ab der Mindestzahl Laufende des Tarifs, mit dem das Spiel abgerechnet wurde
- Kontoverlauf aller Spieler über die Spiele (für einen Graphen), beginnt bei 0 und kann negativ werden

## Noch nicht erfasst

Schneider oder Schwarz angesagt, Sup/Supra (weitere Verdopplungen), Ramsch mit Jungfrau und
Durchmarsch-Details, Legen/Stoß-Varianten. Das lässt sich im Datenmodell nachrüsten, ohne alte
Dateien unlesbar zu machen, weil neue Felder mit Standardwerten gelesen werden.

## CSV-Datei

Eine Zeile pro Spiel, Beträge in Cent, Kopfzeile mit Spaltennamen. Spalten: `Nr`, `Zeit`, `Geber`,
`Spielart`, `Farbe`, `Spieler`, `Mitspieler`, `Mitspielende`, `Geklopft`, `Kontra`, `Re`, `Tout`,
`Gewonnen`, `Schneider`, `Schwarz`, `Laufende`, `Gegenlauf`, `Augen`, `Preis_Cent`, `Preis_berechnet_Cent`, `Notiz`,
danach pro Spieler `<Name> Gewinn_Cent` und `<Name> Stand_Cent`. `Preis_Cent` ist der abgerechnete
Einzelwert, `Preis_berechnet_Cent` der nach Tarif berechnete. Felder mit Komma oder Anführungszeichen
werden nach RFC 4180 maskiert. Der Tarif steht nicht in der CSV, sondern in der JSON-Datei des Tisches.

### Export

"CSV exportieren" speichert die Spiele eines Abends im Ordner Downloads, in zwei Formaten. Beide sind
UTF-8 mit Kennung (BOM) am Dateianfang, damit Umlaute in Excel, Google Tabellen und auf dem Handy
richtig erscheinen:

- **Semikolon (deutsches Excel):** Das deutsche Excel teilt die Spalten nur bei Semikolon auf.
- **Komma (international):** Für Tabellenkalkulationen mit englischen Einstellungen und Programme.

Der Dateiname enthält das Datum des Abends (`schafkopf-2026-10-17.csv`). Eine bestehende Datei wird
nie überschrieben, es kommt eine Nummer dazu.

## Regelwerke und Tarife

- Ein **Regelwerk** ist eine benannte Liste von Spielarten. Im Eingabeformular stehen nur diese zur
  Wahl. Mitgeliefert sind "Alle Spielarten" und "Unsere Runde" (Sauspiel, Wenz, Geier, Farbsolo, Ramsch),
  Letzteres als Standard.
- Ein **Tarif** ist eine benannte Liste von Beträgen. Mitgeliefert ist "20/50".
- Von beiden gibt es jeweils einen **Standard**, der beim neuen Abend vorgewählt ist. Namen sind
  eindeutig (Groß- und Kleinschreibung zählt nicht), es bleibt immer mindestens ein Eintrag, und beim
  Löschen des Standards rückt der erste übrige nach.
- Ein Abend kopiert Spielarten und Tarif beim Anlegen. Änderungen in der Bibliothek berühren weder
  laufende noch vergangene Abende. Während des Abends lassen sich Regelwerk und Tarif unter "Regeln
  und Tarif" wechseln; das gilt ab dem nächsten Spiel.
- Jedes Spiel speichert den Tarif, mit dem es abgerechnet wurde, damit das Bearbeiten alter Spiele
  nicht von einer späteren Tarifänderung abhängt.

### Grafiken in der Statistik

Zu den Zahlen gibt es Grafiken, die Zahlen stehen aber immer daneben, damit keine Farbe allein etwas
aussagt. Grün steht für gewonnen und Orange für verloren (Okabe/Ito, auch bei Farbschwäche
unterscheidbar).

- **Rangliste:** Balken für den Geldstand, von einer Mittellinie aus (Gewinn rechts, Verlust links).
- **Lauf und Gegenlauf:** Ring über alle Spielarten und ein mehrfarbiger Balken je Spielart (Lauf, Gegenlauf, ohne), dazu eine Tabelle.
- **Dieser Abend:** Ring, wie oft die Spielerpartei gewann; Balken für die Anteile mit Klopfen, Kontra,
  Re, Schneider, Schwarz und korrigiertem Preis.
- **Spielarten:** Ring mit der Verteilung; Balken für gewonnen und verloren je Spielart.
- **Gerufene Sau** und **Wer mit wem:** Balken für gewonnen und verloren, untereinander vergleichbar.
- **Pro Spieler:** Ring mit Siegen und Niederlagen und der Siegquote in der Mitte; Balken für die Rollen
  (Spieler, Mitspieler, Gegenspieler), Ansagen (Klopfen, Kontra, Re) und Plätze; Balken für die Bilanz je
  Spielart; eine Ergebnisreihe mit einem Kästchen je Spiel (Sieg, Niederlage, ausgesetzt).

### Spieler

- Die Bibliothek merkt sich die Spielernamen. Bei der Einrichtung eines Abends wählt man jeden Platz aus
  dieser Liste. Ein Name, den man dort neu tippt, kommt beim Start des Abends automatisch in die Liste.
- Namen sind eindeutig, ohne auf Groß- und Kleinschreibung zu achten. In der Verwaltung lassen sie sich
  anlegen, umbenennen und löschen.
- Ein Abend kopiert die Namen beim Anlegen. Umbenennen oder Löschen in der Liste ändert deshalb keinen
  laufenden oder gespielten Abend. Beim Einrichten sind die Spieler des laufenden Abends vorbelegt.
