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

1. Entwickleroptionen einschalten (Einstellungen, Über das Telefon, mehrmals auf die Build-Nummer tippen).
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

## Checkliste auf dem Handy

- [ ] Der Name lässt sich ändern, die Tastatur erscheint, Umlaute gehen.
- [ ] "Tisch anlegen" zeigt das Diagramm und die Legende mit vier Spielern bei 0,00 €.
- [ ] "Testspiel eintragen" bewegt die Linien, der Geldstand wird auch negativ dargestellt.
- [ ] Der Hinweis unten zeigt einen Pfad und **keinen roten Fehler**. Merk dir den Pfad.
- [ ] App komplett schließen und neu öffnen: Der Tisch ist noch da.
- [ ] Das Diagramm passt in der Breite, auch im Dunkelmodus und quer.

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
