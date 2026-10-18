//! Das Geldstand-Diagramm zum Bedienen: Tippen zeigt die Werte eines Spiels, Ziehen verschiebt,
//! zwei Finger zoomen, Doppeltippen oder "Alles" zeigt den ganzen Verlauf.

use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use schafkopf_charts::{balance_chart_svg_view, format_euro, player_color, ChartView};
use schafkopf_core::{balance_history, Table};

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

#[component]
pub fn BalanceChart(table: Signal<Option<Table>>, on_player: EventHandler<usize>) -> Element {
    let mut view = use_signal(|| ChartView::full(0));
    let mut gestures = use_signal(Gestures::default);
    let mut width = use_signal(|| 360.0_f64);
    let mut mounted = use_signal(|| None::<Rc<MountedData>>);

    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    let games = t.games().len();
    let history = balance_history(t);
    let current = view.read().adapted(games);
    let svg = balance_chart_svg_view(&history, &current);
    let shown = current.selected.unwrap_or(games);
    let balances = history[shown].clone();

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

    let info = match current.selected {
        Some(0) => Some("Start: alle bei 0,00 €".to_string()),
        Some(n) => t
            .games()
            .get(n - 1)
            .map(|g| format!("Spiel {n}: {} · {}", title(g), detail(t, g))),
        None => None,
    };

    rsx! {
        if games == 0 {
            p { class: "hint empty",
                "Noch kein Spiel eingetragen. Sobald das erste Spiel gespeichert ist, erscheint hier der Geldstand."
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
            p { class: "chart-info muted", "Spiele {current.start.round()} bis {current.end.round()} von {games}. Doppeltippen zeigt alles." }
        } else if games > 1 {
            p { class: "chart-info muted", "Tippen zeigt ein Spiel, Ziehen verschiebt, mit zwei Fingern zoomen." }
        }

        }

        p { class: "muted legend-title",
            if current.selected.is_some() {
                "Stand nach Spiel {shown}"
            } else {
                "Aktueller Stand"
            }
        }
        p { class: "legend-note", "Tippe auf einen Namen für die Statistik des Spielers." }
        ul { class: "legend",
            for (seat, name) in t.players.iter().enumerate() {
                li { key: "{seat}",
                    button {
                        class: if t.is_present(seat) { "legend-btn" } else { "legend-btn away" },
                        r#type: "button",
                        "aria-label": "Statistik von {name}",
                        onclick: move |_| on_player.call(seat),
                        span { class: "dot", style: "background:{player_color(seat)}" }
                        span { class: "legend-text",
                            span { class: "name", "{name}" }
                            span { class: if balances[seat] < 0 { "amount neg" } else { "amount" }, "{format_euro(balances[seat])}" }
                        }
                        span { class: "chev", "›" }
                    }
                }
            }
        }
    }
}
