//! Das Geldstand-Diagramm zum Bedienen: Tippen zeigt die Werte eines Spiels, Ziehen verschiebt,
//! zwei Finger zoomen, Doppeltippen oder "Alles" zeigt den ganzen Verlauf.

use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use schafkopf_charts::{balance_chart_svg_filtered, format_euro, player_color, ChartView};
use schafkopf_core::{balance_history, Cents, Table};

use super::common::Chip;
use crate::gestures::{Gestures, Outcome};
use crate::summary::{detail, title};

/// Bereichsknöpfe: so viele letzte Spiele. Angeboten werden nur die, die weniger als alle zeigen.
const LAST_GAMES: [usize; 3] = [20, 10, 5];

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// Der Geldstand eines Abends, Spiel für Spiel.
#[component]
pub fn BalanceChart(table: Signal<Option<Table>>, on_player: EventHandler<usize>) -> Element {
    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    let history = balance_history(t);
    let infos: Vec<String> = (0..history.len())
        .map(|n| match n {
            0 => "Start: alle bei 0,00 €".to_string(),
            n => t
                .games()
                .get(n - 1)
                .map(|g| format!("Spiel {n}: {} · {}", title(g), detail(t, g)))
                .unwrap_or_default(),
        })
        .collect();
    rsx! {
        HistoryChart {
            history,
            infos,
            names: t.players.clone(),
            away: (0..t.players.len()).map(|s| !t.is_present(s)).collect(),
            markers: Vec::new(),
            unit: "Spiele".to_string(),
            step: "Spiel".to_string(),
            empty_text: "Noch kein Spiel eingetragen. Sobald das erste Spiel gespeichert ist, erscheint hier der Geldstand.".to_string(),
            on_player: Some(on_player),
        }
    }
}

/// Ein Verlaufsdiagramm mit Linien je Spieler zum Bedienen. `history[n]` ist der Stand nach `n` Schritten
/// (Spiele oder Abende), `infos[n]` der Text zum angetippten Schritt, `markers` sind Trennlinien.
#[component]
pub fn HistoryChart(
    history: Vec<Vec<Cents>>,
    infos: Vec<String>,
    names: Vec<String>,
    away: Vec<bool>,
    markers: Vec<usize>,
    unit: String,
    step: String,
    empty_text: String,
    on_player: Option<EventHandler<usize>>,
) -> Element {
    let mut view = use_signal(|| ChartView::full(0));
    let mut gestures = use_signal(Gestures::default);
    let mut width = use_signal(|| 360.0_f64);
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);
    // Welche Spieler im Diagramm gezeigt werden sollen. Ist keiner gewählt, sind alle zu sehen.
    let mut picked = use_signal(Vec::<bool>::new);

    let names_len = names.len();
    let games = history.len().saturating_sub(1);
    let current = view.read().adapted(games);
    let hidden: Vec<bool> = {
        let picked = picked.read();
        let any = picked.iter().any(|&p| p);
        (0..names.len())
            .map(|s| any && !picked.get(s).copied().unwrap_or(false))
            .collect()
    };
    let svg = balance_chart_svg_filtered(&history, &current, &markers, &hidden);
    let shown = current.selected.unwrap_or(games);
    let balances = history.get(shown).cloned().unwrap_or_default();

    // Die Breite des Diagramms in Pixeln braucht die Gestenrechnung. Sie wird beim Einhängen und bei
    // jedem neuen Berühren gemessen, damit Drehen des Geräts nicht zu falschen Positionen führt.
    let measure = move || {
        if let Some(element) = mounted.read().clone() {
            spawn(async move {
                if let Ok(rect) = element.get_client_rect().await {
                    if rect.width() > 1.0 {
                        width.set(rect.width());
                    }
                }
            });
        }
    };

    let info = current
        .selected
        .and_then(|n| infos.get(n).cloned())
        .filter(|s| !s.is_empty());

    rsx! {
        if games == 0 {
            p { class: "hint empty",
                "{empty_text}"
            }
        }
        if games > 0 {
        div {
            class: "chart",
            onmounted: move |e| {
                mounted.set(Some(e.data()));
                measure();
            },
            onpointerdown: move |e| {
                measure();
                let p = e.element_coordinates();
                let current = view.read().adapted(games);
                gestures.write().down(e.pointer_id(), p.x, p.y, &current, width());
            },
            onpointermove: move |e| {
                let p = e.element_coordinates();
                let before = view.read().adapted(games);
                let mut after = before.clone();
                gestures.write().moved_to(e.pointer_id(), p.x, p.y, &mut after, width());
                if after != before {
                    view.set(after);
                }
            },
            onpointerup: move |e| {
                let p = e.element_coordinates();
                let mut after = view.read().adapted(games);
                let outcome = gestures.write().up(e.pointer_id(), p.x, p.y, now_millis(), &mut after, width());
                if outcome == Outcome::Reset {
                    after.selected = None;
                }
                if let Outcome::Select(game) = outcome {
                    let game = (game.round().max(0.0) as usize).min(games);
                    after.selected = if after.selected == Some(game) { None } else { Some(game) };
                }
                view.set(after);
            },
            onpointercancel: move |e| gestures.write().cancel(e.pointer_id()),
            onpointerleave: move |e| gestures.write().cancel(e.pointer_id()),
            dangerous_inner_html: "{svg}",
        }

        div { class: "chips range",
            Chip {
                label: "Alles".to_string(),
                on: current.is_full(),
                onclick: move |_| {
                    let mut v = view.read().adapted(games);
                    v.reset();
                    view.set(v);
                },
            }
            for n in LAST_GAMES.into_iter().filter(|&n| n < games) {
                Chip {
                    key: "{n}",
                    label: format!("Letzte {n}"),
                    on: !current.is_full() && current.end >= games as f64 && (current.len() - n as f64).abs() < 0.5,
                    onclick: move |_| {
                        let mut v = view.read().adapted(games);
                        v.show_last(n);
                        view.set(v);
                    },
                }
            }
        }

        if let Some(text) = info {
            p { class: "chart-info",
                span { "{text}" }
                button {
                    class: "link",
                    r#type: "button",
                    onclick: move |_| {
                        let mut v = view.read().adapted(games);
                        v.selected = None;
                        view.set(v);
                    },
                    "Auswahl aufheben"
                }
            }
        } else if !current.is_full() {
            p { class: "chart-info muted", "{unit} {current.start.round()} bis {current.end.round()} von {games}. Doppeltippen zeigt alles." }
        } else if games > 1 {
            p { class: "chart-info muted", "Tippen zeigt die Werte eines {step}s, Ziehen verschiebt, mit zwei Fingern zoomen." }
        }

        }

        if games > 0 && names.len() > 1 {
            p { class: "legend-note", "Tippe auf Namen, um nur diese Linien zu zeigen." }
            div { class: "chips",
                for (seat, name) in names.iter().enumerate() {
                    button {
                        key: "{seat}",
                        class: if picked.read().get(seat).copied().unwrap_or(false) { "chip on" } else { "chip" },
                        r#type: "button",
                        "aria-pressed": "{picked.read().get(seat).copied().unwrap_or(false)}",
                        onclick: move |_| {
                            let mut p = picked.write();
                            p.resize(names_len, false);
                            p[seat] = !p[seat];
                        },
                        span { class: "dot", style: "background:{player_color(seat)}" }
                        "{name}"
                    }
                }
                if picked.read().iter().any(|&p| p) {
                    button {
                        class: "chip",
                        r#type: "button",
                        onclick: move |_| picked.write().clear(),
                        "Alle zeigen"
                    }
                }
            }
        }

        p { class: "muted legend-title",
            if current.selected.is_some() {
                "Stand nach {step} {shown}"
            } else {
                "Aktueller Stand"
            }
        }
        if on_player.is_some() {
            p { class: "legend-note", "Tippe auf einen Namen für die Statistik des Spielers." }
        }
        ul { class: "legend",
            for (seat, name) in names.iter().enumerate() {
                {
                    let balance = balances.get(seat).copied().unwrap_or(0);
                    let amount_class = if balance < 0 { "amount neg" } else { "amount" };
                    let away_class = if away.get(seat).copied().unwrap_or(false) { "legend-btn away" } else { "legend-btn" };
                    rsx! {
                        li { key: "{seat}",
                            if let Some(on_player) = on_player {
                                button {
                                    class: "{away_class}",
                                    r#type: "button",
                                    "aria-label": "Statistik von {name}",
                                    onclick: move |_| on_player.call(seat),
                                    span { class: "dot", style: "background:{player_color(seat)}" }
                                    span { class: "legend-text",
                                        span { class: "name", "{name}" }
                                        span { class: "{amount_class}", "{format_euro(balance)}" }
                                    }
                                    span { class: "chev", "›" }
                                }
                            } else {
                                div { class: "{away_class}",
                                    span { class: "dot", style: "background:{player_color(seat)}" }
                                    span { class: "legend-text",
                                        span { class: "name", "{name}" }
                                        span { class: "{amount_class}", "{format_euro(balance)}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
