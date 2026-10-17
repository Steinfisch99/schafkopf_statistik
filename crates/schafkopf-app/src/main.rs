//! Prototyp: prüft auf dem Gerät, ob Eingabefeld, SVG-Diagramm und Dateispeicherung funktionieren.

mod storage;

use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use schafkopf_charts::{balance_chart_svg, format_euro, player_color};
use schafkopf_core::{balance_history, GameInput, GameKind, Table, Tariff};

const CSS: &str = include_str!("style.css");

fn main() {
    dioxus::launch(App);
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Ein Testspiel (Wenz) mit zufälligem Spieler und Ausgang, nur für den Prototyp.
fn random_game(table: &Table) -> GameInput {
    let dealer = table.next_dealer();
    let participants = table.default_participants(dealer);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |d| d.subsec_nanos() as usize);
    let declarer = participants[seed % participants.len()];
    GameInput {
        won: (seed / 7).is_multiple_of(2),
        laufende: ((seed / 13) % 6) as u8,
        played_at: Some(now_secs() as i64),
        ..GameInput::new(dealer, GameKind::Wenz, declarer)
    }
}

#[component]
fn App() -> Element {
    let mut table = use_signal(storage::load);
    let mut names = use_signal(|| "Anna, Ben, Cem, Dora".to_string());
    let mut status = use_signal(String::new);
    let mut failed = use_signal(|| false);

    let mut report = move |result: Result<String, String>| match result {
        Ok(msg) => {
            failed.set(false);
            status.set(msg);
        }
        Err(msg) => {
            failed.set(true);
            status.set(msg);
        }
    };

    let create = move |_| {
        let players: Vec<String> = names().split(',').map(|s| s.trim().to_owned()).collect();
        let result = Table::new("Prototyp", players, 0, Tariff::default())
            .map_err(|e| e.to_string())
            .and_then(|t| {
                let dir = storage::save(&t, None)
                    .map_err(|e| format!("Speichern fehlgeschlagen: {e}"))?;
                table.set(Some(t));
                Ok(format!("Tisch angelegt, gespeichert in {}", dir.display()))
            });
        report(result);
    };

    let add_game = move |_| {
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        let result = (|| {
            let input = random_game(t);
            t.add_game(input).map_err(|e| e.to_string())?;
            let index = t.games().len() - 1;
            let dir = storage::save(t, Some(index))
                .map_err(|e| format!("Speichern fehlgeschlagen: {e}"))?;
            Ok(format!(
                "Spiel {} gespeichert in {}",
                index + 1,
                dir.display()
            ))
        })();
        drop(guard);
        report(result);
    };

    let new_table = move |_| {
        table.set(None);
        status.set(String::new());
    };

    rsx! {
        style { dangerous_inner_html: CSS }
        main {
            h1 { "Schafkopf-Statistik (Prototyp)" }
            match table() {
                None => rsx! {
                    label { r#for: "names", "Spieler in Sitzreihenfolge, mit Komma getrennt" }
                    input {
                        id: "names",
                        value: "{names}",
                        oninput: move |e| names.set(e.value()),
                    }
                    button { onclick: create, "Tisch anlegen" }
                },
                Some(t) => {
                    let svg = balance_chart_svg(&balance_history(&t));
                    let balances = t.balances();
                    rsx! {
                        p { "Nächster Geber: {t.players[t.next_dealer()]}. Spiele: {t.games().len()}" }
                        div { class: "chart", dangerous_inner_html: "{svg}" }
                        ul { class: "legend",
                            for (seat, name) in t.players.iter().enumerate() {
                                li {
                                    span { class: "dot", style: "background:{player_color(seat)}" }
                                    "{name}"
                                    span { class: "amount", "{format_euro(balances[seat])}" }
                                }
                            }
                        }
                        button { onclick: add_game, "Testspiel eintragen" }
                        button { class: "secondary", onclick: new_table, "Neuer Tisch" }
                    }
                }
            }
            if !status().is_empty() {
                p { class: if failed() { "status error" } else { "status" }, "{status}" }
            }
        }
    }
}
