# UI-Technik für die Android-App

Anforderungen: so viel wie möglich Rust, Graphen (Kontoverlauf, Verteilungen), Texteingabe für
Spielernamen, Bedienung mit dem Daumen.

## Empfehlung: Dioxus 0.7

Die Oberfläche wird in Rust geschrieben (RSX), unter Android in einer WebView gezeigt. Graphen
entstehen als **SVG, das Rust selbst erzeugt**, direkt im Oberflächencode. Ein JavaScript-Framework
ist nicht nötig. Der Kern `schafkopf-core` wird unverändert eingebunden.

Warum:

- **Graphen sind in der WebView einfach.** SVG skaliert, kennt Touch-Ereignisse und Tooltips und
  folgt dem hellen/dunklen Modus per CSS. Für einen Linienverlauf, Balken und Kreise reichen wenige
  Zeilen Rust; für mehr gibt es die `plotters` mit SVG-Ausgabe.
- **Texteingabe und Tastatur** sind die von Android, weil es normale Eingabefelder im Browser sind.
- **Rust-Anteil** ist am höchsten: Oberfläche, Zustand und Logik in einer Sprache, ohne JS-Brücke.
- Dioxus 0.7 nennt Android und iOS ausdrücklich als Ziele und bringt Hot-Patching (Änderungen
  erscheinen auf dem Gerät ohne Neubau) mit.

Risiken, die wir beim Prototyp prüfen:

- Android ist bei Dioxus die jüngste Plattform; die Docs der Version 0.5 nennen Mobile noch das
  am wenigsten ausgereifte Ziel, 0.6 brachte `dx serve` für Android. Wie stabil es in 0.7.x
  wirklich ist, haben wir noch nicht auf einem Gerät gesehen.
- Der Build braucht Android SDK und NDK und läuft auf dem Host, nicht in der VM.

## Alternativen

| | Oberfläche | Graphen | Android-Stand | Urteil |
| --- | --- | --- | --- | --- |
| **Slint** | nativ, eigenes Rendering | kein Diagramm-Widget; man müsste Linien und Balken selbst zeichnen (`Path`) | seit Version 1.5 unterstützt, nur mit Rust | Gute Optik, aber Diagramme sind viel Handarbeit. Offene Issues zur Android-Tastatur (soft keyboard, Tastaturlayouts), je nach Android-Version. |
| **egui** | Eigenes Rendering, Werkzeugkasten-Look | `egui_plot` ist gut | Android läuft, die System-Tastatur ist in egui/winit laut Crate `egui_keyboard` nicht nativ umgesetzt | Texteingabe auf dem Handy ist der Knackpunkt. |
| **Tauri 2 + Leptos** | WebView, Oberfläche in Rust per WASM | wie bei Dioxus | Tauri 2 unterstützt Android | Vergleichbar mit Dioxus, aber zwei Build-Stufen (WASM plus App). Möglich als Rückfall. |
| **Tauri + JS/TS** | WebView mit JavaScript | beste Auswahl (Chart.js, ECharts) | wie oben | Nur noch der Kern in Rust. Widerspricht dem Ziel. |

## Quellen und Vorbehalt

Die Einschätzung stützt sich auf Suchergebnisse zu den Release-Notes von Dioxus 0.7, den
Android-Seiten von Slint samt GitHub-Issues und den Docs von `egui_keyboard`. Die Seiten selbst
konnte ich aus der VM nicht abrufen, und zu Dioxus 0.7 auf Android fand ich keinen Erfahrungsbericht.
Deshalb gilt die Empfehlung unter dem Vorbehalt: **erst ein kleiner Prototyp auf deinem Gerät**
(Eingabefeld, ein SVG-Diagramm, Speichern einer Datei), bevor wir die ganze App darauf bauen.

- [Dioxus 0.7 Release](https://dioxuslabs.com/blog/release-070/)
- [Dioxus 0.7.0 auf GitHub](https://github.com/DioxusLabs/dioxus/releases/tag/v0.7.0)
- [Slint auf Android](https://docs.slint.dev/latest/docs/slint/guide/platforms/android)
- [Slint-Issue: Tastatur auf Android 12](https://github.com/slint-ui/slint/issues/11791)
- [egui_keyboard](https://docs.rs/egui_keyboard)
- [Plotters](https://github.com/plotters-rs/plotters)
