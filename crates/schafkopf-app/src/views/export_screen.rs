//! CSV exportieren: in den Ordner Downloads speichern oder den Text kopieren.

use dioxus::document::eval;
use dioxus::prelude::*;
use schafkopf_core::Table;

use super::common::{Chip, Notice};
use crate::export::{self, CsvFormat};
use crate::{now_secs, Screen};

#[component]
pub fn ExportScreen(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
    back: Screen,
) -> Element {
    let mut screen = screen;
    let mut notice = notice;
    let mut format = use_signal(|| CsvFormat::Semicolon);

    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    let file_name = export::file_name(t, now_secs());
    let csv = format().render(t);
    let shown = csv.trim_start_matches('\u{feff}').to_owned();
    let games = t.games().len();

    let save = {
        let file_name = file_name.clone();
        let csv = csv.clone();
        move |_| match export::save_csv(&file_name, &csv) {
            Ok(place) => notice.set(Some(Notice::ok(format!("Gespeichert: {place}")))),
            Err(e) => notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}. Du kannst den Text unten kopieren."
            )))),
        }
    };

    let log_name = export::log_file_name(t, now_secs());
    let log_events = t.events.len();
    let save_log = {
        let log = format().render_log(t);
        let log_name = log_name.clone();
        move |_| match export::save_csv(&log_name, &log) {
            Ok(place) => notice.set(Some(Notice::ok(format!("Gespeichert: {place}")))),
            Err(e) => notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}"
            )))),
        }
    };
    let place = if cfg!(target_os = "android") {
        "im Ordner Downloads deines Handys. Von dort kannst du sie in der Dateien-App öffnen oder teilen."
    } else {
        "in den Ordner Downloads (sonst in den Datenordner der App)."
    };

    rsx! {
        h2 { "CSV exportieren" }
        p { "Die {games} Spiele dieses Abends als Tabelle, gespeichert {place}" }
        p { class: "hint", "Dateiname: {file_name}" }

        section {
            h3 { "Format" }
            div { class: "chips",
                for f in [CsvFormat::Semicolon, CsvFormat::Comma] {
                    Chip {
                        key: "{f.label()}",
                        label: f.label().to_string(),
                        on: format() == f,
                        onclick: move |_| format.set(f),
                    }
                }
            }
            p { class: "hint",
                "Das deutsche Excel braucht das Semikolon. Google Tabellen und LibreOffice erkennen beides. Beide Dateien sind UTF-8 mit Kennung, damit Umlaute stimmen. Beträge stehen in Cent."
            }
        }

        div { class: "actions",
            button { class: "primary big", r#type: "button", onclick: save, "In Downloads speichern" }
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(back), "Zurück" }
        }

        section {
            h3 { "Protokoll des Abends" }
            p { class: "hint",
                "Wann der Abend begann und endete und wann jemand kam, ging oder sich umsetzte ({log_events} Einträge), als zweite Tabelle im selben Format. Dateiname: {log_name}"
            }
            div { class: "actions",
                button { class: "secondary", r#type: "button", onclick: save_log, "Protokoll in Downloads speichern" }
            }
        }

        details {
            summary { "Text anzeigen und kopieren" }
            p { class: "hint", "Zum Einfügen in eine Nachricht oder Notiz: Auf den Text tippen und halten, alles markieren, kopieren." }
            textarea { id: "csv-text", readonly: true, rows: "8", value: "{shown}" }
            button {
                class: "secondary",
                r#type: "button",
                onclick: move |_| {
                    let _ = eval(
                        "const t = document.getElementById('csv-text'); if (t) { t.focus(); t.select(); }",
                    );
                },
                "Alles markieren"
            }
        }
    }
}
