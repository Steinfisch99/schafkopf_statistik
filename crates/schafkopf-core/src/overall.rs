//! Auswertung über mehrere Abende. Die Spieler werden über ihren Namen zusammengeführt, denn die
//! Sitzplätze sind an jedem Abend andere.

use std::collections::BTreeMap;

use crate::clock::minutes_after_noon;
use crate::meta::{table_meta, UseCount};
use crate::model::{Cents, GameKind, PriceMistake};
use crate::scoring::price_doublings;
use crate::stats::{table_stats, KindStats, PlayerStats, RoleStats};
use crate::table::{OffenceKind, Table};

/// Ein Spieler über alle gewählten Abende.
#[derive(Debug, Clone, PartialEq)]
pub struct OverallPlayer {
    pub name: String,
    /// An wie vielen der Abende er mindestens ein Spiel gespielt hat.
    pub evenings: u32,
    pub balance: Cents,
    /// Sein Ergebnis an jedem Abend, an dem er gespielt hat, in der Reihenfolge der Abende.
    pub evening_balances: Vec<Cents>,
    pub games_played: u32,
    pub games_sat_out: u32,
    pub games_dealt: u32,
    pub as_declarer: RoleStats,
    pub as_partner: RoleStats,
    pub as_defender: RoleStats,
    pub best_game: Cents,
    pub worst_game: Cents,
    pub knocks: RoleStats,
    pub kontra_given: RoleStats,
    pub re_given: RoleStats,
    pub touts: RoleStats,
    pub sies: RoleStats,
    pub schneider_wins: u32,
    pub schwarz_wins: u32,
    pub lauf_own_side: u32,
    pub lauf_other_side: u32,
    pub wrong_demands: u32,
    pub wrong_by_kind: BTreeMap<PriceMistake, u32>,
    pub over_demands: u32,
    pub under_demands: u32,
    pub reclaims: u32,
    pub overcharge_gain: Cents,
    pub reclaim_returned: Cents,
    pub undercharge_missed: Cents,
    pub overcharge_loss: Cents,
    pub reclaim_saved: Cents,
    pub undercharge_benefit: Cents,
    pub longest_win_streak: u32,
    pub longest_loss_streak: u32,
    pub by_kind: BTreeMap<GameKind, KindStats>,
    /// Spiele, bei denen er mitgespielt hat und der Preis mindestens einmal (zweimal) verdoppelt war.
    pub doubled_games: u32,
    pub quadrupled_games: u32,
    /// In wie vielen seiner Spiele der Preis genau so oft verdoppelt war (0 = gar nicht, 1 = mal 2, 2 = mal 4, ...).
    pub by_doublings: BTreeMap<u32, u32>,
    /// Vergehen (ohne Neumischen), Neumischen, die Arten und die Strafen, die er gezahlt und bekommen hat.
    pub offences: u32,
    pub misdeals: u32,
    pub offences_by_kind: BTreeMap<OffenceKind, u32>,
    pub penalty_paid: Cents,
    pub penalty_received: Cents,
    /// Wann er an den Abenden kam und ging, in Minuten seit 12:00 Uhr mittags (siehe `minutes_after_noon`). Wer
    /// bis zum Ende blieb, geht mit dem Ende des Abends in die Zahlen ein.
    pub arrivals: Vec<i32>,
    pub departures: Vec<i32>,
    /// An wie vielen Abenden er später dazukam, früher ging, und wie oft er nach dem Gehen zurückkam.
    pub late_arrivals: u32,
    pub early_departures: u32,
    pub comebacks: u32,
}

impl OverallPlayer {
    fn new(name: &str) -> Self {
        OverallPlayer {
            name: name.to_owned(),
            evenings: 0,
            balance: 0,
            evening_balances: Vec::new(),
            games_played: 0,
            games_sat_out: 0,
            games_dealt: 0,
            as_declarer: RoleStats::default(),
            as_partner: RoleStats::default(),
            as_defender: RoleStats::default(),
            best_game: 0,
            worst_game: 0,
            knocks: RoleStats::default(),
            kontra_given: RoleStats::default(),
            re_given: RoleStats::default(),
            touts: RoleStats::default(),
            sies: RoleStats::default(),
            schneider_wins: 0,
            schwarz_wins: 0,
            lauf_own_side: 0,
            lauf_other_side: 0,
            wrong_demands: 0,
            wrong_by_kind: BTreeMap::new(),
            over_demands: 0,
            under_demands: 0,
            reclaims: 0,
            overcharge_gain: 0,
            reclaim_returned: 0,
            undercharge_missed: 0,
            overcharge_loss: 0,
            reclaim_saved: 0,
            undercharge_benefit: 0,
            longest_win_streak: 0,
            longest_loss_streak: 0,
            by_kind: BTreeMap::new(),
            doubled_games: 0,
            quadrupled_games: 0,
            by_doublings: BTreeMap::new(),
            offences: 0,
            misdeals: 0,
            offences_by_kind: BTreeMap::new(),
            penalty_paid: 0,
            penalty_received: 0,
            arrivals: Vec::new(),
            departures: Vec::new(),
            late_arrivals: 0,
            early_departures: 0,
            comebacks: 0,
        }
    }

    /// Der beste und der schlechteste Abend.
    pub fn best_evening(&self) -> Option<Cents> {
        self.evening_balances.iter().copied().max()
    }

    pub fn worst_evening(&self) -> Option<Cents> {
        self.evening_balances.iter().copied().min()
    }

    /// An wie vielen Abenden er im Plus lag.
    pub fn evenings_won(&self) -> u32 {
        self.evening_balances.iter().filter(|&&b| b > 0).count() as u32
    }

    /// Wie oft er selbst gedoppelt hat: geklopft, Kontra oder Re gegeben.
    pub fn doubles_given(&self) -> u32 {
        self.knocks.games + self.kontra_given.games + self.re_given.games
    }

    fn add(&mut self, p: &PlayerStats) {
        let role = |a: &mut RoleStats, b: &RoleStats| {
            a.games += b.games;
            a.wins += b.wins;
        };
        if p.games_played > 0 {
            self.evenings += 1;
            self.evening_balances.push(p.balance);
        }
        self.balance += p.balance;
        self.games_played += p.games_played;
        self.games_sat_out += p.games_sat_out;
        self.games_dealt += p.games_dealt;
        role(&mut self.as_declarer, &p.as_declarer);
        role(&mut self.as_partner, &p.as_partner);
        role(&mut self.as_defender, &p.as_defender);
        role(&mut self.knocks, &p.knocks);
        role(&mut self.kontra_given, &p.kontra_given);
        role(&mut self.re_given, &p.re_given);
        role(&mut self.touts, &p.touts);
        role(&mut self.sies, &p.sies);
        self.best_game = self.best_game.max(p.best_game);
        self.worst_game = self.worst_game.min(p.worst_game);
        self.schneider_wins += p.schneider_wins;
        self.schwarz_wins += p.schwarz_wins;
        self.lauf_own_side += p.lauf_own_side;
        self.lauf_other_side += p.lauf_other_side;
        self.wrong_demands += p.wrong_demands;
        self.over_demands += p.over_demands;
        self.under_demands += p.under_demands;
        self.reclaims += p.reclaims;
        self.overcharge_gain += p.overcharge_gain;
        self.reclaim_returned += p.reclaim_returned;
        self.undercharge_missed += p.undercharge_missed;
        self.overcharge_loss += p.overcharge_loss;
        self.reclaim_saved += p.reclaim_saved;
        self.undercharge_benefit += p.undercharge_benefit;
        for (m, n) in &p.wrong_by_kind {
            *self.wrong_by_kind.entry(*m).or_default() += n;
        }
        for (times, n) in &p.by_doublings {
            *self.by_doublings.entry(*times).or_default() += n;
        }
        self.offences += p.offences;
        self.misdeals += p.misdeals;
        self.penalty_paid += p.penalty_paid;
        self.penalty_received += p.penalty_received;
        for (k, n) in &p.offences_by_kind {
            *self.offences_by_kind.entry(*k).or_default() += n;
        }
        self.longest_win_streak = self.longest_win_streak.max(p.longest_win_streak);
        self.longest_loss_streak = self.longest_loss_streak.max(p.longest_loss_streak);
        for (kind, k) in &p.by_kind {
            let e = self.by_kind.entry(*kind).or_default();
            e.games += k.games;
            e.wins += k.wins;
            e.balance += k.balance;
        }
    }

    /// Alle Spiele des Spielers, Siege und Gewinn insgesamt.
    pub fn overall(&self) -> RoleStats {
        RoleStats {
            games: self.games_played,
            wins: self.as_declarer.wins + self.as_partner.wins + self.as_defender.wins,
        }
    }
}

/// Die Zahlen für alle gewählten Abende zusammen.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct OverallStats {
    pub evenings: u32,
    pub games: u32,
    pub turnover: Cents,
    /// Pro Spielart: Anzahl und Siege der Spielerpartei.
    pub by_kind: BTreeMap<GameKind, KindStats>,
    /// Wie viele Spiele wie oft verdoppelt waren: 0 = gar nicht, 1 = mal 2, 2 = mal 4, usw.
    pub by_doublings: BTreeMap<u32, u32>,
    pub declarer_side: RoleStats,
    pub knocked_games: u32,
    pub kontra_games: u32,
    pub re_games: u32,
    pub schneider_games: u32,
    pub schwarz_games: u32,
    pub corrected_games: u32,
    pub bock_games: u32,
    pub bock_started: u32,
    pub mistake_counts: BTreeMap<PriceMistake, u32>,
    pub overcharged_games: u32,
    pub undercharged_games: u32,
    pub reclaimed_games: u32,
    pub overcharge_cents: Cents,
    pub reclaimed_cents: Cents,
    /// Unterm Strich für die Gewinner bei zu hohem Preis: mehr bekommen (plus) oder durch das
    /// Zurückverlangen weniger als zustand (minus), summiert über alle Gewinner.
    pub overcharge_net_cents: Cents,
    /// Vergehen (ohne Neumischen), Neumischen, die Arten und das als Strafe bewegte Geld.
    pub offences: u32,
    pub misdeals: u32,
    pub offences_by_kind: BTreeMap<OffenceKind, u32>,
    pub penalty_cents: Cents,
    pub players: Vec<OverallPlayer>,
    /// Spiele je Tarif und je Regelwerk (die meisten zuerst) und wie oft sie von einem Spiel zum nächsten wechselten.
    pub tariffs: Vec<UseCount>,
    pub tariff_changes: u32,
    pub rule_sets: Vec<UseCount>,
    pub rule_set_changes: u32,
    /// Beginn und Dauer der Abende (Minuten seit 12:00 Uhr, Sekunden).
    pub evening_starts: Vec<i32>,
    pub evening_lengths: Vec<i64>,
    /// Eckdaten jedes Abends in der Reihenfolge der Abende: Beginn, Spiele, Dauer.
    pub evening_facts: Vec<EveningFact>,
}

impl OverallStats {
    /// Der Anteil der Spiele, die mindestens `times`-mal verdoppelt waren (1 = gedoppelt, 2 = vervierfacht).
    pub fn doubled_at_least(&self, times: u32) -> u32 {
        self.by_doublings
            .iter()
            .filter(|(n, _)| **n >= times)
            .map(|(_, c)| c)
            .sum()
    }
}

/// Wertet die Abende gemeinsam aus. Die Reihenfolge der Abende bestimmt die Reihenfolge der
/// Abendergebnisse eines Spielers.
pub fn overall_stats(tables: &[&Table]) -> OverallStats {
    let mut out = OverallStats::default();
    let mut index: BTreeMap<String, usize> = BTreeMap::new();
    for table in tables {
        let stats = table_stats(table);
        out.evenings += 1;
        out.games += stats.games;
        out.turnover += stats.turnover;
        for (kind, k) in &stats.by_kind {
            let e = out.by_kind.entry(*kind).or_default();
            e.games += k.games;
            e.wins += k.wins;
            e.balance += k.balance;
        }
        out.knocked_games += stats.knocked_games;
        out.kontra_games += stats.kontra_games;
        out.re_games += stats.re_games;
        out.schneider_games += stats.schneider_games;
        out.schwarz_games += stats.schwarz_games;
        out.corrected_games += stats.corrected_games;
        out.overcharged_games += stats.overcharged_games;
        out.undercharged_games += stats.undercharged_games;
        out.reclaimed_games += stats.reclaimed_games;
        out.overcharge_cents += stats.overcharge_cents;
        out.reclaimed_cents += stats.reclaimed_cents;
        out.overcharge_net_cents += stats.overcharge_net_cents;
        out.offences += stats.offences;
        out.misdeals += stats.misdeals;
        out.penalty_cents += stats.penalty_cents;
        for (k, n) in &stats.offences_by_kind {
            *out.offences_by_kind.entry(*k).or_default() += n;
        }
        out.bock_games += stats.bock_games;
        out.bock_started += stats.bock_started;
        for (m, n) in &stats.mistake_counts {
            *out.mistake_counts.entry(*m).or_default() += n;
        }
        for game in table.games() {
            let tariff = game.tariff.as_ref().unwrap_or(&table.tariff);
            let times = price_doublings(tariff, &game.input);
            *out.by_doublings.entry(times).or_default() += 1;
            if game.input.kind != GameKind::Ramsch {
                out.declarer_side.games += 1;
                out.declarer_side.wins += u32::from(game.input.won);
            }
            for &seat in &game.input.participants {
                let Some(name) = table.players.get(seat) else {
                    continue;
                };
                let i = *index.entry(name.clone()).or_insert_with(|| {
                    out.players.push(OverallPlayer::new(name));
                    out.players.len() - 1
                });
                out.players[i].doubled_games += u32::from(times >= 1);
                out.players[i].quadrupled_games += u32::from(times >= 2);
            }
        }
        for p in &stats.players {
            let i = *index.entry(p.name.clone()).or_insert_with(|| {
                out.players.push(OverallPlayer::new(&p.name));
                out.players.len() - 1
            });
            out.players[i].add(p);
        }
        let meta = table_meta(table);
        let merge = |into: &mut Vec<UseCount>, from: &[UseCount]| {
            for c in from {
                match into.iter_mut().find(|u| u.name == c.name) {
                    Some(u) => u.games += c.games,
                    None => into.push(c.clone()),
                }
            }
        };
        merge(&mut out.tariffs, &meta.tariffs);
        merge(&mut out.rule_sets, &meta.rule_sets);
        out.tariff_changes += meta.tariff_changes;
        out.rule_set_changes += meta.rule_set_changes;
        let offset = table.utc_offset_min;
        out.evening_facts.push(EveningFact {
            started: meta.started,
            utc_offset_min: offset,
            games: stats.games,
            length: meta.started.zip(meta.ended).map(|(s, e)| (e - s).max(0)),
        });
        if let Some(start) = meta.started {
            out.evening_starts.push(minutes_after_noon(start, offset));
            if let Some(end) = meta.ended {
                out.evening_lengths.push((end - start).max(0));
            }
        }
        for presence in &meta.presence {
            let Some(&i) = index.get(&presence.name) else {
                continue;
            };
            let p = &mut out.players[i];
            if let Some(at) = presence.arrived {
                p.arrivals.push(minutes_after_noon(at, offset));
            }
            let gone = if presence.left_early {
                presence.left
            } else {
                meta.ended
            };
            if let Some(at) = gone {
                p.departures.push(minutes_after_noon(at, offset));
            }
            p.late_arrivals += u32::from(!presence.from_start);
            p.early_departures += u32::from(presence.left_early);
            p.comebacks += presence.comebacks;
        }
    }
    out
}

/// Mittel, frühester und spätester Wert einer Liste von Uhrzeiten (Minuten seit 12:00 Uhr).
pub fn time_spread(times: &[i32]) -> Option<(i32, i32, i32)> {
    let min = *times.iter().min()?;
    let max = *times.iter().max()?;
    let avg = (times.iter().map(|&t| i64::from(t)).sum::<i64>() / times.len() as i64) as i32;
    Some((avg, min, max))
}

/// Die Eckdaten eines Abends für den Überblick über mehrere Abende.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EveningFact {
    /// Beginn in Unix-Sekunden, falls bekannt, und die Abweichung der Ortszeit von UTC.
    pub started: Option<i64>,
    pub utc_offset_min: i32,
    pub games: u32,
    /// Dauer in Sekunden (bis zum Ende oder zur letzten Aktivität), falls Beginn und Ende bekannt sind.
    pub length: Option<i64>,
}
