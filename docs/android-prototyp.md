# Android-Prototyp ausprobieren

Der Prototyp (`crates/schafkopf-app`) soll auf dem Handy drei Dinge klären, bevor wir die ganze
App bauen:

1. Funktioniert die **Texteingabe** (Tastatur, Umlaute) in der Dioxus-WebView?
2. Wird das **SVG-Diagramm** gut dargestellt (Breite, hell/dunkel)?
3. Lässt sich **JSON und CSV speichern** und nach einem Neustart wieder laden?

Er legt einen Tisch aus Namen an, trägt auf Knopfdruck zufällige Testspiele (Wenz) ein, zeichnet den
Geldstand und schreibt `schafkopf-tisch.json` und `schafkopf-spiele.csv`. Beim Start lädt er den
gespeicherten Tisch wieder.

## Was geprüft ist und was nicht

In der VM geprüft: Kern, Diagramm-Erzeugung und Speicherlogik haben Tests, und die App **übersetzt für
den Desktop** (`--features desktop`). Das Flake wird ausgewertet.

**Nicht geprüft:** der Android-Build selbst, der Start auf einem Gerät und das Verhalten der WebView.
Die Android-Umgebung (SDK, NDK, Gradle) ist am wahrscheinlichsten die Stelle, an der beim ersten
Versuch etwas nachgebessert werden muss. Schick mir dann die Fehlermeldung.

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

Der Prototyp soll drei Fragen beantworten: Funktioniert die **Eingabe**, sieht das **Diagramm** gut aus,
und bleiben die **Daten** gespeichert? Alles andere ist Zugabe. Notiere bei jedem Punkt, ob es klappt,
und schreib mir, was nicht passt.

### 1. Eingabe und Tastatur
- [ ] Das Namensfeld lässt sich antippen, die Tastatur erscheint, und das Feld wird nicht von ihr verdeckt.
- [ ] Umlaute und Sonderzeichen gehen: `Müller, Jürgen, Ärmel, Zoë`.
- [ ] Mit 4 Namen entsteht ein Tisch. Teste auch **2** Namen und **10** Namen.
- [ ] **Fehlerfälle** zeigen einen roten Hinweis, ohne dass die App abstürzt: 1 Name, 11 Namen, ein
      leerer Name (`Anna,,Ben,Cem`), ein doppelter Name (`Anna, Anna, Ben, Cem`).

### 2. Diagramm und Legende
- [ ] Direkt nach dem Anlegen steht in der Legende jeder Name mit eigener Farbe und `0,00 €`.
- [ ] Nach "Testspiel eintragen" bewegen sich die Linien. Trag etwa 20 Spiele ein.
- [ ] Negative Werte liegen **unter der dicken Nulllinie**, die Achse zeigt `−1,20` mit Komma und Minus.
- [ ] Die Endpunkte der Linien passen zu den Beträgen in der Legende.
- [ ] Die Beträge in der Legende ergeben zusammen **0,00 €** (rechne nach). Das muss immer stimmen.
- [ ] Bei **10 Spielern** sind die Linien und Farben noch zu unterscheiden. Welche sind schwer zu trennen?
- [ ] "Nächster Geber" wandert nach jedem Spiel zum nächsten Namen in deiner Reihenfolge.

### 3. Darstellung
- [ ] Hochformat und Querformat sehen brauchbar aus, das Diagramm passt in die Breite.
- [ ] Dunkelmodus (Systemeinstellung) ist lesbar: Text, Achsen, Linien, Eingabefeld.
- [ ] Größere Systemschrift (Einstellungen, Anzeige, Schriftgröße) bricht nichts.
- [ ] Nichts verschwindet unter Statusleiste, Kamera-Aussparung oder Gestenleiste unten.
- [ ] Die Knöpfe sind mit dem Daumen gut zu treffen.

### 4. Speichern und Laden (das Wichtigste)
- [ ] Unten steht nach dem Anlegen und nach jedem Spiel ein **Pfad und kein roter Fehler**. Schick mir
      den Pfad.
- [ ] App **komplett schließen** (in der Übersicht wegwischen) und neu öffnen: Tisch, Spiele und
      Diagramm sind noch da.
- [ ] Auch nach einem **Neustart des Handys** sind die Daten noch da.
- [ ] "Neuer Tisch" legt einen leeren Tisch an. Danach überschreibt der nächste Tisch die alten
      Dateien. Das ist im Prototyp so gewollt, ein Archiv gibt es noch nicht.

Die CSV-Datei kannst du ohne `adb` (also ohne Kabel oder Wireless Debugging) noch nicht ansehen, sie
liegt im privaten App-Ordner. Ein
Export zum Teilen ist eine Aufgabe für die richtige App. Geprüft wird hier nur, dass kein Fehler kommt.

### 5. Stabilität
- [ ] Der Start dauert nur wenige Sekunden und die App stürzt nicht ab.
- [ ] Zwischen Apps wechseln und zurückkehren behält den Zustand.
- [ ] Das Gerät drehen, während Text im Feld steht, verliert den Text nicht.
- [ ] Im **Flugmodus** verhält sich alles genauso. Die App braucht kein Netz.

### Was du mir mitteilst
Handymodell und Android-Version, was nicht geklappt hat (gern mit Screenshot) und was dich beim
Benutzen gestört hat, auch wenn es nur ein Gefühl ist ("zu klein", "zu träge").

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
