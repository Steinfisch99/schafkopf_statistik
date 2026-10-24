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
| Re | wer von der Spielerpartei (Spieler oder Mitspieler) Re gegeben hat; verdoppelt, nur nach Kontra. Das Team, das Kontra gegeben hat, kann kein Re geben. Ältere Spiele ohne Angabe: der Spieler | nein |
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
`Spielart`, `Farbe`, `Spieler`, `Mitspieler`, `Mitspielende`, `Geklopft`, `Kontra`, `Re`, `Re_von`, `Tout`,
`Gewonnen`, `Schneider`, `Schwarz`, `Laufende`, `Gegenlauf`, `Bockspiel`, `Bockrunde_ausgeloest`, `Doppler_Anzahl`, `Augen`, `Verlangter_Preis_Cent`, `Preis_Cent`, `Preis_berechnet_Cent`, `Notiz`,
danach pro Spieler `<Name> Gewinn_Cent` und `<Name> Stand_Cent`. `Preis_Cent` ist der abgerechnete
Einzelwert, `Preis_berechnet_Cent` der nach Tarif berechnete. Felder mit Komma oder Anführungszeichen
werden nach RFC 4180 maskiert. Der Tarif steht je Spiel in den Spalten `Tarif_...` der CSV (Preise in Cent, Laufende, Optionen), damit auch Wechsel mitten im Abend zu sehen sind; zusätzlich in der JSON-Datei des Tisches.

### Export

"CSV exportieren" speichert die Spiele eines Abends im Ordner Downloads, in zwei Formaten. Beide sind
UTF-8 mit Kennung (BOM) am Dateianfang, damit Umlaute in Excel, Google Tabellen und auf dem Handy
richtig erscheinen:

- **Semikolon (deutsches Excel):** Das deutsche Excel teilt die Spalten nur bei Semikolon auf.
- **Komma (international):** Für Tabellenkalkulationen mit englischen Einstellungen und Programme.

In der Spiele-CSV steht, **wer** gedoppelt hat: `Geklopft` (alle Klopfer), `Kontra` (wer), `Re_von` (wer von der Spielerpartei), dazu `Tout`, `Sie`, `Bockspiel` und `Doppler_Anzahl` (wie oft der Preis nach dem Tarif verdoppelt wurde). `Verlangter_Preis_Cent` ist der am Tisch verlangte Preis, falls er vom berechneten abwich (bei Zurückverlangen unterscheidet er sich von `Preis_Cent`).

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
  **Wer hat falsch verlangt?** (siehe unten) und **Was wurde vergessen oder falsch gemacht?**
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

## Version

Die Versionsnummer steht in `Cargo.toml` (`[workspace.package]`) und erscheint in der App unten in der Tischübersicht und oben in der Einrichtung. Neuer Tag: zweite Stelle erhöhen, sonst die dritte.

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

### Wer falsch verlangt hat

- Falsch verlangen kann nur die **Gewinnerseite**, denn der Verlierer verlangt nie. Beim Sauspiel sind es also
  beide, die zusammen spielen (Spieler und Mitspieler, bei verlorenem Spiel die beiden Gegner), beim
  gewonnenen Solo der Spieler, beim verlorenen Solo die drei Gegner, beim Ramsch alle, die nicht verloren haben.
- Sobald das Ergebnis eingetragen ist, sind **alle Gewinner automatisch** als falsch verlangt eingetragen und
  folgen dem Ergebnis, wenn es sich ändert. Zur Wahl stehen nur die Gewinner, einzelne lassen sich
  abwählen (mindestens einer bleibt).
- In der Statistik zählt jeder der Eingetragenen einmal (und je Fehlerart). Die CSV-Spalte
  "Falsch_verlangt_von" nennt alle, getrennt durch Semikolon. Alte Spiele mit nur einem Namen bleiben lesbar.

### Zu viel verlangt und zurückverlangen

- Wurde der Preis **zu hoch** verlangt, bekommen die Gewinner das Mehr nur, wenn es keinem auffällt. Fällt es
  den Verlierern auf, **verlangen sie es zurück**. Sie dürfen das **Doppelte des Mehrbetrags** zurückverlangen
- Beim Eintragen erscheint bei zu hohem Preis die Frage "Ist es den Verlierern aufgefallen?": **Nicht
  aufgefallen** (der verlangte Preis gilt, Geld und Stand folgen ihm) oder **Zurückverlangt** (der Preis sinkt
  wie unten beschrieben) mit der Wahl, **wer zurückverlangt hat** (nur Verlierer, ohne Wahl alle, einer bleibt immer).
  Der verlangte Preis bleibt in beiden Fällen für die Statistik gespeichert.
- Bei **zu wenig** Verlangtem gibt es nichts zurückzuverlangen, der verlangte Preis gilt (die Gewinner
  bekommen weniger, als ihnen zustand).
- **Option im Tarif:** "Zu viel Verlangtes darf zurückverlangt werden" (Standard an, auch für alte Tarife). Aus:
  Der verlangte Preis gilt immer, die Frage erscheint nicht.
- **Statistik** (Abend und Gesamtstatistik, im Abschnitt "Falsch verlangt"): wie oft zu viel und zu wenig
  verlangt wurde, wie oft davon zurückverlangt, das zu viel verlangte Geld insgesamt, davon zurückgeholt und
  davon geblieben, und **wer wie oft zurückverlangt** hat. Je Spieler: zu viel und zu wenig verlangt,
  zurückverlangt, und das Geld in Euro: zu viel bekommen (nicht aufgefallen), zurückgeben müssen, zu wenig
  bekommen; als Verlierer zu viel gezahlt, durch Zurückverlangen gespart, weniger gezahlt als nötig.
- CSV: zusätzlich "Zurueckverlangt" (ja/nein) und "Zurueckverlangt_von".

#### Wie viel zurückverlangt wird

Die Verlierer dürfen **das Doppelte des Mehrbetrags** zurückverlangen, sonst würden die Gewinner immer zu viel
verlangen. Die App nimmt immer den **Höchstbetrag** (zu wenig zurückverlangt wird nicht erfasst):

- Mehrbetrag = verlangter Preis minus berechneter Preis, je Gegner.
- Der Preis sinkt auf **verlangt − 2 · Mehrbetrag**, nie unter 0.
- Beispiel: Das Spiel kostet 60, verlangt werden 80. Mehrbetrag 20, die Verlierer verlangen 40 zurück, es
  bleiben **40**. Die Gewinner bekommen damit 20 weniger, als ihnen zugestanden hätte.
- Bei 60 und verlangten 120 (oder mehr) bleibt 0. Die Gewinner können durchs Falschverlangen nie ins Minus kommen: Sie geben höchstens das
  zurück, was sie bekommen haben, und stehen dann wie vor dem Spiel da. Wer statt 1 € 20 € verlangt, gibt höchstens die 20 € zurück,
  nicht 2 · 19 €. Das Geld, das als zurückgeholt zählt, ist der tatsächlich zurückgegebene Betrag.
- Statistik: "Zurückverlangt (Geld)" ist das tatsächlich zurückgegebene Geld (hier 40 je Gewinner). "Unterm
  Strich für die Gewinner" ist die Summe aus dem, was sie mehr oder (nach dem Zurückverlangen) weniger bekommen
  haben als berechnet (hier −20 je Gewinner). Je Spieler: "Zurückgeben müssen" (Gewinner) und "Zurückverlangt
  (zurückgeholt)" (Verlierer) in diesem Sinn.

### Fehlerarten bei falsch verlangtem Preis

Wer zu viel verlangt, hat nichts vergessen, sondern **dazugedichtet**. Die Auswahl der Fehlerarten richtet sich
deshalb nach der Richtung des Preises:

- **Zu viel verlangt:** Falscher Grundtarif, Falsche Anzahl Doppler, Laufende nicht alle oder keine, Falsche Partei
  gewonnen, Verrechnet, Etwas anderes und zusätzlich **Schneider dazugedichtet**, **Schwarz dazugedichtet**,
  **Bock dazugedichtet**.
- **Zu wenig verlangt:** dieselben gemeinsamen Arten und zusätzlich **Bockrunde vergessen**, **Schneider nicht
  verlangt**, **Schwarz nicht verlangt**.
- Eine schon gewählte Fehlerart bleibt sichtbar, auch wenn sie nach einer Preisänderung nicht mehr zur Richtung
  passt. Alte Spiele mit "nicht verlangt" bleiben unverändert lesbar.

### Sie

**Sie** ist keine eigene Spielart, sondern die **Steigerung des Tout beim Farbsolo**. Im Formular lässt sich der Knopf
"Sie" erst drücken, wenn "Tout" an ist (nur beim Farbsolo). Beim Sie gilt:

- Der Spieler hat **immer alle 8 Laufende** (sie werden von selbst eingetragen, der Zähler und "Gegenlauf" entfallen).
- **Schneider und Schwarz zählen nicht** zum Preis (wie beim Tout, sie bleiben für die Statistik eingetragen).
- Der Preis ist **wie beim Tout**, Sie verdoppelt nicht noch einmal (Klopfen, Kontra, Re und Bock gelten wie sonst).
  Der Unterschied zum Tout sind die 8 Laufende, die immer zählen. Beispiel mit Standardtarif: (50 + 8 · 10) mal 2 = 260
  je Gegner, der Spieler bekommt dreifach.
- Wird Tout abgeschaltet oder die Spielart gewechselt, fällt Sie weg. Die Spieleliste zeigt "Tout, Sie", die CSV hat
  eine Spalte "Sie". Alte Dateien ohne das Feld laden wie bisher (kein Sie).

### Vergehen, Strafzahlungen und Neumischen

- Im Abend gibt es "Vergehen oder Neumischen". Eingetragen werden die **Art** (Nicht bedient, Aus der Reihe
  gespielt, Karten gezeigt, Absprache oder Zeichen, Falsch gegeben (neu gemischt), Etwas anderes) und **wer** es
  war. Beim **Neumischen** ist der Geber der Täter und bleibt derselbe (der Geber rückt nicht weiter).
- **Strafe:** Der Täter zahlt jedem **Mitspieler der laufenden Runde** (die, die beim nächsten Spiel mitspielen, ohne
  den Täter; wer aussetzt, bekommt nichts, auch wenn er am Tisch sitzt) den Betrag aus dem Tarif ("Vergehen je
  Mitspieler", Standard 0,50 €; "Neumischen je Mitspieler", Standard **0**, also keine Strafe). Betrag und Empfänger
  lassen sich beim Eintragen ändern. Er wird festgeschrieben, eine spätere Tarifänderung berührt ihn nicht.
- **Option im Tarif:** "Vergehen und Neumischen kosten eine Strafe" (Standard an, auch für alte Tarife). Aus:
  Alles wird trotzdem eingetragen und gezählt, kostet aber nichts.
- Die Strafen zählen zum Kontostand und stehen im Verlauf: Eine Strafe nach n Spielen steckt im Stand nach
  Spiel n (vor dem ersten Spiel im Startwert). Ein Eintrag lässt sich zurücknehmen.
- **Statistik** (Abend und Gesamtstatistik, Abschnitt "Vergehen und Neumischen"): Zahl der Vergehen und des
  Neumischens, nach Art, nach Spieler, die Strafen insgesamt; je Spieler gezahlte und bekommene Strafen.
- **Liste:** Unter den Spielen steht "Vergehen und Neumischen" mit Uhrzeit, Spieler, Art und Strafe.
- **CSV:** Die Ereignis-Datei (`schafkopf-ereignisse.csv`) enthält die Vergehen als Zeilen "Vergehen: Art" und hat
  eine zusätzliche Spalte "Strafe_je_Mitspieler_Cent".
- Alte Dateien ohne Vergehen laden wie bisher (`offences` fehlt = keine).

### Darstellung der Spiele

- In der Spieleliste haben **Spiele zu zweit** (Sauspiel, Hochzeit, blau), **Alleinspiele** (Wenz, Geier, Farbsolo,
  Farbwenz, Farbgeier, Sie, Bettel, violett) und **Ramsch** (grau) einen farbigen Rand und eine leichte
  Hintergrundfarbe. Eine Legende steht über der Liste, der Name der Spielart steht zusätzlich im Text.
- Das Symbol für **Schellen** ist wie auf dem bayerischen Blatt eine Kugel mit gelbem Feld oben, rotem Band mit drei Löchern und grünem Blatt unten (vorher eine Kirchenglocke).

### Doppler aufgeschlüsselt

- Die Statistik zeigt, in wie vielen Spielen der Preis **genau** so oft verdoppelt war: ohne Doppler, nur verdoppelt
  (mal 2), vervierfacht (mal 4), verachtfacht (mal 8), und so weiter. Gezählt werden Klopfen, Kontra, Re, Tout und Bock
  (beim Ramsch nur, was der Tarif zählen lässt).
- Das gibt es für **den Abend**, für **alle gewählten Abende** und **je Spieler** (seine Spiele, bei denen er
  mitgespielt hat). Die Zahlen "in gedoppelten Spielen" und "in vervierfachten Spielen" bleiben und zählen
  "mindestens".

### Spieler vergleichen

- In der Karte eines Spielers (Abend und Gesamtstatistik) lässt sich fast jede Zahl **antippen**: Darunter erscheint die
  **Rangliste dieser Zahl** für alle Spieler, die an dem Abend (bzw. den gewählten Abenden) dabei waren. Wer die Karte
  geöffnet hat, ist hervorgehoben, gleiche Werte teilen sich den Platz, der größte Wert steht oben. Ein zweiter Tipp
  schließt die Liste, pro Karte ist eine offen.
- Antippbar sind: Bilanz, Ø pro Spiel und Abend, bestes und schlechtestes Spiel, bester und schlechtester Abend, Abende
  und Abende im Plus, gespielt, ausgesetzt, gegeben, die Siegquote als Spieler, Mitspieler und Gegenspieler, Klopfen,
  Kontra, Re, jede Doppler-Stufe, Siege mit Schneider und Schwarz, Lauf eigene und andere Seite, Serien, Vergehen,
  Neumischen, Strafen und falsch verlangte Preise. Quoten werden nach dem Anteil sortiert (mit Spiele-Zahl dahinter),
  wer keine Spiele in der Rolle hat, steht nicht in der Liste.

### Wer doppelt am häufigsten, Tout und Bockrunden

- **Wer doppelt am häufigsten?** Eigener Abschnitt in der Statistik des Abends und in der Gesamtstatistik: Klopfen, Kontra und Re
  zusammen, die meisten zuerst, mit der Aufteilung (Klopfen, Kontra, Re) und dem Anteil an den Spielen des Spielers. Gleiche
  Summen teilen sich den Platz. Wer nicht mitgespielt hat, steht nicht in der Liste.
- **Tout:** Abschnitt "Tout" mit der Rangliste, wie oft jemand Tout angesagt hat (Sie zählt mit, wird aber extra genannt) und wie oft
  er gewann. In der Spielerkarte stehen "Tout gespielt" und, falls vorhanden, "Davon Sie". Beides ist antippbar (Rangliste nach
  Zahl der Tout-Spiele). Ohne Tout steht ein Hinweis.
- **Selbst gedoppelt** ist in der Spielerkarte antippbar (Abend und Gesamtstatistik), die Rangliste sortiert nach der Summe.
- **Bockrunden:** "Bockrunden entstanden" steht unter "Dieser Abend" (nur wenn es Bock gab), in der Gesamtstatistik mit der Zahl der Abende.
- Alte Dateien laden wie bisher, die Zahlen werden aus den Spielen berechnet.

### Metadaten: Tarife, Regelwerke, Kommen und Gehen

- **Tarif und Regelwerk:** Jedes Spiel merkt sich den Namen des Tarifs und des Regelwerks aus der Bibliothek, mit denen es gespielt
  wurde (`tariff_name`, `rule_set_name`; Bearbeiten ändert das nicht). Die Statistik zeigt **Spiele je Tarif** und je Regelwerk und
  **wie oft gewechselt wurde**, gezählt von einem Spiel zum nächsten (ein Wechsel nach dem letzten Spiel zählt nicht). Spiele mit
  eigenem, nicht gespeichertem Tarif heißen "Eigener Tarif" (bei mehreren verschiedenen "Eigener Tarif 1", "2", ...); ältere Spiele ohne
  Namen bekommen den Namen des jetzigen Tarifs, wenn die Werte gleich sind. In der CSV stehen die Spalten `Tarif`, `Regelwerk` und alle Tarifwerte (`Tarif_Sauspiel_Cent` usw.) je Spiel.
- **Wer kam, wer ging** (Abend): je Spieler, wann er kam (von Anfang an mit Uhrzeit, sonst Uhrzeit und "nach Spiel N") und wann er
  ging oder "blieb bis zum Ende", dazu wie oft er zwischendurch weg war.
- **Wann kommt wer, wann geht wer?** (Gesamtstatistik): je Spieler Ø Ankunft und Ø Ende mit frühestem und spätestem Mal, an wie vielen
  Abenden er später kam und früher ging; dazu Ø Beginn und Ø Dauer der Abende. Uhrzeiten werden als Minuten seit 12:00 Uhr gemittelt,
  damit 01:00 Uhr nach 23:00 Uhr liegt. Wer bis zum Ende blieb, geht mit dem Ende des Abends ein (bei einem offenen Abend die letzte
  Aktivität). Abende ohne Zeitstempel (alte Dateien) fehlen in den Uhrzeiten.

### Abende im Überblick

- In der Gesamtstatistik: **Spiele pro Abend** (Ø, wenigste und meiste, je mit Datum), **Dauer eines Abends** (Ø, kürzester und längster, je mit
  Datum), **Spiele pro Stunde** (alle Abende mit bekannter Dauer zusammen) und eine Tabelle mit Datum, Spielen und Dauer jedes Abends.
- Die Dauer geht vom Beginn bis zum Ende, bei einem Abend ohne Ende bis zur letzten Aktivität. Abende ohne Beginn (alte Dateien) zählen bei
  den Spielen mit ("ohne Datum"), aber nicht bei der Dauer. Bei gleichen Werten gilt der erste Abend.

### Keiner spielt: Zusammenwerfen und Stock

- Das **Regelwerk** (`RuleSet.no_game`, `Table.no_game_rule`) wählt, was passiert, wenn keiner spielen will: **Ramsch** (wie bisher, eine Spielart),
  **Zusammenwerfen** (der Nächste gibt, kein Geld) oder **Stock**. Alte Dateien: Ramsch. Bei den beiden letzten gibt es am Tisch den Knopf "Keiner spielt"
  (`Table::add_no_game`). Er schiebt den Geber weiter (`next_dealer` zählt die Einträge seit dem letzten Spiel).
- **Stock:** Jeder, der am Tisch sitzt, legt die **Einlage** aus dem Tarif (`Tariff.stock_amount`, Standard 0,20 €) in die Mitte. Wer ein **Sauspiel** ansagt und
  **gewinnt**, bekommt den ganzen Stock, je zur Hälfte an sich und den Mitspieler (bei ungeradem Betrag bekommt der Spieler den größeren Anteil), zusätzlich zum
  normalen Preis. **Verliert** die Spielerpartei, muss sie den Stock **doppeln**: beide zahlen zusammen noch einmal so viel ein, wie darin liegt (je die Hälfte), der Stock
  verdoppelt sich. Das lässt sich im Tarif abschalten (`stock_doubling`, alte Tarife: an). Alle anderen Spielarten (auch die Hochzeit) lassen den Stock unberührt.
- Gespeichert wird in `Table.stock_moves` (`StockMove`: Art, Änderung des Stocks, Auszahlung je Sitzplatz, `after_games`). Einlagen sind eingetragen,
  Auszahlungen und Verdopplungen werden aus den Spielen **abgeleitet** und bei jedem Eintragen, Bearbeiten, Löschen und Rückgängig neu berechnet (`rebuild_stock`).
  Die Beträge stecken in Kontostand, Verlauf und `PlayerStats.balance`, aber nicht in den Spielen selbst (bestes/schlechtestes Spiel bleiben unberührt).
  In der Ereignis-CSV stehen sie als Zeilen (Spalte `Stock_Cent` = Änderung des Stocks).

### Geldstand über alle Abende, Startwert, Nachträge

- Die **Bibliothek** merkt sich je Spieler einen **Startwert** (`start_balances`) und beliebige **Nachträge** (`Adjustment`: Spieler, Betrag, Zeitpunkt, Notiz), zum Beispiel
  für Spiele woanders. `Library::account` rechnet je gespeichertem Spieler Startwert + Nachträge + alle Abende (Archiv und laufender, ohne Testabende, mit Strafen und
  Stock) zusammen. Spieler werden über den Namen zugeordnet (Groß-/Kleinschreibung egal). Beim Umbenennen eines Spielers wandern Startwert und Nachträge mit, frühere Abende
  behalten aber den alten Namen und zählen dann nicht mehr für ihn.
- Zu sehen ist es oben auf der ersten Seite ("Neuer Abend"). Die Daten liegen in `schafkopf-bibliothek.json` und sind in der Sicherung enthalten.

### Stock aufteilen, Zusammenwerfen getrennt zählen, CSV

- **Stock aufteilen** (`Table::split_stock`, `StockMoveKind::Split`): Der ganze Stock geht zu gleichen Teilen an alle, die am Tisch sitzen (`StockMove.seats`); ein Rest in Cent geht der Reihe nach an die ersten Sitzplätze. Wie die Einlagen von Hand eingetragen und zurücknehmbar (`StockMoveKind::is_manual`), der Betrag wird in `rebuild_stock` neu berechnet. Der Geber ändert sich nicht.
- **Zusammenwerfen vs. Einzahlen:** Ein "Keiner spielt" ohne Einlage (`StockMove::is_reshuffle`) zählt in `StockStats.reshuffles`, eines mit Einlage in `deposit_rounds`; `no_game_rounds` bleibt die Summe.
- **Gegeben:** Wer bei "Keiner spielt" gegeben hat, steht in `Table::pass_dealers` (Anwesenheit aus dem folgenden Spiel); `PlayerStats.deals_without_game`. Die Zahl "Gegeben" in der Spielerkarte ist Spiele plus diese; am Tisch gilt Spiele + `no_game_rounds`.
- **Bewegtes Geld** (`TableStats.turnover`) enthält jetzt auch die Gewinne aus dem Stock (Auszahlung, Aufteilen).
- **CSV der Spiele:** Neue Spalten `Stock_Bewegung` (Auszahlung/Verdopplung durch dieses Spiel), `Stock_Cent` (Stock nach dem Spiel) und je Spieler `<Name> Stock_Cent` (sein Stock-Ergebnis bis dahin, inkl. Einlagen). `Tarif` und `Regelwerk` kommen aus `meta::tariff_labels` und `rule_set_labels`, also wie in der Statistik ("Eigener Tarif" statt leer).
- **Doppler-Tabelle:** `DoublingTable` (Tabelle plus Kuchen) beim Abend und in der Gesamtstatistik.

### Statistik zum Stock

- `TableStats.stock` (`StockStats`): Zahl der Runden "Keiner spielt", **höchster Stand**, eingelegt, durch Doppeln dazugekommen, **hineingeflossen** insgesamt, ausgezahlt (mit Zahl), Rest am Ende. In der Gesamtstatistik: Summen, der Höchststand ist der höchste aller Abende.
- `PlayerStats.stock` / `OverallPlayer.stock` (`PlayerStock`): je Spieler eingelegt, fürs Doppeln gezahlt, aus dem Stock bekommen, wie oft gewonnen und gedoppelt, unterm Strich (bekommen minus gezahlt). Abschnitt "Stock" in allen drei Statistik-Bildschirmen, leer ohne Stock.

### Einrichtung übernimmt geänderte Tarife

- Die Einrichtung ("Neuer Abend") merkt sich, von welchem Bibliothekstarif ihre Felder stammen (`SetupState.synced`). Wird der Tarif in der Verwaltung geändert, übernimmt sie die neuen Werte, solange der Nutzer die Felder nicht für den Abend angepasst hat (`sync_tariff`). Vorher blieben die Werte vom Start der App stehen. Wurde der gewählte Tarif gelöscht oder umbenannt, gilt der Standard.

- In der Spielerkarte (Abend und Gesamtstatistik) gibt es den Abschnitt "Stock" mit **Unterm Strich**, **Eingezahlt** und **Bekommen** (Metriken `StockNet`, `StockPaid`, `StockReceived`); ein Tipp zeigt die Rangliste. Nur Spieler, die den Stock genutzt haben, stehen darin.

### Abend mit Uhrzeit beenden, beendeten Abend wieder öffnen

- "Abend beenden" nimmt eine Uhrzeit (`hh:mm`, Ortszeit) statt "jetzt": `latest_clock_before` nimmt die jüngste solche Zeit, die nicht in der Zukunft liegt (nach Mitternacht also der Vortag). Leer heißt jetzt. Das Ende darf nicht vor `Table::last_activity` liegen. Gespeichert wird es wie bisher in `Table.ended_at`.
- Ein archivierter Abend lässt sich mit `storage::reopen_past` wieder zum laufenden machen (erst als laufender gespeichert, dann aus dem Archiv gelöscht). `ended_at` bleibt stehen und füllt das Feld für das Ende vor; wer dieselbe Uhrzeit lässt, behält den alten Zeitpunkt (auch Tage später). Nur möglich, wenn nicht gerade ein Abend mit Spielen läuft. Danach geht alles wie am laufenden Abend, auch Spiele bearbeiten.
