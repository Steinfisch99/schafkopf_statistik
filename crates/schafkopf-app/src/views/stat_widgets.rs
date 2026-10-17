//! Bausteine für die Grafiken der Statistik: Ringe, Balken und die Ergebnisreihe.
//!
//! Farben allein tragen nie eine Aussage: Neben jedem Balken und Ring stehen die Zahlen dazu.

use dioxus::prelude::*;
use schafkopf_charts::{donut_svg, format_euro, player_color};
use schafkopf_core::Cents;

use crate::stats_text::{bar_percent, diverging, percent};

/// Sieg und Niederlage in zwei Farben, die sich auch bei Farbschwäche unterscheiden (Okabe/Ito).
pub const WIN_COLOR: &str = "#009E73";
pub const LOSE_COLOR: &str = "#D55E00";

/// Ein Ring mit Legende. `parts` sind `(Name, Wert, Farbe)`. Stücke ohne Wert stehen in der
/// Legende, bleiben im Ring aber unsichtbar.
#[component]
pub fn DonutWithLegend(
    parts: Vec<(String, u32, String)>,
    center: String,
    sub: String,
    label: String,
) -> Element {
    let slices: Vec<(f64, &str)> = parts
        .iter()
        .map(|(_, value, color)| (f64::from(*value), color.as_str()))
        .collect();
    let svg = donut_svg(&slices, &center, &sub, &label);
    rsx! {
        div { class: "donut-wrap",
            div { class: "donut", dangerous_inner_html: "{svg}" }
            ul { class: "donut-legend",
                for (name, value, color) in parts {
                    li { key: "{name}",
                        span { class: "swatch", style: "background:{color}" }
                        span { "{value} {name}" }
                    }
                }
            }
        }
    }
}

/// Siege und Niederlagen eines Spielers (oder der Spielerpartei) als Ring mit der Siegquote.
#[component]
pub fn WinDonut(
    wins: u32,
    games: u32,
    win_name: String,
    lose_name: String,
    label: String,
) -> Element {
    let sub = match games {
        0 => String::new(),
        1 => "1 Spiel".to_string(),
        n => format!("{n} Spiele"),
    };
    rsx! {
        DonutWithLegend {
            parts: vec![
                (win_name, wins, WIN_COLOR.to_string()),
                (lose_name, games.saturating_sub(wins), LOSE_COLOR.to_string()),
            ],
            center: if games == 0 { "–".to_string() } else { percent(wins, games) },
            sub,
            label,
        }
    }
}

/// Balken mit Siegen und Niederlagen hintereinander. `max` ist die Länge der längsten Zeile in
/// Spielen, damit die Zeilen untereinander vergleichbar sind. `value` ist der Text rechts.
#[component]
pub fn StackedBar(label: String, won: u32, lost: u32, max: u32, value: String) -> Element {
    let won_width = bar_percent(won, max);
    let lost_width = bar_percent(lost, max);
    rsx! {
        div { class: "bar-row",
            span { class: "bar-label", "{label}" }
            div { class: "bar-track",
                div { class: "bar-win", style: "width:{won_width}%" }
                div { class: "bar-lose", style: "width:{lost_width}%" }
            }
            span { class: "bar-value", "{value}" }
        }
    }
}

/// Balken für einen Anteil: `part` von `total`, mit dem Text `value` rechts.
#[component]
pub fn ShareBar(label: String, part: u32, total: u32, value: String) -> Element {
    let width = bar_percent(part, total);
    rsx! {
        div { class: "bar-row",
            span { class: "bar-label", "{label}" }
            div { class: "bar-track",
                div { class: "bar-fill", style: "width:{width}%" }
            }
            span { class: "bar-value", "{value}" }
        }
    }
}

/// Balken, der von einer Mittellinie aus nach rechts (Gewinn) oder links (Verlust) wächst.
/// `max_abs` ist der größte Betrag aller Zeilen und bestimmt den Maßstab.
#[component]
pub fn DivergingRow(label: String, seat: Option<usize>, balance: Cents, max_abs: Cents) -> Element {
    let (left, width) = diverging(balance, max_abs);
    let color = if balance < 0 { LOSE_COLOR } else { WIN_COLOR };
    let amount = format_euro(balance);
    rsx! {
        div { class: "bar-row",
            span { class: "bar-label",
                if let Some(seat) = seat {
                    span { class: "swatch", style: "background:{player_color(seat)}" }
                }
                "{label}"
            }
            div { class: "diverge",
                div { class: "fill", style: "left:{left}%;width:{width}%;background:{color}" }
            }
            span { class: if balance < 0 { "bar-value neg" } else { "bar-value pos" }, "{amount}" }
        }
    }
}

/// Die Ergebnisse Spiel für Spiel: ein Kästchen je Spiel, `+` Sieg, `−` Niederlage, gestrichelt
/// ausgesetzt.
#[component]
pub fn ResultStrip(results: Vec<Option<bool>>) -> Element {
    rsx! {
        div { class: "strip", role: "img", "aria-label": "Ergebnisse Spiel für Spiel",
            for (index, result) in results.into_iter().enumerate() {
                match result {
                    Some(true) => rsx! { span { key: "{index}", class: "w", title: "Spiel {index + 1}: Sieg", "+" } },
                    Some(false) => rsx! { span { key: "{index}", class: "l", title: "Spiel {index + 1}: Niederlage", "−" } },
                    None => rsx! { span { key: "{index}", class: "off", title: "Spiel {index + 1}: ausgesetzt" } },
                }
            }
        }
    }
}

/// Farben für Lauf, Gegenlauf und "ohne Lauf": Blau und Gelb aus der Okabe/Ito-Palette, dazu ein Grau.
pub const LAUF_COLOR: &str = "#0072B2";
pub const GEGENLAUF_COLOR: &str = "#E69F00";
pub const NO_LAUF_COLOR: &str = "#9AA0A6";

/// Balken aus mehreren Stücken hintereinander, jedes mit eigener Farbe. `max` ist die Länge der
/// längsten Zeile in Spielen, damit die Zeilen untereinander vergleichbar sind.
#[component]
pub fn SegmentBar(label: String, segments: Vec<(u32, String)>, max: u32, value: String) -> Element {
    rsx! {
        div { class: "bar-row",
            span { class: "bar-label", "{label}" }
            div { class: "bar-track",
                for (index, (count, color)) in segments.into_iter().enumerate() {
                    div {
                        key: "{index}",
                        class: "bar-seg",
                        style: "width:{bar_percent(count, max)}%;background:{color}",
                    }
                }
            }
            span { class: "bar-value", "{value}" }
        }
    }
}
