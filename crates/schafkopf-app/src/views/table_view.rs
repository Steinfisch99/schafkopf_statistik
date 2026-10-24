//! Der Tisch: Geldstand, Spieleliste und die Wege zu Eingabe, Tarif und neuem Abend.

use dioxus::prelude::*;

use schafkopf_charts::format_euro;
use schafkopf_core::{format_clock, latest_clock_before, BockMode, NoGameRule, Table};

use super::balance_chart::BalanceChart;
use super::common::{AppVersion, ConfirmButton, Notice, TextField};
use super::game_list::{GameList, OffenceList, StockList};
use crate::summary::{evening_times, stock_text};
use crate::{now_secs, storage, Screen};

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

    let no_game = move |_| {
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        let added = t.add_no_game(Some(now_secs() as i64)).cloned();
        let result = added.map(|m| stock_text(t, &m));
        let saved = result.as_ref().ok().map(|_| storage::save(t, None));
        drop(guard);
        notice.set(Some(match (result, saved) {
            (Err(e), _) => Notice::error(e.to_string()),
            (Ok(_), Some(Err(e))) => Notice::error(format!("Speichern fehlgeschlagen: {e}")),
            (Ok(text), _) => Notice::ok(text),
        }));
    };

    let split_stock = move |_| {
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        let added = t.split_stock(Some(now_secs() as i64)).cloned();
        let result = added.map(|m| stock_text(t, &m));
        let saved = result.as_ref().ok().map(|_| storage::save(t, None));
        drop(guard);
        notice.set(Some(match (result, saved) {
            (Err(e), _) => Notice::error(e.to_string()),
            (Ok(_), Some(Err(e))) => Notice::error(format!("Speichern fehlgeschlagen: {e}")),
            (Ok(text), _) => Notice::ok(text),
        }));
    };

    // Leer: Der Abend endet jetzt. Sonst die Uhrzeit, zu der er endete. Ein wieder geöffneter Abend
    // zeigt seine alte Endzeit vor.
    let mut end_time = use_signal(|| {
        table
            .peek()
            .as_ref()
            .and_then(|t| t.ended_at.map(|e| format_clock(e, t.utc_offset_min)))
            .unwrap_or_default()
    });

    let finish = move |_| {
        let now = now_secs();
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        let typed = end_time.read().trim().to_string();
        let end = if typed.is_empty() {
            now as i64
        } else if let Some(old) = t
            .ended_at
            .filter(|&e| format_clock(e, t.utc_offset_min) == typed)
        {
            old
        } else if let Some(at) = latest_clock_before(&typed, now as i64, crate::utc_offset_min()) {
            at
        } else {
            drop(guard);
            return notice.set(Some(Notice::error(
                "Die Uhrzeit des Endes bitte so eingeben: 23:15",
            )));
        };
        if t.last_activity().is_some_and(|last| end < last) {
            drop(guard);
            return notice.set(Some(Notice::error(
                "Das Ende liegt vor dem letzten Eintrag des Abends. Nimm eine spätere Uhrzeit.",
            )));
        }
        t.finish(end);
        let empty = t.games().is_empty();
        let saved = storage::save(t, None);
        drop(guard);
        if let Err(e) = saved {
            return notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}"
            ))));
        }
        // Ein Abend ohne ein einziges Spiel ist nichts, was man aufheben müsste.
        let archived = if empty {
            storage::discard_current()
        } else {
            storage::archive_current(now).map(|_| ())
        };
        if let Err(e) = archived {
            return notice.set(Some(Notice::error(format!(
                "Der Abend konnte nicht archiviert werden: {e}"
            ))));
        }
        table.set(None);
        screen.set(if empty {
            Screen::Setup
        } else {
            Screen::Past(now)
        });
        notice.set(Some(Notice::ok(if empty {
            "Abend beendet (ohne Spiele, deshalb nicht aufgehoben)"
        } else {
            "Abend beendet und bei den früheren Abenden abgelegt"
        })));
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
    let bock_level = t.bock_level();
    let bock_text = match (bock_pending, bock_level) {
        (0, _) => None,
        (1, 1) => Some("Bock: Das nächste Spiel ist ein Bockspiel, der Preis ist doppelt.".to_string()),
        (n, 1) => Some(format!(
            "Bock: Die nächsten {n} Spiele sind Bockspiele, der Preis ist doppelt."
        )),
        (n, level) => Some(format!(
            "Bock: {level} Bockrunden laufen zugleich, die nächsten {n} Spiele kosten das {}-fache.",
            1u32 << level
        )),
    };
    let times = evening_times(t);

    rsx! {
        section { class: "dealer",
            span { "Nächster Geber: " }
            b { "{t.players[t.next_dealer()]}" }
            span { class: "muted", " · {game_count} Spiele" }
            if let Some(times) = &times {
                p { class: "muted small", "Abend: {times}" }
            }
            p { class: "muted small rules",
                "Regelwerk: {t.rule_set_name.as_deref().unwrap_or(\"eigene Auswahl\")} · Tarif: {t.tariff_name.as_deref().unwrap_or(\"Eigener Tarif\")}"
            }
            if let Some(text) = &bock_text {
                p { class: "bock-note", "{text}" }
            }
            if t.no_game_rule == NoGameRule::Stock {
                p { class: "bock-note", "Stock: {format_euro(t.stock())} liegen in der Mitte." }
            }
        }
        button {
            class: "primary big",
            r#type: "button",
            onclick: move |_| screen.set(Screen::Game),
            "Spiel eintragen"
        }
        if t.no_game_rule.has_pass() {
            ConfirmButton {
                label: "Keiner spielt",
                confirm_label: if t.no_game_rule == NoGameRule::Stock { "Ja, alle legen in den Stock".to_string() } else { "Ja, zusammenwerfen".to_string() },
                onconfirm: no_game,
            }
        }
        if t.no_game_rule == NoGameRule::Stock && t.stock() > 0 {
            ConfirmButton {
                label: "Stock fair an alle aufteilen",
                confirm_label: "Ja, Stock gleichmäßig aufteilen",
                onconfirm: split_stock,
            }
        }
        BalanceChart {
            table,
            on_player: move |seat| screen.set(Screen::Stats { player: Some(seat) }),
        }
        GameList { table, screen, editable: true }
        OffenceList { table, notice, editable: true }
        StockList { table, notice, editable: true }
        div { class: "actions stack",
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Offence), "Vergehen oder Neumischen" }
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Seats), "Am Tisch: Spieler kommen und gehen" }
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
            TextField {
                label: "Ende des Abends (Uhrzeit, leer = jetzt)",
                value: end_time(),
                hint: "zum Beispiel 23:15",
                oninput: move |v| end_time.set(v),
            }
            ConfirmButton {
                label: "Abend beenden",
                confirm_label: "Ja, Abend beenden",
                onconfirm: finish,
            }
            ConfirmButton {
                label: "Neuer Abend",
                confirm_label: "Ja, neu beginnen",
                onconfirm: move |_| screen.set(Screen::Setup),
            }
        }
        AppVersion {}
    }
}
