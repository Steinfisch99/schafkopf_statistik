//! Auswertung über mehrere Abende. Die Spieler werden über ihren Namen zusammengeführt, denn die
//! Sitzplätze sind an jedem Abend andere.

use std::collections::BTreeMap;

use crate::model::{Cents, GameKind, PriceMistake};
use crate::scoring::price_doublings;
use crate::stats::{table_stats, KindStats, PlayerStats, RoleStats};
use crate::table::Table;

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
    pub schneider_wins: u32,
    pub schwarz_wins: u32,
    pub lauf_own_side: u32,
    pub lauf_other_side: u32,
    pub wrong_demands: u32,
    pub wrong_by_kind: BTreeMap<PriceMistake, u32>,
    pub longest_win_streak: u32,
    pub longest_loss_streak: u32,
    pub by_kind: BTreeMap<GameKind, KindStats>,
    /// Spiele, bei denen er mitgespielt hat und der Preis mindestens einmal (zweimal) verdoppelt war.
    pub doubled_games: u32,
    pub quadrupled_games: u32,
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
            schneider_wins: 0,
            schwarz_wins: 0,
            lauf_own_side: 0,
            lauf_other_side: 0,
            wrong_demands: 0,
            wrong_by_kind: BTreeMap::new(),
            longest_win_streak: 0,
            longest_loss_streak: 0,
            by_kind: BTreeMap::new(),
            doubled_games: 0,
            quadrupled_games: 0,
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
        self.best_game = self.best_game.max(p.best_game);
        self.worst_game = self.worst_game.min(p.worst_game);
        self.schneider_wins += p.schneider_wins;
        self.schwarz_wins += p.schwarz_wins;
        self.lauf_own_side += p.lauf_own_side;
        self.lauf_other_side += p.lauf_other_side;
        self.wrong_demands += p.wrong_demands;
        for (m, n) in &p.wrong_by_kind {
            *self.wrong_by_kind.entry(*m).or_default() += n;
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
    pub players: Vec<OverallPlayer>,
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
        out.bock_games += stats.bock_games;
        out.bock_started += stats.bock_started;
        for (m, n) in &stats.mistake_counts {
            *out.mistake_counts.entry(*m).or_default() += n;
        }
        for game in table.games() {
            let tariff = game.tariff.as_ref().unwrap_or(&table.tariff);
            let times = price_doublings(tariff, &game.input);
            *out.by_doublings.entry(times).or_default() += 1;
            out.declarer_side.games += 1;
            out.declarer_side.wins += u32::from(game.input.won);
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
    }
    out
}
