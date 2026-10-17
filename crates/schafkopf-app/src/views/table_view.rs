//! Der Tisch: Geldstand, Spieleliste und die Wege zu Eingabe, Tarif und neuem Abend.

use dioxus::prelude::*;

use schafkopf_core::Table;

use super::balance_chart::BalanceChart;
use super::common::{ConfirmButton, Notice};
use crate::summary::{detail, signed_euro, title};
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
    let games = t.games();

    rsx! {
        section { class: "dealer",
            span { "Nächster Geber: " }
            b { "{t.players[t.next_dealer()]}" }
            span { class: "muted", " · {games.len()} Spiele" }
        }
        button {
            class: "primary big",
            r#type: "button",
            onclick: move |_| screen.set(Screen::Game),
            "Spiel eintragen"
        }
        BalanceChart { table }
        if !games.is_empty() {
            details { class: "history", open: games.len() <= 5,
                summary { "Spiele ({games.len()})" }
                ol { reversed: true, start: "{games.len()}",
                    for (index, game) in games.iter().enumerate().rev() {
                        li { key: "{index}",
                            div { class: "game-line",
                                b { "{title(game)}" }
                                span { class: if game.payouts[game.input.declarer] < 0 { "amount neg" } else { "amount" },
                                    "{signed_euro(game.payouts[game.input.declarer])}"
                                }
                            }
                            div { class: "muted", "{detail(t, game)}" }
                        }
                    }
                }
            }
        }
        div { class: "actions stack",
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Tariff), "Tarif ändern" }
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Export), "CSV exportieren" }
            if !games.is_empty() {
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
