//! Neuer Abend: Spieler, erster Geber und Tarif festlegen.

use dioxus::prelude::*;
use schafkopf_core::{Table, Tariff};

use super::common::{Notice, TariffFields, TextField};
use crate::draft::TariffDraft;
use crate::{now_secs, storage, Screen};

const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 10;

#[component]
pub fn Setup(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;
    let mut names = use_signal(|| vec![String::new(); 4]);
    let mut first_dealer = use_signal(|| 0usize);
    let tariff = use_signal(|| TariffDraft::from_tariff(&Tariff::default()));
    let running = table.read().is_some();
    let count = names.read().len();

    let start = move |_| {
        let players: Vec<String> = names.read().iter().map(|n| n.trim().to_owned()).collect();
        let built = tariff.read().to_tariff().and_then(|t| {
            Table::new("Abend", players, first_dealer(), t).map_err(|e| e.to_string())
        });
        let mut new_table = match built {
            Ok(t) => t,
            Err(e) => return notice.set(Some(Notice::error(e))),
        };
        let now = now_secs();
        new_table.created_at = Some(now as i64);
        if let Err(e) = storage::archive_current(now) {
            return notice.set(Some(Notice::error(format!(
                "Der alte Abend konnte nicht archiviert werden: {e}"
            ))));
        }
        if let Err(e) = storage::save(&new_table, None) {
            return notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}"
            ))));
        }
        table.set(Some(new_table));
        screen.set(Screen::Table);
        notice.set(Some(Notice::ok("Abend gestartet")));
    };

    rsx! {
        h2 { "Neuer Abend" }
        section {
            h3 { "Spieler in Sitzreihenfolge" }
            p { class: "hint", "Im Uhrzeigersinn, so wie ihr sitzt. Zwei bis zehn Spieler." }
            for i in 0..count {
                div { class: "row", key: "{i}",
                    TextField {
                        label: format!("Spieler {}", i + 1),
                        value: names.read()[i].clone(),
                        hint: "Name".to_string(),
                        oninput: move |v| names.write()[i] = v,
                    }
                    button {
                        class: "secondary icon",
                        r#type: "button",
                        "aria-label": "Spieler entfernen",
                        disabled: count <= MIN_PLAYERS,
                        onclick: move |_| {
                            names.write().remove(i);
                            if first_dealer() >= names.read().len() {
                                first_dealer.set(0);
                            }
                        },
                        "−"
                    }
                }
            }
            button {
                class: "secondary",
                r#type: "button",
                disabled: count >= MAX_PLAYERS,
                onclick: move |_| names.write().push(String::new()),
                "+ Spieler"
            }
        }
        section {
            h3 { "Erster Geber" }
            select {
                aria_label: "Erster Geber",
                onchange: move |e| {
                    if let Ok(i) = e.value().parse::<usize>() {
                        first_dealer.set(i);
                    }
                },
                for (i, name) in names.read().iter().enumerate() {
                    option {
                        key: "{i}",
                        value: "{i}",
                        selected: first_dealer() == i,
                        if name.trim().is_empty() { "Spieler {i + 1}" } else { "{name}" }
                    }
                }
            }
        }
        section {
            h3 { "Tarif" }
            p { class: "hint", "Lässt sich später jederzeit ändern. Schon eingetragene Spiele bleiben dabei unverändert." }
            TariffFields { draft: tariff }
        }
        div { class: "actions",
            button { class: "primary big", r#type: "button", onclick: start, "Abend starten" }
            if running {
                button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "Abbrechen" }
            }
        }
    }
}
