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
| Bockspiel | Ja, wenn das Spiel ein Bockspiel war (doppelter Preis). Vorgewählt, solange ein Bock offen ist | nein |
| Bockrunde entstanden | Ja, wenn das Spiel eine Bockrunde ausgelöst hat. Schwarz schaltet es von selbst ein | nein |
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
`Gewonnen`, `Schneider`, `Schwarz`, `Laufende`, `Gegenlauf`, `Bockspiel`, `Bockrunde_ausgeloest`, `Augen`, `Preis_Cent`, `Preis_berechnet_Cent`, `Notiz`,
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

### Frühere Abende

- Beginnt man einen neuen Abend, wandern Tisch und CSV des alten in den Ordner `archiv` auf dem Handy.
  Ein Abend ohne ein einziges Spiel wird dabei verworfen statt archiviert.
- "Frühere Abende" listet sie, der neueste zuerst. Jeder lässt sich mit Diagramm, Statistik und Spielen
  ansehen, als CSV exportieren und löschen. Bearbeiten lässt sich ein früherer Abend nicht.
- Ein Tipp auf einen Namen unter dem Diagramm öffnet die Statistik, mit der Karte dieses Spielers.

### Bock

Manche Runden spielen mit Bock: Passiert etwas Bestimmtes (zum Beispiel Schwarz), sind die folgenden
Spiele Bockspiele mit doppeltem Preis.

- Das **Regelwerk** legt fest, ob es Bock gibt: "Kein Bock", "Nur das nächste Spiel" oder "Die ganze
  nächste Runde" (so viele Spiele, wie Spieler am Tisch sitzen). Mitgeliefert hat "Unsere Runde" den
  Bock für das nächste Spiel, "Alle Spielarten" keinen.
- Beim Eintragen gibt es zwei Knöpfe: **Bockspiel** (doppelter Preis, vorgewählt, solange ein Bock offen
  ist) und **Bockrunde entstanden**. Schwarz schaltet Letzteren von selbst ein, wenn es Bock gibt. Von
  Hand lässt sich beides ändern.
- Der Bock verdoppelt zusätzlich zu Klopfen, Kontra und Re. Ein verlangter Preis ersetzt wie immer die
  Rechnung.
- Wie viele Bockspiele noch offen sind, wird aus den Spielen **abgeleitet** und nicht mitgezählt.
  Deshalb stimmt es auch nach Bearbeiten, Rückgängig und Löschen. Ein neuer Auslöser während einer
  Bockrunde verlängert sie.
- Die Statistik zählt Bockspiele und ausgelöste Bockrunden, wenn es an dem Abend welche gab.

### Spieler kommen und gehen

- Ein **neuer Spieler** bekommt einen neuen Sitzplatz hinten in der Spielerliste, damit die Sitzplätze
  der früheren Spiele gleich bleiben. Die **Sitzreihenfolge** (`seating`) ist getrennt davon gespeichert:
  Der Neue wird hinter einem gewählten Spieler einsortiert. Für frühere Spiele hat er Auszahlung 0.
- Wer **geht**, steht in `absent`. Sein Stand, seine Spiele und seine Statistik bleiben. Wer
  **zurückkommt**, sitzt wieder auf dem alten Platz. Es bleiben immer 2 bis 10 am Tisch.
- Jedes Spiel merkt sich, wer dabei am Tisch saß (`at_table`). Nur dafür zählt "ausgesetzt" in der
  Statistik, und eine Bockrunde ist so lang wie die Zahl der Spieler von damals.
- Geber und Voreinstellung der Mitspielenden springen über Gegangene hinweg. Neue Spiele gehen nur
  mit Spielern am Tisch, alte lassen sich weiter bearbeiten.
- Die CSV hat für jeden Spieler des Abends zwei Spalten, auch für Gegangene und Spät-Gekommene. Wer
  dazukommt, schreibt die Datei samt Kopfzeile neu.

### Mehrere Bockrunden

- Ein Spiel kann **mehrere Bockrunden** auslösen (selten, bis zu 4). Beim Eintragen erscheint dann die Wahl
  "Wie viele Bockrunden sind durch dieses Spiel entstanden?".
- Das **Regelwerk** legt fest, wie sich Bockrunden verbinden, auch wenn eine entsteht, während schon
  eine läuft: **Hintereinander** (zwei Bockrunden: die nächsten zwei Runden sind je gedoppelt) oder
  **Übereinander** (solange beide laufen, ist der Preis vervierfacht, bei dreien mal 8 usw.).
- Bei "Übereinander" ist ein Bockspiel vorgewählt mit so vielen Verdopplungen, wie Bockrunden gerade
  laufen. Von Hand lässt es sich ändern ("mal 2", "mal 4", ...).
- CSV: zusätzlich die Spalten "Bockrunden_Anzahl" und "Bock_Verdopplungen". Die Statistik zählt die
  ausgelösten Bockrunden einzeln.

### Beginn, Ende und Protokoll des Abends

- Der Abend merkt sich **Beginn** (`created_at`), **Ende** (`ended_at`) und die Abweichung der Ortszeit von
  UTC (`utc_offset_min`, die App liest sie aus der WebView), damit Uhrzeiten stimmen.
- **Abend beenden** (Tischübersicht) setzt das Ende, legt den Abend bei den früheren Abenden ab und zeigt
  ihn dort. Beginnt man einen neuen Abend, ohne den alten zu beenden, endet er mit seiner letzten
  bekannten Aktivität (letztes Spiel, letztes Kommen oder Gehen, sonst der Beginn).
- Das **Protokoll** (`events`) hält jedes Dazukommen, Gehen, Zurückkommen und Umsetzen fest: Zeit, Ereignis,
  Spieler, hinter wem, und nach wie vielen Spielen.
- Das Protokoll ist eine **eigene CSV** (`schafkopf-ereignisse.csv`, im Archiv je Abend dabei), weil seine
  Zeilen eine andere Form haben als die Spiele. Spalten: Zeit (Ortszeit), Zeit_Unix, Ereignis, Spieler,
  Hinter, Nach_Spiel_Nr. Erste Zeile "Abend beginnt", letzte "Abend endet". Der Export-Bildschirm hat dafür
  einen eigenen Knopf (Dateiname `schafkopf-JJJJ-MM-TT-protokoll.csv`).
- **Umsetzen:** Am Bildschirm "Am Tisch" rückt jeder Spieler mit den Pfeilen einen Platz. Der Geber geht ab
  dem nächsten Spiel in der neuen Reihenfolge weiter. Es steht im Protokoll.

### Bock beim Ramsch

- Im **Tarif** (beim Ramsch) gibt es neben "Klopfen verdoppelt den Preis" auch **"Bock verdoppelt den
  Preis"**. Aus: Der Bock beim Ramsch wird trotzdem eingetragen und gezählt, ändert den Preis aber nicht.
- Im **Regelwerk** (Bock an) legt "Beim Ramsch: Was passiert mit einer Bockrunde?" fest:
  **Ramsch zählt als Bockspiel** (er verbraucht ein Bockspiel, wie bisher), **Bockrunde wartet** (der Ramsch
  verbraucht keines, die Bockrunde gilt für das nächste Spiel) oder **Bockrunde verfällt** (mit dem
  Ramsch ist die ganze laufende Bockrunde weg). Wer nicht doppeln und die Bockrunde verfallen lassen will,
  stellt beides ein: Preis aus, Bockrunde verfällt.
- Beim Ramsch ist "Bockspiel" im Formular nur vorgewählt, wenn die Regel den Ramsch als Bockspiel zählt.

### Falsch verlangt

- Weicht der verlangte Preis vom berechneten ab, erscheinen im Formular **erst dann** zwei Auswahlen:
  **Wer hat falsch verlangt?** (ohne Wahl: der Spieler) und **Was wurde vergessen oder falsch gemacht?**
  (Mehrfachauswahl).
- Fehlerarten: Falscher Grundtarif, Falsche Anzahl Doppler, Bockrunde vergessen, Laufende nicht alle oder
  keine, Schneider nicht verlangt, Schwarz nicht verlangt (zwei getrennte Knöpfe), Falsche Partei gewonnen,
  Verrechnet, Etwas anderes.
- Stimmt der Preis am Ende doch, werden beide Angaben verworfen.
- Statistik: Wie oft jeder falsch verlangt hat (Rangliste und Spielerkarte, je Fehlerart) und welche
  Fehlerarten am Abend vorkamen. CSV: "Falsch_verlangt_von" und "Fehlerarten" (mit Semikolon getrennt).

### Gesamtstatistik über mehrere Abende

- Erreichbar über **Frühere Abende → Gesamtstatistik über mehrere Abende**. Standardmäßig zählt **jeder**
  Abend mit mindestens einem Spiel (auch der laufende), einzelne lassen sich abwählen und wieder
  dazunehmen ("Keinen", "Alle").
- **Testabende** (Abend beim Start oder unter "Regeln und Tarif" markiert, bei früheren Abenden mit
  "Als Testabend markieren") erscheinen in der Liste der früheren Abende mit "Test", zählen aber **nie**
  in der Gesamtstatistik und sind dort nicht wählbar.
- Spieler werden über den **Namen** zusammengeführt (die Sitzplätze sind an jedem Abend andere).
- Allgemein: Abende, Spiele, bewegtes Geld, Siege der Spielerpartei, **wie oft gedoppelt wurde** (ohne,
  gedoppelt, vervierfacht, mal 8, ...; mindestens gedoppelt, mindestens vervierfacht), Anteil mit
  Klopfen/Kontra/Re/Schneider/Schwarz/Bock, Spielarten, falsch verlangte Preise.
  Verdopplungen zählen nach dem Preis: Ein Ramsch, bei dem Klopfen oder Bock laut Tarif nicht zählen, gilt
  nicht als gedoppelt.
- Je Spieler: Gesamtgewinn, Abende (und wie oft im Plus), Ø pro Abend und Spiel, bester und
  schlechtester Abend, Ergebnis je Abend, bestes und schlechtestes Spiel, Serien, Siege nach Rolle,
  wie oft er selbst gedoppelt hat (Klopfen, Kontra, Re), in wie vielen gedoppelten und vervierfachten
  Spielen er saß, Schneider/Schwarz, Lauf, falsch verlangt, Ergebnis je Spielart.

### Tout

- Beim Tout zählen **Schneider und Schwarz nicht zum Preis**: Der Spieler muss ohnehin alle Stiche machen.
  Der Preis ist Grundpreis plus Laufende, dann verdoppelt. Schneider und Schwarz lassen sich trotzdem
  eintragen und gehen in die Statistik ein. Im Formular steht dazu ein Hinweis, sobald Tout an ist.
- Ältere Spiele mit Tout und Schneider oder Schwarz, die schon gespeichert sind, behalten ihren
  festgeschriebenen Preis. Beim Bearbeiten wird nach der neuen Regel gerechnet.

## Wo die Daten liegen und Sicherung

Die App legt alles in ihrem **Datenordner** ab: auf Android `/data/data/de.schafkopf.statistik/files`
(privat, per Dateimanager nicht erreichbar, bei Deinstallation oder neuem Handy weg), am Rechner
`~/schafkopf` (oder `SCHAFKOPF_DATA_DIR`).

| Datei | Inhalt |
| --- | --- |
| `schafkopf-bibliothek.json` | Spieler, Regelwerke (Spielarten, Bock) und Tarife samt Standards |
| `schafkopf-tisch.json` | der laufende Abend mit allen Spielen und dem Protokoll |
| `schafkopf-spiele.csv` | seine Spiele als Tabelle |
| `schafkopf-ereignisse.csv` | sein Protokoll (Beginn, Ende, Kommen, Gehen, Umsetzen) |
| `archiv/schafkopf-tisch-<Zeitpunkt>.json` | je ein früherer Abend, dazu `schafkopf-spiele-…csv` und `schafkopf-ereignisse-…csv` |

### Sicherung erstellen und einspielen

- **Sicherung erstellen** (Einrichtung → "Sicherung" oder "Regeln und Tarif" → "Sicherung öffnen")
  packt alle diese Dateien samt einer `INFO.txt` in `schafkopf-sicherung-JJJJ-MM-TT.zip` im Ordner
  **Downloads**. Die CSV-Tabellen darin lassen sich auch am Rechner direkt öffnen.
- **Sicherung einspielen:** "ZIP-Datei wählen" öffnet den Dateiauswahl-Dialog des Handys. Danach steht da, was
  in der Datei steckt, und du wählst, was übernommen wird: **Spieler**, **Regelwerke (Spielarten, Bock)**,
  **Tarife**, **frühere Abende**, **laufender Abend**.
- Eingespielt wird nur **Neues**: Spieler, Regelwerke und Tarife mit einem Namen, den es schon gibt, bleiben
  unverändert (Groß- und Kleinschreibung zählt nicht), auch die eigenen Standards bleiben. Frühere Abende
  mit demselben Zeitpunkt im Archiv werden übersprungen. Der laufende Abend wird nur übernommen, wenn hier
  gerade keiner läuft.
- Die Datei wird geprüft: Es muss eine ZIP sein, unsichere Pfade werden ignoriert, ausgepackt darf sie
  höchstens 64 MB groß sein, und nicht lesbare Dateien werden gezählt und übersprungen.
