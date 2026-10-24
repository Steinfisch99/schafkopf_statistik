//! Frühere Abende: die Liste, und ein einzelner Abend zum Ansehen mit Diagramm, Statistik und Export.

use dioxus::prelude::*;

use schafkopf_core::Table;

use super::balance_chart::BalanceChart;
use super::common::{ConfirmButton, Notice};
use super::export_screen::ExportScreen;
use super::game_list::{GameList, OffenceList, StockList};
use super::seats_screen::EventLog;
use super::stats_screen::StatsScreen;
use crate::navigation::back_target;
use crate::summary::{evening_date, evening_times, games_text, leader_text, players_text};
use crate::{storage, Screen};

/// Welcher Teil eines früheren Abends gezeigt wird.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PastSection {
    Overview,
    /// Die Statistik, auf Wunsch mit der Karte eines Spielers geöffnet.
    Stats(Option<usize>),
    Export,
}

/// Alle früheren Abende, der neueste zuerst.
#[component]
pub fn PastList(screen: Signal<Screen>, has_table: bool) -> Element {
    let mut screen = screen;
    // Beim Öffnen einmal gelesen. Wer einen Abend löscht, kommt über diese Liste zurück und
    // zeichnet sie damit neu.
    let list = use_signal(storage::list_past);
    let back = back_target(Screen::PastList, has_table).unwrap_or(Screen::Setup);

    rsx! {
        h2 { "Frühere Abende" }
        if list.read().is_empty() {
            p { class: "hint empty",
                "Noch keine früheren Abende. Ein Abend landet hier, sobald du einen neuen beginnst und er mindestens ein Spiel hatte."
            }
        } else {
            p { class: "hint", "Tippe einen Abend an, um Diagramm, Statistik und Spiele noch einmal anzusehen." }
            for past in list.read().iter() {
                button {
                    key: "{past.stamp}",
                    class: "past-card",
                    r#type: "button",
                    onclick: {
                        let stamp = past.stamp;
                        move |_| screen.set(Screen::Past(stamp))
                    },
                    div { class: "past-title",
                        b { "{evening_date(&past.table, past.stamp)}" }
                        if past.table.debug {
                            span { class: "badge", " Test" }
                        }
                        span { class: "muted", " · {games_text(past.table.games().len())}" }
                    }
                    if let Some(times) = evening_times(&past.table) {
                        div { class: "muted", "{times}" }
                    }
                    div { class: "muted", "{players_text(&past.table)}" }
                    if let Some(leader) = leader_text(&past.table) {
                        div { class: "past-leader", "Vorne: {leader}" }
                    }
                }
            }
        }
        div { class: "actions stack",
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Overall), "Gesamtstatistik über mehrere Abende" }
        }
        div { class: "actions",
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(back), "Zurück" }
        }
    }
}

/// Ein früherer Abend. Er wird beim Öffnen geladen und lässt sich nur ansehen, exportieren oder
/// löschen. Diese eine Komponente zeigt alle Teile, damit der geladene Abend zwischen
/// Übersicht, Statistik und Export nicht neu gelesen wird.
#[component]
pub fn PastEveningView(
    stamp: u64,
    section: PastSection,
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;
    let mut viewed = use_signal(|| storage::load_past(stamp));

    // Der Abend wird wieder zum laufenden: Spiele lassen sich bearbeiten, danach beendet man ihn neu.
    let reopen = move |_| {
        if table.read().as_ref().is_some_and(|t| !t.games().is_empty()) {
            return notice.set(Some(Notice::error(
                "Es läuft noch ein Abend mit Spielen. Beende ihn zuerst, dann lässt sich dieser öffnen.",
            )));
        }
        match storage::reopen_past(stamp) {
            Ok(Some(reopened)) => {
                table.set(Some(reopened));
                screen.set(Screen::Table);
                notice.set(Some(Notice::ok(
                    "Abend wieder geöffnet. Nach dem Bearbeiten unten wieder beenden.",
                )));
            }
            Ok(None) => notice.set(Some(Notice::error("Diesen Abend gibt es nicht mehr."))),
            Err(e) => notice.set(Some(Notice::error(format!("Öffnen fehlgeschlagen: {e}")))),
        }
    };

    let delete = move |_| {
        match storage::delete_past(stamp) {
            Ok(()) => notice.set(Some(Notice::ok("Abend gelöscht"))),
            Err(e) => notice.set(Some(Notice::error(format!("Löschen fehlgeschlagen: {e}")))),
        }
        screen.set(Screen::PastList);
    };

    let toggle_debug = move |_| {
        let mut guard = viewed.write();
        let Some(t) = guard.as_mut() else { return };
        t.debug = !t.debug;
        let saved = storage::save_past(stamp, t);
        let debug = t.debug;
        drop(guard);
        notice.set(Some(match saved {
            Ok(()) if debug => {
                Notice::ok("Als Testabend markiert, er zählt nicht mehr in der Gesamtstatistik")
            }
            Ok(()) => {
                Notice::ok("Markierung entfernt, der Abend zählt wieder in der Gesamtstatistik")
            }
            Err(e) => Notice::error(format!("Speichern fehlgeschlagen: {e}")),
        }));
    };
    let is_debug = viewed.read().as_ref().is_some_and(|t| t.debug);

    let (date, times, players, rules, game_count) = match viewed.read().as_ref() {
        Some(t) => (
            evening_date(t, stamp),
            evening_times(t),
            players_text(t),
            format!(
                "Regelwerk: {} · Tarif: {}",
                t.rule_set_name.as_deref().unwrap_or("eigene Auswahl"),
                t.tariff_name.as_deref().unwrap_or("Eigener Tarif")
            ),
            t.games().len(),
        ),
        None => {
            return rsx! {
                h2 { "Abend nicht gefunden" }
                p { class: "hint", "Diesen Abend gibt es nicht mehr." }
                div { class: "actions",
                    button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::PastList), "Zurück" }
                }
            };
        }
    };

    match section {
        PastSection::Stats(player) => rsx! {
            StatsScreen { table: viewed, screen, back: Screen::Past(stamp), focus: player }
        },
        PastSection::Export => rsx! {
            ExportScreen { table: viewed, screen, notice, back: Screen::Past(stamp) }
        },
        PastSection::Overview => rsx! {
            h2 {
                "Abend vom {date}"
                if is_debug {
                    span { class: "badge", " Test" }
                }
            }
            if let Some(times) = &times {
                p { class: "muted", "{times}" }
            }
            p { class: "muted", "{players}" }
            p { class: "muted small", "{games_text(game_count)} · {rules}" }
            p { class: "hint", "Dieser Abend ist abgeschlossen. Du kannst ihn ansehen, exportieren, löschen oder zum Bearbeiten wieder öffnen." }
            BalanceChart {
                table: viewed,
                on_player: move |seat| screen.set(Screen::PastStats { stamp, player: Some(seat) }),
            }
            EventLog { table: viewed }
            GameList { table: viewed, screen, editable: false }
            OffenceList { table: viewed, notice, editable: false }
            StockList { table: viewed, notice, editable: false }
            div { class: "actions stack",
                button {
                    class: "secondary",
                    r#type: "button",
                    onclick: move |_| screen.set(Screen::PastStats { stamp, player: None }),
                    "Statistik"
                }
                button {
                    class: "secondary",
                    r#type: "button",
                    onclick: move |_| screen.set(Screen::PastExport(stamp)),
                    "CSV exportieren"
                }
                button {
                    class: "secondary",
                    r#type: "button",
                    onclick: toggle_debug,
                    if is_debug { "Testabend-Markierung entfernen" } else { "Als Testabend markieren" }
                }
                ConfirmButton {
                    label: "Abend wieder öffnen (Spiele bearbeiten)",
                    confirm_label: "Ja, wieder öffnen",
                    onconfirm: reopen,
                }
                ConfirmButton { label: "Abend löschen", confirm_label: "Ja, endgültig löschen", onconfirm: delete }
                button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::PastList), "Zurück" }
            }
        },
    }
}
