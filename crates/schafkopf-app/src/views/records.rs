//! Rekorde (teuerstes Spiel je Spielart, meiste Laufende) und die Verteilung der Laufenden bei
//! Sauspielen und Solos, für einen Abend und für mehrere Abende.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use schafkopf_charts::format_euro;
use schafkopf_core::{Cents, GameKind, LaufCount, OverallStats, Table, TableStats};

use super::stat_widgets::StackedBar;
use super::stats_screen::Kv;
use crate::summary::{detail, evening_date, title};

/// Eine Rekordzeile: Name, Wert und die Beschreibung des Spiels darunter.
#[derive(Debug, Clone, PartialEq)]
pub struct RecordRow {
    pub label: String,
    pub value: String,
    pub detail: String,
}

type KindFilter = fn(GameKind) -> bool;

/// Wie oft es wie viele Laufende gab: Zeilen `(Name, gewonnen, verloren)`.
#[derive(Debug, Clone, PartialEq)]
pub struct LaufGroup {
    pub title: String,
    pub rows: Vec<(String, u32, u32)>,
}

fn record_row(
    label: String,
    table: &Table,
    game: usize,
    value: Cents,
    laufende: Option<u8>,
    with_date: bool,
) -> Option<RecordRow> {
    let g = table.games().get(game)?;
    let mut text = format!("{}, {}", title(g), detail(table, g));
    if with_date {
        text = format!("{} · {text}", evening_date(table, 0));
    }
    Some(RecordRow {
        label,
        value: match laufende {
            Some(n) => format!("{n} Laufende"),
            None => format_euro(value),
        },
        detail: text,
    })
}

/// Rekorde eines Abends.
pub fn evening_records(table: &Table, stats: &TableStats) -> Vec<RecordRow> {
    let mut rows = Vec::new();
    if let Some((game, value)) = stats.most_expensive {
        rows.extend(record_row(
            "Teuerstes Spiel".into(),
            table,
            game,
            value,
            None,
            false,
        ));
    }
    for kind in GameKind::ALL {
        if let Some(&(game, value)) = stats.most_expensive_by_kind.get(&kind) {
            rows.extend(record_row(
                format!("Teuerstes {}", kind.label()),
                table,
                game,
                value,
                None,
                false,
            ));
        }
    }
    if let Some((game, n)) = stats.most_laufende {
        rows.extend(record_row(
            "Meiste Laufende".into(),
            table,
            game,
            0,
            Some(n),
            false,
        ));
    }
    rows
}

/// Rekorde über mehrere Abende (in der Reihenfolge, in der sie an `overall_stats` gingen).
pub fn overall_records(tables: &[&Table], stats: &OverallStats) -> Vec<RecordRow> {
    let mut rows = Vec::new();
    let mut add = |label: String, r: &schafkopf_core::GameRef, laufende: bool| {
        if let Some(t) = tables.get(r.evening) {
            let n = laufende.then_some(r.laufende);
            rows.extend(record_row(label, t, r.game, r.value, n, true));
        }
    };
    if let Some(r) = &stats.most_expensive {
        add("Teuerstes Spiel".into(), r, false);
    }
    for kind in GameKind::ALL {
        if let Some(r) = stats.most_expensive_by_kind.get(&kind) {
            add(format!("Teuerstes {}", kind.label()), r, false);
        }
    }
    if let Some(r) = &stats.most_laufende {
        add("Meiste Laufende".into(), r, true);
    }
    rows
}

/// Sauspiele und Solos (alle Alleinspiele mit Laufenden zusammen), je Zahl der Laufenden.
pub fn lauf_groups(counts: &BTreeMap<GameKind, BTreeMap<u8, LaufCount>>) -> Vec<LaufGroup> {
    let groups: [(&str, KindFilter); 2] = [
        ("Sauspiele", |k| k == GameKind::Sauspiel),
        ("Solos", |k| k.is_solo() && k.has_extras()),
    ];
    let mut out = Vec::new();
    for (name, member) in groups {
        let mut merged: BTreeMap<u8, LaufCount> = BTreeMap::new();
        for (kind, by_count) in counts {
            if member(*kind) {
                for (n, c) in by_count {
                    merged.entry(*n).or_default().add(c);
                }
            }
        }
        let mut rows = Vec::new();
        for (n, c) in &merged {
            let mut push = |label: String, r: schafkopf_core::RoleStats| {
                if r.games > 0 {
                    rows.push((label, r.wins, r.games - r.wins));
                }
            };
            if *n == 0 {
                push("ohne Lauf".into(), c.lauf);
            } else {
                push(format!("{n} Laufende"), c.lauf);
                push(format!("{n} Gegenlauf"), c.gegenlauf);
            }
        }
        if !rows.is_empty() {
            out.push(LaufGroup {
                title: name.into(),
                rows,
            });
        }
    }
    out
}

#[component]
pub fn RecordsSection(rows: Vec<RecordRow>, groups: Vec<LaufGroup>) -> Element {
    if rows.is_empty() && groups.is_empty() {
        return rsx! {};
    }
    rsx! {
        section {
            h3 { "Rekorde" }
            if !rows.is_empty() {
                p { class: "legend-note", "Der Preis ist der Einzelwert je Gegner, so wie er in der Spieleliste steht." }
                for row in rows {
                    Kv { key: "{row.label}", label: row.label.clone(), value: row.value.clone() }
                    p { class: "muted small", "{row.detail}" }
                }
            }
            if !groups.is_empty() {
                h4 { "Wie viele Laufende?" }
                p { class: "legend-note", "Grün gewonnen, orange verloren (Spielerpartei). Gezählt wird ab der Mindestzahl im Tarif." }
                for group in groups {
                    h5 { key: "{group.title}", "{group.title}" }
                    {
                        let max = group.rows.iter().map(|(_, w, l)| w + l).max().unwrap_or(0);
                        rsx! {
                            for (label, won, lost) in group.rows.iter().cloned() {
                                StackedBar {
                                    key: "{group.title}-{label}",
                                    label: label,
                                    won: won,
                                    lost: lost,
                                    max: max,
                                    value: format!("{won} von {}", won + lost),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
