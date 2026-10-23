//! Metadaten eines Abends: welcher Tarif und welches Regelwerk galten und wie oft sie wechselten,
//! und wer wann kam und ging.

use crate::model::Seat;
use crate::scoring::Tariff;
use crate::table::{EventKind, Table};

/// Wie viele Spiele unter einem Namen (Tarif oder Regelwerk) liefen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseCount {
    pub name: String,
    pub games: u32,
}

/// Wann ein Spieler an einem Abend am Tisch war.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Presence {
    pub seat: Seat,
    pub name: String,
    /// Von Anfang an dabei, sonst dazugekommen.
    pub from_start: bool,
    /// Wann er (das erste Mal) am Tisch war: der Beginn des Abends oder das Dazukommen. `None`, wenn
    /// die Zeit nicht bekannt ist (alte Dateien).
    pub arrived: Option<i64>,
    /// Wie viele Spiele schon gespielt waren, als er kam (0 = vor dem ersten Spiel).
    pub arrived_after_games: usize,
    /// Wann er zuletzt gegangen ist. `None`: Er war am Ende noch da.
    pub left: Option<i64>,
    /// Ob er am Ende nicht mehr am Tisch war (`left` kann trotzdem unbekannt sein).
    pub left_early: bool,
    pub left_after_games: Option<usize>,
    /// Wie oft er zwischendurch ging und wiederkam.
    pub comebacks: u32,
}

/// Die Metadaten eines Abends.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableMeta {
    /// Spiele je Tarif, die meisten zuerst.
    pub tariffs: Vec<UseCount>,
    /// Wie oft der Tarif von einem Spiel zum nächsten wechselte.
    pub tariff_changes: u32,
    pub rule_sets: Vec<UseCount>,
    pub rule_set_changes: u32,
    pub presence: Vec<Presence>,
    /// Beginn des Abends und sein Ende (bei einem offenen Abend die letzte bekannte Aktivität).
    pub started: Option<i64>,
    pub ended: Option<i64>,
}

fn tally(labels: &[String]) -> (Vec<UseCount>, u32) {
    let mut counts: Vec<UseCount> = Vec::new();
    for label in labels {
        match counts.iter_mut().find(|c| &c.name == label) {
            Some(c) => c.games += 1,
            None => counts.push(UseCount {
                name: label.clone(),
                games: 1,
            }),
        }
    }
    // Stabil sortieren: bei gleicher Zahl bleibt, was zuerst gespielt wurde, vorn.
    counts.sort_by_key(|c| std::cmp::Reverse(c.games));
    let changes = labels.windows(2).filter(|w| w[0] != w[1]).count() as u32;
    (counts, changes)
}

/// Der Name des Tarifs je Spiel. Spiele ohne Namen (eigener Tarif, ältere Dateien) heißen "Eigener Tarif",
/// bei mehreren verschiedenen "Eigener Tarif 1", "Eigener Tarif 2", ... nach ihrem ersten Auftreten.
pub(crate) fn tariff_labels(table: &Table) -> Vec<String> {
    let mut custom: Vec<&Tariff> = Vec::new();
    let mut raw: Vec<Result<String, &Tariff>> = Vec::new();
    for game in table.games() {
        let tariff = game.tariff.as_ref().unwrap_or(&table.tariff);
        let name = game.tariff_name.clone().or_else(|| {
            // Ältere Spiele kennen den Namen nicht: Passt der Tarif zum jetzigen, gilt dessen Name.
            (tariff == &table.tariff)
                .then(|| table.tariff_name.clone())
                .flatten()
        });
        match name {
            Some(n) => raw.push(Ok(n)),
            None => {
                if !custom.contains(&tariff) {
                    custom.push(tariff);
                }
                raw.push(Err(tariff));
            }
        }
    }
    raw.into_iter()
        .map(|r| match r {
            Ok(n) => n,
            Err(_) if custom.len() == 1 => "Eigener Tarif".to_owned(),
            Err(t) => {
                let i = custom.iter().position(|c| *c == t).unwrap_or(0);
                format!("Eigener Tarif {}", i + 1)
            }
        })
        .collect()
}

pub(crate) fn rule_set_labels(table: &Table) -> Vec<String> {
    table
        .games()
        .iter()
        .map(|g| {
            g.rule_set_name
                .clone()
                .or_else(|| table.rule_set_name.clone())
                .unwrap_or_else(|| "Eigene Auswahl".to_owned())
        })
        .collect()
}

/// Wertet die Metadaten eines Abends aus.
pub fn table_meta(table: &Table) -> TableMeta {
    let (tariffs, tariff_changes) = tally(&tariff_labels(table));
    let (rule_sets, rule_set_changes) = tally(&rule_set_labels(table));
    let presence = (0..table.players.len())
        .map(|seat| {
            let joined = table
                .events
                .iter()
                .find(|e| e.seat == seat && e.kind == EventKind::Joined);
            let mut p = Presence {
                seat,
                name: table.players[seat].clone(),
                from_start: joined.is_none(),
                arrived: joined.map_or(table.created_at, |e| e.at),
                arrived_after_games: joined.map_or(0, |e| e.after_games),
                left: None,
                left_early: false,
                left_after_games: None,
                comebacks: 0,
            };
            for e in table.events.iter().filter(|e| e.seat == seat) {
                match e.kind {
                    EventKind::Left => {
                        p.left_early = true;
                        p.left = e.at;
                        p.left_after_games = Some(e.after_games);
                    }
                    EventKind::Returned => {
                        p.left_early = false;
                        p.left = None;
                        p.left_after_games = None;
                        p.comebacks += 1;
                    }
                    EventKind::Joined | EventKind::Moved => {}
                }
            }
            p
        })
        .collect();
    TableMeta {
        tariffs,
        tariff_changes,
        rule_sets,
        rule_set_changes,
        presence,
        started: table.created_at,
        ended: table.ended_at.or_else(|| table.last_activity()),
    }
}
