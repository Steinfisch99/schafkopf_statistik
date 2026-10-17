# Android-App bauen und testen

Die App (`crates/schafkopf-app`) führt durch den Abend: Abend einrichten, Spiele eintragen, Geldstand
ansehen, Tarif ändern. Diese Anleitung zeigt, wie man sie auf dem Handy installiert und was man dort
prüfen sollte.

## Was geprüft ist und was nicht

In der VM geprüft: Kern, Diagramm, Eingabelogik und Speichern haben Tests (43). Außerdem lief die App
am Desktop in einem virtuellen Bildschirm mit Handybreite, und ich habe sie durchgeklickt: Abend
einrichten, ein Spiel mit Klopfen, Kontra, Re, Schneider und Laufenden eintragen (der Preis stimmt),
Tarif ändern (alte Spiele bleiben unverändert), Rückgängig und Neuer Abend mit Archiv.

Auf dem Handy (Android 16) funktionierten beim ersten Prototyp der Build, die Installation, die
Tastatur, das Diagramm und das Laden nach dem Schließen. **Neu und noch nicht auf dem Handy
geprüft:** die Eingabemasken in echter Größe, die Bedienung mit dem Daumen, Dunkelmodus und große
Schrift in den neuen Bildschirmen.

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
- [ ] Vier Namen mit Umlauten eintragen, die Tastatur verdeckt das Feld nicht.
- [ ] "+ Spieler" und "−" gehen von 2 bis 10 Spielern. Bei 2 ist "−" ausgegraut, bei 10 "+ Spieler".
- [ ] Ein leerer Name ergibt einen roten Hinweis oben, kein Absturz. Ein doppelter Name auch.
- [ ] Der erste Geber ist wählbar.
- [ ] Der Tarif lässt sich mit Komma eingeben (`0,20`). "abc" ergibt einen Hinweis.
- [ ] "Abend starten" zeigt die Tischansicht mit dem gewählten Geber und `0,00 €` für alle.

### 2. Spiel eintragen (der Hauptablauf)
- [ ] Als Geber ist der Richtige vorgewählt, bei den Mitspielenden stehen der Geber, der Spieler vor
      ihm und die zwei danach.
- [ ] "Spiel speichern" bleibt ausgegraut, bis alles gewählt ist. Der Hinweis darüber sagt, was fehlt.
- [ ] Beim Antippen von Spieler, Mitspieler, Geklopft, Kontra und Re **springt nichts** auf dem Bildschirm.
- [ ] Bei Spieler und Mitspieler ist "Kontra" ausgegraut.
- [ ] Der Preis ändert sich sofort. Rechenprobe beim Sauspiel: 0,20 €, Schneider +0,10 €, ab 3 Laufenden
      je +0,10 €, jedes Klopfen, Kontra und Re verdoppelt.
- [ ] Die Auszahlung darunter ergibt zusammen null. Beim Solo (Wenz, Farbsolo) bekommt der Spieler das
      Dreifache.
- [ ] Ein verlangter Preis (`0,80`) ersetzt die Rechnung. "abc" ergibt einen Hinweis.
- [ ] Wenz, Farbsolo, Ramsch und Bettel zeigen nur passende Felder (Farbe, Tout, kein Mitspieler).
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
      einsehbar). Ein Export zum Teilen steht noch aus.

### 5. Darstellung
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
