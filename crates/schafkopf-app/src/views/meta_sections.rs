//! Metadaten-Statistik: welcher Tarif und welches Regelwerk galten, und wann wer kam und ging.

use dioxus::prelude::*;
use schafkopf_core::{
    format_duration, format_minutes_after_noon, EveningFact, OverallPlayer, Presence, UseCount,
};

use super::stat_widgets::ShareBar;
use super::stats_screen::{count_share, Kv};
use crate::stats_text::{
    arrival_text, decimal, departure_text, evenings_summary, fact_date, spread_text,
};

#[component]
fn UseBars(title: String, uses: Vec<UseCount>, changes: u32, evenings: Option<u32>) -> Element {
    let total: u32 = uses.iter().map(|u| u.games).sum();
    rsx! {
        h4 { "{title}" }
        for u in uses {
            ShareBar {
                key: "{u.name}",
                label: u.name.clone(),
                part: u.games,
                total: total,
                value: count_share(u.games, total),
            }
        }
        Kv { label: "Wechsel von einem Spiel zum nächsten", value: changes.to_string() }
        if let Some(n) = evenings.filter(|&n| n > 1) {
            Kv { label: "Wechsel pro Abend (Ø)", value: format!("{:.1}", f64::from(changes) / f64::from(n)).replace('.', ",") }
        }
    }
}

/// Tarife und Regelwerke: Spiele je Name und wie oft gewechselt wurde.
#[component]
pub fn TariffSection(
    tariffs: Vec<UseCount>,
    tariff_changes: u32,
    rule_sets: Vec<UseCount>,
    rule_set_changes: u32,
    evenings: Option<u32>,
) -> Element {
    rsx! {
        section {
            h3 { "Tarif und Regelwerk" }
            p { class: "legend-note", "Gezählt werden die Spiele. Ein Wechsel ist, wenn ein Spiel mit einem anderen Tarif (oder Regelwerk) gespielt wurde als das davor." }
            UseBars { title: "Tarife", uses: tariffs, changes: tariff_changes, evenings }
            UseBars { title: "Regelwerke", uses: rule_sets, changes: rule_set_changes, evenings }
        }
    }
}

/// Wer an diesem Abend wann kam und ging.
#[component]
pub fn EveningPresence(rows: Vec<Presence>, offset_min: i32) -> Element {
    rsx! {
        section {
            h3 { "Wer kam, wer ging" }
            table { class: "stat-table",
                thead { tr { th { "Spieler" } th { "Kam" } th { "Ging" } } }
                tbody {
                    for p in rows {
                        tr { key: "{p.seat}",
                            td { class: "name-cell", "{p.name}" }
                            td { "{arrival_text(&p, offset_min)}" }
                            td { "{departure_text(&p, offset_min)}" }
                        }
                    }
                }
            }
        }
    }
}

/// Wann die Spieler über die gewählten Abende im Schnitt kommen und gehen.
#[component]
pub fn OverallPresence(
    players: Vec<OverallPlayer>,
    evening_starts: Vec<i32>,
    evening_lengths: Vec<i64>,
) -> Element {
    let mut rows: Vec<OverallPlayer> = players
        .into_iter()
        .filter(|p| !p.arrivals.is_empty() || !p.departures.is_empty())
        .collect();
    // Wer im Schnitt am frühesten kommt, steht oben.
    let avg = |v: &[i32]| v.iter().map(|&t| i64::from(t)).sum::<i64>() / v.len().max(1) as i64;
    rows.sort_by_key(|p| (avg(&p.arrivals), p.name.clone()));
    let start_avg = (!evening_starts.is_empty()).then(|| avg(&evening_starts) as i32);
    let length_avg = (!evening_lengths.is_empty())
        .then(|| evening_lengths.iter().sum::<i64>() / evening_lengths.len() as i64);
    rsx! {
        section {
            h3 { "Wann kommt wer, wann geht wer?" }
            p { class: "legend-note", "Ø Uhrzeit mit frühestem und spätestem Mal über die gewählten Abende. Wer bis zum Ende blieb, geht mit dem Ende des Abends in die Zahl ein. Die frühesten Gäste stehen oben." }
            if let Some(m) = start_avg {
                Kv { label: "Abende beginnen im Schnitt um", value: format_minutes_after_noon(m) }
            }
            if let Some(secs) = length_avg {
                Kv { label: "Ein Abend dauert im Schnitt", value: schafkopf_core::format_duration(secs) }
            }
            table { class: "stat-table",
                thead { tr { th { "Spieler" } th { "Kommt" } th { "Geht" } th { "Später" } th { "Früher" } } }
                tbody {
                    for p in rows {
                        tr { key: "{p.name}",
                            td { class: "name-cell", "{p.name}" }
                            td { "{spread_text(&p.arrivals)}" }
                            td { "{spread_text(&p.departures)}" }
                            td { "{p.late_arrivals}" }
                            td { "{p.early_departures}" }
                        }
                    }
                }
            }
            p { class: "legend-note", "Später: an so vielen Abenden kam er erst im Lauf des Abends dazu. Früher: so oft ging er, bevor der Abend zu Ende war." }
        }
    }
}

/// Wie viele Spiele wir pro Abend schaffen und wie lange ein Abend dauert, mit dem kleinsten und größten Wert und
/// dem Datum dazu.
#[component]
pub fn EveningsOverview(facts: Vec<EveningFact>) -> Element {
    let Some(summary) = evenings_summary(&facts) else {
        return rsx! {};
    };
    let games = summary.games.clone();
    rsx! {
        section {
            h3 { "Abende im Überblick" }
            p { class: "legend-note", "Über die gewählten Abende. Die Dauer gibt es nur für Abende, deren Beginn bekannt ist (bis zum Ende oder zur letzten Aktivität)." }
            h4 { "Spiele pro Abend" }
            Kv { label: "Im Schnitt", value: decimal(games.average) }
            Kv { label: "Wenigste", value: format!("{} ({})", games.min.value, games.min.date) }
            Kv { label: "Meiste", value: format!("{} ({})", games.max.value, games.max.date) }
            if let Some(length) = summary.length.clone() {
                h4 { "Dauer eines Abends" }
                Kv { label: "Im Schnitt", value: format_duration(length.average as i64) }
                Kv { label: "Kürzester", value: format!("{} ({})", format_duration(length.min.value), length.min.date) }
                Kv { label: "Längster", value: format!("{} ({})", format_duration(length.max.value), length.max.date) }
            }
            if let Some(rate) = summary.games_per_hour {
                Kv { label: "Spiele pro Stunde", value: decimal(rate) }
            }
            table { class: "stat-table",
                thead { tr { th { "Datum" } th { "Spiele" } th { "Dauer" } } }
                tbody {
                    for (i, f) in facts.iter().enumerate() {
                        tr { key: "{i}",
                            td { "{fact_date(f)}" }
                            td { "{f.games}" }
                            td { "{f.length.map_or(\"–\".to_string(), format_duration)}" }
                        }
                    }
                }
            }
        }
    }
}
