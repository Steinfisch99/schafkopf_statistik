# Android-App bauen und testen

Die App (`crates/schafkopf-app`) führt durch den Abend: Abend einrichten, Spiele eintragen, Geldstand
ansehen, Tarif ändern. Diese Anleitung zeigt, wie man sie auf dem Handy installiert und was man dort
prüfen sollte.

## Was geprüft ist und was nicht

In der VM geprüft: Kern, Diagramm, Eingabelogik, Speichern und Export haben Tests (138). Außerdem lief die App
am Desktop in einem virtuellen Bildschirm mit Handybreite, und ich habe sie durchgeklickt: Abend
einrichten, ein Spiel mit Klopfen, Kontra, Re, Schneider und Laufenden eintragen (der Preis stimmt),
Tarif ändern (alte Spiele bleiben unverändert), Rückgängig, Neuer Abend mit Archiv und der CSV-Export
in beide Formate (Excel mit Semikolon und BOM, allgemein mit Komma).

Auf dem Handy (Android 16) funktionierten beim ersten Prototyp der Build, die Installation, die
Tastatur, das Diagramm und das Laden nach dem Schließen. **Neu und noch nicht auf dem Handy
geprüft:** die Eingabemasken in echter Größe, die Bedienung mit dem Daumen, Dunkelmodus und große
Schrift in den neuen Bildschirmen. **Gar nicht geprüft** ist das Speichern in den Ordner Downloads auf
Android, weil der JNI-Teil dort nur auf dem Handy läuft.

## Einmalig: Handy vorbereiten

1. Entwickleroptionen einschalten (Einstellungen, Über das Telefon, mehrmals auf die Build-Nummer tippen). Das ist nur für den Weg mit `adb` nötig, nicht für die APK.
2. **USB-Debugging** einschalten und das Handy per USB anschließen.
3. Die Abfrage "USB-Debugging zulassen" auf dem Handy bestätigen.

## Auf dem Host

Der erste Aufruf lädt etwa 1,1 GiB Android SDK und NDK und braucht entsprechend Zeit. Danach sind
sie im Nix-Speicher.

Android-Umgebung öffnen und prüfen, ob das Handy erkannt wird:

```sh
cd ~/git-repos/sandbox/schafkopf
nix develop .#android
adb devices
```

In der Android-Shell die App bauen, installieren und starten:

```sh
cd crates/schafkopf-app
dx serve --platform android
```

Änderungen am Code erscheinen danach per Hot-Patching auf dem Handy.

### Schneller Test am Desktop (ohne Handy)

Prüft nur die Oberfläche, nicht Android. Die WebView braucht unter Nix eventuell Nacharbeit.

```sh
cd ~/git-repos/sandbox/schafkopf
nix develop .#desktop -c cargo run -p schafkopf-app --features desktop
```


## Ohne Kabel und ohne WLAN-Verbindung zum PC: APK installieren

Wenn weder USB-Kabel noch stabiles WLAN zwischen PC und Handy möglich sind, bauen wir eine APK und
bringen sie als Datei aufs Handy. Nachteil: Es gibt kein Hot-Patching, jede Änderung heißt neu bauen
und neu übertragen. Für den ersten Prototyp-Test reicht das.

Ein Debug-Build wird von Gradle automatisch mit einem Debug-Schlüssel signiert und lässt sich
deshalb ohne eigenes Zertifikat installieren. Er ist nur für den eigenen Gebrauch gedacht.

**Kurzfassung mit Skript:** `scripts/build-apk.sh` macht alle folgenden Schritte (bauen, APK suchen,
prüfen, dass sie für das Handy gebaut ist, und mit `wormhole-rs` senden):

```sh
cd ~/git-repos/sandbox/schafkopf
./scripts/build-apk.sh
```

Mit `--no-send` baut es nur, mit `--send-only` schickt es die zuletzt gebaute APK noch einmal, ohne zu
bauen. Ist `wormhole-rs` nicht installiert, holt das Skript es über Nix. Auf dem Handy gibst du den
Code ein, den `wormhole-rs` anzeigt. Die Debug-APK ist groß (um 200 MB), die Übertragung dauert also.

Die einzelnen Schritte, falls du sie von Hand machen willst:

Auf dem Host bauen und die APK suchen:

```sh
cd ~/git-repos/sandbox/schafkopf
nix develop .#android
cd crates/schafkopf-app
dx build --platform android --target aarch64-linux-android
find ../../target -name '*.apk'
```

**Wichtig ist `--target aarch64-linux-android`.** Ohne angeschlossenes Gerät baut `dx` für die
Architektur des PCs (meist `x86_64`, die von Emulatoren). Handys sind `aarch64`; eine x86-APK lässt
sich dort nicht installieren, Android meldet nur "App nicht installiert".

Der genaue Ordner hängt von der `dx`-Version ab, deshalb die Suche. Laut Issues auf GitHub liegt sie
unter `target/dx/schafkopf-app/debug/android/app/` in `build/outputs/apk/debug/app-debug.apk`.

Vor dem Übertragen prüfen, ob die APK für das Handy gebaut ist. Dort muss `lib/arm64-v8a/` stehen:

```sh
unzip -l PFAD/app-debug.apk | grep "lib/"
```

Aufs Handy bringen: Die APK in einen Cloud-Ordner legen, per E-Mail oder Messenger an dich selbst
schicken, oder auf einen USB-Stick kopieren (am Handy mit OTG-Adapter). Der Upload verträgt auch
schlechtes WLAN, weil er nicht in Echtzeit laufen muss.

Auf dem Handy:

1. Die APK-Datei antippen (Dateien-App oder Download-Benachrichtigung).
2. Android fragt, ob diese App Apps installieren darf. Das für diese eine Quelle erlauben.
3. Erscheint die Warnung von Google Play Protect, "Trotzdem installieren" wählen.
4. Die App "schafkopf-app" öffnen und die Checkliste oben durchgehen.

Wurde die App schon installiert, ersetzt eine neue APK mit gleicher Kennung die alte und behält ihre
Daten.

## Checkliste auf dem Handy

Gehe einen Abend durch, so wie ihr ihn spielt. Notiere, was nicht klappt oder sich umständlich anfühlt.

### 1. Abend einrichten
- [ ] **Beim allerersten Mal** gibt es noch keine gespeicherten Spieler: Die Plätze sind Eingabefelder. Vier Namen mit Umlauten eintragen, die Tastatur verdeckt das Feld nicht.
- [ ] Nach "Abend starten" meldet die App, dass neue Spieler gemerkt wurden. Beim **nächsten Abend** sind die Plätze Auswahllisten mit den gespeicherten Namen, die Spieler des letzten Abends sind vorbelegt.
- [ ] In der Auswahl sind schon gewählte Namen ausgegraut, "+ Neuer Spieler …" öffnet ein Eingabefeld, "▾" daneben führt zurück zur Liste.
- [ ] "+ Spieler" und "−" gehen von 2 bis 10 Spielern. Bei 2 ist "−" ausgegraut, bei 10 "+ Spieler".
- [ ] Ein leerer Name ergibt einen roten Hinweis oben, kein Absturz. Ein doppelter Name auch.
- [ ] Der erste Geber ist wählbar.
- [ ] **Regelwerk** und **Tarif** sind vorgewählt (die Standards) und lassen sich per Auswahl wechseln. Unter dem Regelwerk steht, was gespielt wird.
- [ ] "Für diesen Abend anpassen" öffnet die Tarif-Felder. Komma geht (`0,20`), "abc" ergibt einen Hinweis. Die Felder "Laufende zählen ab" heißen Sauspiel, **Wenz, Geier** und Solo.
- [ ] "Abend starten" zeigt die Tischansicht mit dem gewählten Geber und `0,00 €` für alle.

### 2. Spiel eintragen (der Hauptablauf)
- [ ] Als Geber ist der Richtige vorgewählt, bei den Mitspielenden stehen der Geber, der Spieler vor
      ihm und die zwei danach.
- [ ] "Spiel speichern" bleibt ausgegraut, bis alles gewählt ist. Der Hinweis darüber sagt, was fehlt.
- [ ] Beim Antippen von Spieler, Mitspieler, Geklopft, Kontra und Re **springt nichts** auf dem Bildschirm.
- [ ] **Gegenlauf:** Unter dem Zähler für die Laufenden gibt es einen Knopf "Gegenlauf (die Gegner hatten sie)". Er ist ohne Laufende ausgegraut, der Preis ändert sich nicht. In der Spieleliste steht dann "… Laufende (Gegenlauf)".
- [ ] **Laufende:** Beim Sauspiel, der Hochzeit und dem Farbsolo gehen sie bis **14**, beim Wenz und Geier bis 4, beim Farbwenz und Farbgeier bis 11. Wechselst du die Spielart, kürzt sich ein zu hoher Wert.
- [ ] Bei Spieler und Mitspieler ist "Kontra" ausgegraut.
- [ ] Der Preis ändert sich sofort. Rechenprobe beim Sauspiel: 0,20 €, Schneider +0,10 €, ab 3 Laufenden
      je +0,10 €, jedes Klopfen, Kontra und Re verdoppelt.
- [ ] Die Auszahlung darunter ergibt zusammen null. Beim Solo (Wenz, Farbsolo) bekommt der Spieler das
      Dreifache.
- [ ] Ein verlangter Preis (`0,80`) ersetzt die Rechnung. "abc" ergibt einen Hinweis.
- [ ] Die Spielart-Auswahl zeigt nur die Spielarten des gewählten Regelwerks.
- [ ] **Geier:** Laufende zählen schon ab 2 (wie beim Wenz) und gehen in den Preis ein. Beim Farbsolo erst ab 3.
- [ ] **Ramsch:** Ohne Haken wählst du den **Verlierer**. Mit "Durchmarsch" wählst du, wer ihn gemacht hat. "Gewonnen/Verloren" gibt es beim Ramsch nicht.
- [ ] Wenz, Farbsolo und Bettel zeigen nur passende Felder (Farbe, Tout, kein Mitspieler).
- [ ] Nach "Spiel speichern" siehst du die Tischansicht, das Diagramm hat sich bewegt, und der
      Geber ist um einen Platz weiter.

### 3. Tischansicht
- [ ] "Spiele" aufklappen zeigt Spielart, wer gespielt hat, Ergebnis und den Betrag des Spielers.
- [ ] "Tarif ändern" und speichern: neue Spiele rechnen mit dem neuen Tarif, ältere Spiele behalten
      ihren Betrag.
- [ ] "Letztes Spiel rückgängig" fragt nach ("Ja, rückgängig"), "Abbrechen" bricht ab. Die Knöpfe
      springen dabei nicht.
- [ ] "Neuer Abend" fragt nach, beginnt dann bei der Einrichtung, und dort geht "Abbrechen" zurück.

### 4. Daten
- [ ] App komplett schließen und neu öffnen: Abend, Spiele und Diagramm sind noch da.
- [ ] Auch nach einem Neustart des Handys.
- [ ] Nach einem neuen Abend ist der alte weg, aber **archiviert** (im App-Ordner, ohne `adb` nicht
      einsehbar). Den Export gibt es als eigenen Punkt unten.

### 5. CSV-Export
Das ist die am wenigsten geprüfte Stelle: Der Weg über Android (`MediaStore`, per JNI) lässt sich nur
auf dem Handy ausprobieren. Der Export am Desktop ist getestet.

- [ ] Tischansicht, "CSV exportieren": Dateiname mit dem Datum des Abends, Format "Semikolon" ist
      vorgewählt.
- [ ] "In Downloads speichern" zeigt unten eine Meldung `Gespeichert: Downloads/schafkopf-….csv` und
      **keinen roten Fehler**. Ist es ein Fehler, schick mir den Wortlaut.
- [ ] Dateien-App, Ordner Downloads: Die Datei liegt dort.
- [ ] Die Datei lässt sich öffnen (Google Tabellen, Excel, Dateien-App). **Umlaute stimmen**, die
      Spalten sind getrennt, jede Zeile ist ein Spiel. Beträge stehen in Cent.
- [ ] Ein zweites Mal speichern überschreibt nichts, Android hängt eine Nummer an.
- [ ] Format "Komma (international)" speichert auch. **Umlaute stimmen in beiden Formaten.**
- [ ] "Text anzeigen und kopieren": Text markieren, kopieren und in einer Notiz einfügen geht.
- [ ] Die Meldung unten verdeckt nichts Wichtiges, und beim Erscheinen springt nichts.

### 6. Diagramm bedienen
Die Rechnung hinter den Gesten ist getestet, und am Desktop habe ich Tippen, Ziehen und die Knöpfe mit
der Maus ausprobiert. **Der Zwei-Finger-Zoom und das Doppeltippen mit dem Finger sind nur simuliert**,
das musst du auf dem Handy prüfen.

- [ ] **Tippen** auf eine Stelle markiert das nächste Spiel: gestrichelte Linie, ein Punkt je Spieler,
      darüber die Beschreibung des Spiels, in der Legende "Stand nach Spiel N".
- [ ] Nochmal auf dasselbe Spiel tippen oder "Auswahl aufheben" nimmt die Markierung weg.
- [ ] **Mit einem Finger nach links oder rechts ziehen** verschiebt den Ausschnitt. Nach oben oder
      unten wischen scrollt weiterhin die ganze Seite.
- [ ] **Zwei Finger auseinander** zoomen hinein, **zusammen** wieder heraus. Die Stelle zwischen den
      Fingern bleibt, wo sie ist. Weiter als drei Spiele geht es nicht hinein.
- [ ] Nach dem Zoomen **einen Finger anheben**, mit dem anderen weiterziehen: Der Ausschnitt springt nicht.
- [ ] **Doppeltippen** zeigt wieder den ganzen Verlauf und nimmt die Markierung weg.
- [ ] Die Knöpfe "Alles" und "Letzte 20 / 10 / 5" funktionieren (sie erscheinen nur, wenn es so viele
      Spiele gibt).
- [ ] Beim Hineinzoomen passt sich die Hochachse an, die Zahlen bleiben gut lesbar.
- [ ] Nach dem **Drehen des Handys** trifft ein Tippen das richtige Spiel.
- [ ] Nach einem neuen Spiel zeigt das Diagramm wieder den ganzen Verlauf.
- [ ] Der Zoom läuft flüssig, auch bei vielen Spielen.

### 7. Statistik
Am Desktop mit einem Abend von 14 Spielen durchgeklickt, die Zahlen untereinander geprüft (zum
Beispiel ergeben die Siege nach Rolle die Siegquote in der Rangliste). Zu den Zahlen gibt es jetzt
Grafiken, die Zahlen stehen aber immer daneben.

- [ ] Tischansicht, "Statistik" öffnet den Bildschirm, "Zurück" geht zurück.
- [ ] **Rangliste:** Balken für den Geldstand (Gewinn nach rechts, Verlust nach links) und darunter die
      Tabelle mit Reihenfolge, Beträgen, Spielen und Siegquote.
- [ ] **Dieser Abend:** Spiele, bewegtes Geld, teuerstes Spiel mit Beschreibung. Ein **Ring** zeigt, wie
      oft die Spielerpartei gewann. Darunter Balken für die Anteile mit Klopfen, Kontra, Re, Schneider,
      Schwarz und korrigiertem Preis.
- [ ] **Spielarten:** Ring mit der Verteilung, Balken (grün gewonnen, orange verloren) und die Tabelle.
- [ ] **Lauf und Gegenlauf:** Ring über alle Spielarten, Balken je Spielart (blau Lauf, gelb Gegenlauf, grau ohne) und eine Tabelle mit Spielen, Lauf, Gegenlauf und durchschnittlicher Zahl Laufende. Die Zahlen passen zu dem, was du eingetragen hast.
- [ ] In der Karte jedes Spielers stehen "Lauf auf der eigenen Seite" und "auf der Gegenseite".
- [ ] **Gerufene Sau** und **Wer mit wem** haben Balken. Sie sind untereinander vergleichbar, ein längerer
      Balken heißt mehr Spiele.
- [ ] **Die Spieler im Detail:** Antippen klappt einen Spieler auf.
  - [ ] Oben ein **Ring mit Siegen und Niederlagen** und der Siegquote in der Mitte.
  - [ ] Bilanz, Siege nach Rolle (Spieler, Mitspieler, Gegenspieler) und Ansagen als Balken mit den Zahlen.
  - [ ] **Serien** mit einem Kästchen je Spiel (+ Sieg, − Niederlage, gestrichelt ausgesetzt).
  - [ ] Nach Spielart: Balken für Gewinn und Verlust, dazu die Tabelle.
  - [ ] Nach Platz: Balken und Tabelle.
- [ ] Alles stimmt mit deinem Gefühl vom Abend überein. Sind die Farben (grün/orange) gut zu unterscheiden?
- [ ] Die Balken und Tabellen passen in die Breite (auch bei langen Namen, im Querformat, bei großer Schrift).
- [ ] Ohne Spiele steht ein Hinweis statt leerer Tabellen.

### 8. Spiele bearbeiten
Am Desktop durchgeklickt: Ein Spiel von "gewonnen" auf "verloren" geändert, danach stimmten
Kontostände, Diagramm und die CSV-Zeile.

- [ ] In der Tischansicht unter "Spiele" hat jedes Spiel einen Link "Bearbeiten".
- [ ] Das Formular ist mit dem Spiel vorbelegt (Spielart, Rollen, Ansagen, Laufende, Preis).
- [ ] Eine Änderung (zum Beispiel Verloren statt Gewonnen) ändert die Vorschau sofort, "Änderungen
      speichern" übernimmt sie, Diagramm, Legende und Statistik passen sich an.
- [ ] Wurde der Tarif zwischendurch geändert, rechnet das bearbeitete Spiel **mit dem Tarif von damals**.
- [ ] "Spiel löschen" fragt nach ("Ja, löschen"). Danach ist das Spiel weg, alle übrigen bleiben.
- [ ] Die Spielart des bearbeiteten Spiels steht in der Auswahl, auch wenn sie im Regelwerk fehlt.

### 9. Regelwerke und Tarife
Am Desktop durchgeklickt: Regelwerk angelegt, zum Standard gemacht, mit ihm einen Abend begonnen,
Tarif unter einem Namen gespeichert und übernommen. Die Dateien habe ich nachgesehen.

- [ ] "Spieler, Regelwerke und Tarife verwalten" (bei der Einrichtung und unter "Regeln und Tarif") zeigt oben die **Spieler**, dann
      "Alle Spielarten" und "Unsere Runde" (Standard) sowie den Tarif "20/50" (Standard).
- [ ] Ein **neues Regelwerk** lässt sich anlegen (Name, Spielarten antippen, "Anlegen") und erscheint
      in der Liste. Ein leerer oder doppelter Name und ein Regelwerk ohne Spielart ergeben einen roten Hinweis.
- [ ] Ein bestehendes lässt sich ändern und umbenennen, "Als Standard" macht es zum Standard, "Löschen"
      fragt nach. Der letzte Eintrag lässt sich nicht löschen.
- [ ] Das gilt genauso für **Tarife**.
- [ ] **Spieler:** Ein neuer Name lässt sich anlegen, ein bestehender umbenennen oder löschen (mit Rückfrage). Ein leerer oder doppelter Name (auch in anderer Schreibweise) ergibt einen roten Hinweis. Schon gespielte Abende behalten ihre Namen.
- [ ] Bei der Einrichtung ist das Standard-Regelwerk vorgewählt. Die Namen, die du schon getippt hast,
      sind noch da, wenn du aus der Verwaltung zurückkommst.
- [ ] In der Tischansicht steht "Regelwerk: … · Tarif: …".
- [ ] "Regeln und Tarif": Ein anderes Regelwerk gilt ab dem nächsten Spiel. Einen gespeicherten Tarif als
      Vorlage laden, anpassen und "Für diesen Abend übernehmen". "Als Tarif speichern" legt ihn unter
      einem Namen in der Bibliothek ab.
- [ ] Ein laufender Abend ändert sich nicht, wenn du in der Verwaltung etwas änderst.
- [ ] Nach dem Schließen und Öffnen der App sind Regelwerke, Tarife und die Standards noch da.

### 10. Zurück-Taste
Im Browser (WebKit) mit der Zurück-Maustaste durchgeklickt: Von Statistik, Regeln und Tarif und dem
Eintragen führt sie in die Übersicht, mehrfach hintereinander und nach dem App-Knopf "Zurück". Auf
einem Handy ist das noch nicht ausprobiert.

- [ ] Beim **Eintragen eines Spiels** führt die Zurück-Taste (oder Zurück-Geste) zur Übersicht mit dem
      Diagramm, die App wandert nicht in den Hintergrund.
- [ ] Dasselbe bei Statistik, "Regeln und Tarif", "CSV exportieren" und beim Bearbeiten eines Spiels.
- [ ] In der Verwaltung der Regelwerke führt sie zu "Regeln und Tarif" bzw. zur Einrichtung.
- [ ] Auf der **Übersicht** selbst geht die App wie gewohnt in den Hintergrund.
- [ ] Erst "Zurück" in der App antippen, dann die Zurück-Taste: Die Übersicht bleibt, nichts Seltsames.
- [ ] Mehrmals hin und zurück (Statistik, Zurück-Taste, Statistik, Zurück-Taste) geht jedes Mal.

### 11. Darstellung
- [ ] Alles ist mit dem Daumen gut zu treffen.
- [ ] Hochformat und Querformat, Dunkelmodus und größere Systemschrift gehen.
- [ ] Nichts verschwindet unter Statusleiste, Kamera-Aussparung oder Gestenleiste.
- [ ] Im Flugmodus verhält sich alles genauso.

### Was du mir mitteilst
Handymodell und Android-Version, was nicht geklappt hat (gern mit Screenshot) und was dich beim
Benutzen gestört hat, auch wenn es nur ein Gefühl ist ("zu klein", "zu umständlich").

Die gespeicherten Dateien lassen sich bei einem Debug-Build so ansehen:

```sh
adb shell run-as de.schafkopf.statistik ls files
adb exec-out run-as de.schafkopf.statistik cat files/schafkopf-spiele.csv
```

## Wenn etwas nicht geht

- **Speichern schlägt fehl:** Der Pfad `/data/data/de.schafkopf.statistik/files` stammt aus der
  App-Kennung in `Dioxus.toml` und ist eine Annahme. Die Fehlermeldung steht im roten Hinweis.
- **`dx` findet das SDK oder NDK nicht:** Der `shellHook` in `flake.nix` setzt `ANDROID_HOME` und
  `NDK_HOME` und gibt sie beim Öffnen der Shell aus. Schick mir diese Zeilen.
- **Gradle meckert über `aapt2`:** `GRADLE_OPTS` in der Flake leitet es auf das aapt2 des Nix-SDK um.
