//! Geldflüsse als Sankey-Diagramm: wer an wen wie viel gezahlt hat.

use dioxus::prelude::*;
use schafkopf_charts::{format_euro, player_sankey_svg, sankey_svg};
use schafkopf_core::Cents;

use super::common::Chip;

/// Was ein Spieler an die anderen gezahlt und von ihnen bekommen hat, die größten Beträge zuerst.
pub fn flow_lines(
    names: &[String],
    flows: &[Vec<Cents>],
    player: usize,
) -> (Vec<String>, Vec<String>) {
    let mut paid: Vec<(Cents, usize)> = flows
        .get(player)
        .map(|row| {
            row.iter()
                .copied()
                .enumerate()
                .map(|(j, v)| (v, j))
                .collect()
        })
        .unwrap_or_default();
    let mut got: Vec<(Cents, usize)> = flows
        .iter()
        .enumerate()
        .map(|(i, row)| (row.get(player).copied().unwrap_or(0), i))
        .collect();
    let line = |list: &mut Vec<(Cents, usize)>| -> Vec<String> {
        list.retain(|&(v, j)| v > 0 && j != player);
        list.sort_by_key(|&(v, j)| (std::cmp::Reverse(v), j));
        list.iter()
            .map(|&(v, j)| {
                format!(
                    "{}: {}",
                    names.get(j).map_or("?", String::as_str),
                    format_euro(v)
                )
            })
            .collect()
    };
    (line(&mut paid), line(&mut got))
}

#[component]
pub fn FlowSection(names: Vec<String>, flows: Vec<Vec<Cents>>, split: Signal<bool>) -> Element {
    let mut split = split;
    let mut picked = use_signal(|| None::<usize>);
    let total: Cents = flows.iter().flatten().sum();
    if total <= 0 {
        return rsx! {};
    }
    let svg = sankey_svg(&names, &flows, picked());
    let details = picked().map(|p| {
        let (paid, got) = flow_lines(&names, &flows, p);
        (names.get(p).cloned().unwrap_or_default(), paid, got)
    });
    rsx! {
        section {
            h3 { "Geldflüsse" }
            p { class: "legend-note",
                "Links steht, wer gezahlt hat, rechts, wer bekommen hat. Die Bänder zeigen, wie viel von wem zu wem geflossen ist. "
                "Beim Sauspiel zahlt jeder Verlierer seinen ganzen Betrag an einen der beiden Gewinner (die Zuordnung wechselt von Spiel zu Spiel). Mit dem Schalter zahlt er stattdessen die Hälfte an jeden, dann kommen halbe Preise vor. Beim Stock kommt es anteilig von denen, die eingezahlt haben."
            }
            div { class: "chips range",
                Chip {
                    label: "Sauspiel: Betrag auf beide Gewinner aufteilen".to_string(),
                    on: split(),
                    onclick: move |_| split.set(!split()),
                }
            }
            div { class: "chart sankey", dangerous_inner_html: "{svg}" }
            div { class: "chips range",
                for (seat, name) in names.iter().enumerate() {
                    Chip {
                        key: "{seat}",
                        label: name.clone(),
                        on: picked() == Some(seat),
                        onclick: move |_| {
                            let now = picked();
                            picked.set(if now == Some(seat) { None } else { Some(seat) });
                        },
                    }
                }
            }
            if let Some((name, paid, got)) = details {
                p { class: "chart-info",
                    b { "{name}" }
                    " zahlte an: "
                    if paid.is_empty() { "niemanden" } else { "{paid.join(\", \")}" }
                }
                p { class: "chart-info",
                    b { "{name}" }
                    " bekam von: "
                    if got.is_empty() { "niemandem" } else { "{got.join(\", \")}" }
                }
            } else {
                p { class: "chart-info muted", "Tippe auf einen Namen, um nur seine Bänder zu sehen." }
            }
        }
    }
}

/// Das Sankey-Diagramm eines einzelnen Spielers mit den Beträgen in den Flüssen.
#[component]
pub fn PlayerFlow(
    names: Vec<String>,
    flows: Vec<Vec<Cents>>,
    player: usize,
    split: Signal<bool>,
) -> Element {
    let mut split = split;
    let touched = flows
        .get(player)
        .is_some_and(|row| row.iter().any(|&v| v > 0))
        || flows
            .iter()
            .any(|row| row.get(player).copied().unwrap_or(0) > 0);
    if !touched {
        return rsx! {};
    }
    let svg = player_sankey_svg(&names, &flows, player);
    rsx! {
        h4 { "Geldflüsse" }
        p { class: "legend-note", "Links: von wem er Geld bekam, rechts: an wen er zahlte. Die Zahlen stehen in den Bändern." }
        div { class: "chips range",
            Chip {
                label: "Sauspiel: Betrag auf beide Gewinner aufteilen".to_string(),
                on: split(),
                onclick: move |_| split.set(!split()),
            }
        }
        div { class: "chart sankey", dangerous_inner_html: "{svg}" }
    }
}
