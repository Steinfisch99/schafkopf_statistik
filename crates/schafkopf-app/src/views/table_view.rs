//! Der Tisch: Geldstand, Spieleliste und die Wege zu Eingabe, Tarif und neuem Abend.

use dioxus::prelude::*;

use schafkopf_core::{BockMode, Table};

use super::balance_chart::BalanceChart;
use super::common::{ConfirmButton, Notice};
use super::game_list::GameList;
use crate::{storage, Screen};

#[component]
pub fn TableView(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;

    let undo = move |_| {
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        if t.undo_last().is_none() {
            return;
        }
        let saved = storage::save(t, None);
        drop(guard);
        match saved {
            Ok(_) => notice.set(Some(Notice::ok("Letztes Spiel rückgängig gemacht"))),
            Err(e) => notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}"
            )))),
        }
    };

    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    let game_count = t.games().len();
    let bock_pending = if t.bock_mode == BockMode::Off {
        0
    } else {
        t.bock_pending()
    };
    let bock_text = match bock_pending {
        0 => None,
        1 => Some("Bock: Das nächste Spiel ist ein Bockspiel, der Preis ist doppelt.".to_string()),
        n => Some(format!(
            "Bock: Die nächsten {n} Spiele sind Bockspiele, der Preis ist doppelt."
        )),
    };

    rsx! {
        section { class: "dealer",
            span { "Nächster Geber: " }
            b { "{t.players[t.next_dealer()]}" }
            span { class: "muted", " · {game_count} Spiele" }
            p { class: "muted small rules",
                "Regelwerk: {t.rule_set_name.as_deref().unwrap_or(\"eigene Auswahl\")} · Tarif: {t.tariff_name.as_deref().unwrap_or(\"Eigener Tarif\")}"
            }
            if let Some(text) = &bock_text {
                p { class: "bock-note", "{text}" }
            }
        }
        button {
            class: "primary big",
            r#type: "button",
            onclick: move |_| screen.set(Screen::Game),
            "Spiel eintragen"
        }
        BalanceChart {
            table,
            on_player: move |seat| screen.set(Screen::Stats { player: Some(seat) }),
        }
        GameList { table, screen, editable: true }
        div { class: "actions stack",
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Stats { player: None }), "Statistik" }
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::PastList), "Frühere Abende" }
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Settings), "Regeln und Tarif" }
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Export), "CSV exportieren" }
            if game_count > 0 {
                ConfirmButton {
                    label: "Letztes Spiel rückgängig",
                    confirm_label: "Ja, rückgängig",
                    onconfirm: undo,
                }
            }
            ConfirmButton {
                label: "Neuer Abend",
                confirm_label: "Ja, neu beginnen",
                onconfirm: move |_| screen.set(Screen::Setup),
            }
        }
    }
}
