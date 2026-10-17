use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{Cents, GameKind, Seat, Suit};
use crate::table::Table;

/// Spiele und Siege in einer Rolle, Position oder Kategorie.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleStats {
    pub games: u32,
    pub wins: u32,
}

impl RoleStats {
    fn record(&mut self, won: bool) {
        self.games += 1;
        self.wins += u32::from(won);
    }

    /// Siegquote zwischen 0 und 1, `None` ohne Spiele.
    pub fn win_rate(&self) -> Option<f64> {
        (self.games > 0).then(|| f64::from(self.wins) / f64::from(self.games))
    }
}

/// Pro Spielart. Beim Spieler: seine Spiele, Siege und sein Gewinn.
/// Am Tisch: Anzahl, Siege der Spielerpartei und Summe der Einzelwerte.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KindStats {
    pub games: u32,
    pub wins: u32,
    pub balance: Cents,
}

/// Läufe und Gegenläufe einer Spielart. Gezählt wird nur, was nach dem Tarif des Spiels ein Lauf
/// ist, also ab der Mindestzahl an Laufenden dieser Spielart.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaufendeStats {
    /// Alle Spiele dieser Art.
    pub games: u32,
    /// Spiele, in denen die Spielerpartei die Laufenden hatte (Lauf).
    pub lauf: u32,
    /// Spiele, in denen die Gegenpartei sie hatte (Gegenlauf).
    pub gegenlauf: u32,
    /// Summe der Laufenden in den Lauf- und in den Gegenlauf-Spielen.
    pub lauf_sum: u32,
    pub gegenlauf_sum: u32,
}

impl LaufendeStats {
    /// Spiele ohne Lauf und ohne Gegenlauf.
    pub fn without(&self) -> u32 {
        self.games - self.lauf - self.gegenlauf
    }

    /// Durchschnittliche Zahl Laufende in den Lauf-Spielen.
    pub fn average_lauf(&self) -> Option<f64> {
        (self.lauf > 0).then(|| f64::from(self.lauf_sum) / f64::from(self.lauf))
    }

    /// Durchschnittliche Zahl Laufende in den Gegenlauf-Spielen.
    pub fn average_gegenlauf(&self) -> Option<f64> {
        (self.gegenlauf > 0).then(|| f64::from(self.gegenlauf_sum) / f64::from(self.gegenlauf))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerStats {
    pub seat: Seat,
    pub name: String,
    pub balance: Cents,
    pub games_played: u32,
    pub games_sat_out: u32,
    pub games_dealt: u32,
    pub as_declarer: RoleStats,
    pub as_partner: RoleStats,
    /// Gegenspieler (beim Ramsch alle, die nicht verloren haben).
    pub as_defender: RoleStats,
    /// Größter Gewinn und größter Verlust in einem einzelnen Spiel.
    pub best_game: Cents,
    pub worst_game: Cents,
    /// Geklopft: wie oft, und in wie vielen dieser Spiele hat man gewonnen?
    pub knocks: RoleStats,
    pub kontra_given: RoleStats,
    pub re_given: RoleStats,
    pub schneider_wins: u32,
    pub schwarz_wins: u32,
    /// Spiele mit einem Lauf oder Gegenlauf, bei denen die eigene Seite die Laufenden hatte, und
    /// solche, bei denen die andere Seite sie hatte.
    pub lauf_own_side: u32,
    pub lauf_other_side: u32,
    pub longest_win_streak: u32,
    pub longest_loss_streak: u32,
    /// Positiv = aktuelle Siegesserie, negativ = aktuelle Verlustserie.
    pub current_streak: i32,
    pub by_kind: BTreeMap<GameKind, KindStats>,
    /// Ergebnis nach Platz unter den Mitspielenden, gezählt ab links vom Geber: 0 = Vorhand, dann
    /// Mittelhand, Hinterhand und 3 = der Letzte (bei vier Spielern der Geber).
    pub by_position: [RoleStats; 4],
}

impl PlayerStats {
    fn new(seat: Seat, name: &str) -> Self {
        PlayerStats {
            seat,
            name: name.to_owned(),
            balance: 0,
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
            longest_win_streak: 0,
            longest_loss_streak: 0,
            current_streak: 0,
            by_kind: BTreeMap::new(),
            by_position: [RoleStats::default(); 4],
        }
    }

    pub fn win_rate(&self) -> Option<f64> {
        let wins = self.as_declarer.wins + self.as_partner.wins + self.as_defender.wins;
        (self.games_played > 0).then(|| f64::from(wins) / f64::from(self.games_played))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TableStats {
    pub games: u32,
    /// Summe aller Gewinne, also das insgesamt bewegte Geld.
    pub turnover: Cents,
    /// Teuerstes Spiel als (Index in `Table::games`, Einzelwert).
    pub most_expensive: Option<(usize, Cents)>,
    pub by_kind: BTreeMap<GameKind, KindStats>,
    /// Sauspiele nach gerufener Sau.
    pub by_called_ace: BTreeMap<Suit, RoleStats>,
    pub knocked_games: u32,
    pub kontra_games: u32,
    pub re_games: u32,
    pub schneider_games: u32,
    pub schwarz_games: u32,
    /// Spiele, in denen ein anderer Preis verlangt wurde, als der Tarif ergibt.
    pub corrected_games: u32,
    /// Spiele, die Bockspiele waren (doppelter Preis), und Spiele, die eine Bockrunde ausgelöst haben.
    pub bock_games: u32,
    pub bock_started: u32,
    /// Läufe und Gegenläufe je Spielart, nur für Spielarten mit Zuschlägen.
    pub laufende_by_kind: BTreeMap<GameKind, LaufendeStats>,
    pub players: Vec<PlayerStats>,
    /// `partner_matrix[a][b]`: Sauspiele/Hochzeiten, in denen a und b zusammen gespielt haben.
    pub partner_matrix: Vec<Vec<RoleStats>>,
}

/// Wertet einen ganzen Tisch aus.
pub fn table_stats(table: &Table) -> TableStats {
    let n = table.players.len();
    let ring = table.ring();
    let ring_pos = |s: Seat| ring.iter().position(|&r| r == s).unwrap_or(s);
    let mut players: Vec<PlayerStats> = table
        .players
        .iter()
        .enumerate()
        .map(|(s, name)| PlayerStats::new(s, name))
        .collect();
    let mut stats = TableStats {
        games: 0,
        turnover: 0,
        most_expensive: None,
        by_kind: BTreeMap::new(),
        by_called_ace: BTreeMap::new(),
        knocked_games: 0,
        kontra_games: 0,
        re_games: 0,
        schneider_games: 0,
        schwarz_games: 0,
        corrected_games: 0,
        bock_games: 0,
        bock_started: 0,
        laufende_by_kind: BTreeMap::new(),
        players: Vec::new(),
        partner_matrix: vec![vec![RoleStats::default(); n]; n],
    };

    for (idx, game) in table.games().iter().enumerate() {
        let g = &game.input;
        let active = &g.participants;
        stats.games += 1;
        stats.turnover += game.payouts.iter().filter(|&&p| p > 0).sum::<Cents>();
        if stats.most_expensive.is_none_or(|(_, v)| game.value > v) {
            stats.most_expensive = Some((idx, game.value));
        }
        let kind = stats.by_kind.entry(g.kind).or_default();
        kind.games += 1;
        kind.wins += u32::from(g.won);
        kind.balance += game.value;
        if let (GameKind::Sauspiel, Some(suit)) = (g.kind, g.suit) {
            stats.by_called_ace.entry(suit).or_default().record(g.won);
        }
        stats.knocked_games += u32::from(!g.knockers.is_empty());
        stats.kontra_games += u32::from(g.kontra_by.is_some());
        stats.re_games += u32::from(g.re);
        stats.schneider_games += u32::from(g.schneider || g.schwarz);
        stats.schwarz_games += u32::from(g.schwarz);
        stats.corrected_games += u32::from(game.value != game.calculated_value);
        stats.bock_games += u32::from(g.bock);
        stats.bock_started += u32::from(g.starts_bock);
        if let Some(partner) = g.partner {
            stats.partner_matrix[g.declarer][partner].record(g.won);
            stats.partner_matrix[partner][g.declarer].record(g.won);
        }
        let lauf_counted = g.kind.has_extras()
            && g.laufende > 0
            && g.laufende
                >= game
                    .tariff
                    .as_ref()
                    .unwrap_or(&table.tariff)
                    .laufende_min(g.kind);
        if g.kind.has_extras() {
            let entry = stats.laufende_by_kind.entry(g.kind).or_default();
            entry.games += 1;
            if lauf_counted && g.gegenlauf {
                entry.gegenlauf += 1;
                entry.gegenlauf_sum += u32::from(g.laufende);
            } else if lauf_counted {
                entry.lauf += 1;
                entry.lauf_sum += u32::from(g.laufende);
            }
        }

        players[g.dealer].games_dealt += 1;
        for (seat, p) in players.iter_mut().enumerate() {
            if !table.was_at_table(game, seat) {
                continue;
            }
            if !active.contains(&seat) {
                p.games_sat_out += 1;
                continue;
            }
            // Wer im Team der Spielerpartei ist, gewinnt mit ihr; alle anderen gewinnen, wenn sie verliert.
            let on_team = g.is_on_declarer_team(seat);
            let won = on_team == g.won;
            let payout = game.payouts[seat];
            if lauf_counted {
                // Die Seite, die die Laufende hatte: die Spielerpartei bei einem Lauf, sonst die Gegner.
                if on_team != g.gegenlauf {
                    p.lauf_own_side += 1;
                } else {
                    p.lauf_other_side += 1;
                }
            }

            p.games_played += 1;
            p.balance += payout;
            p.best_game = p.best_game.max(payout);
            p.worst_game = p.worst_game.min(payout);
            if seat == g.declarer {
                p.as_declarer.record(won);
            } else if g.partner == Some(seat) {
                p.as_partner.record(won);
            } else {
                p.as_defender.record(won);
            }
            if g.knockers.contains(&seat) {
                p.knocks.record(won);
            }
            if g.kontra_by == Some(seat) {
                p.kontra_given.record(won);
            }
            if g.re && seat == g.declarer {
                p.re_given.record(won);
            }
            if won {
                p.schneider_wins += u32::from(g.schneider || g.schwarz);
                p.schwarz_wins += u32::from(g.schwarz);
            }

            let k = p.by_kind.entry(g.kind).or_default();
            k.games += 1;
            k.wins += u32::from(won);
            k.balance += payout;

            let order = |s: Seat| (ring_pos(s) + n - ring_pos(g.dealer) - 1) % n;
            let position = active.iter().filter(|&&s| order(s) < order(seat)).count();
            p.by_position[position].record(won);

            p.current_streak = match (won, p.current_streak) {
                (true, s) if s > 0 => s + 1,
                (true, _) => 1,
                (false, s) if s < 0 => s - 1,
                (false, _) => -1,
            };
            if won {
                p.longest_win_streak = p.longest_win_streak.max(p.current_streak as u32);
            } else {
                p.longest_loss_streak = p.longest_loss_streak.max(p.current_streak.unsigned_abs());
            }
        }
    }
    stats.players = players;
    stats
}

/// Kontostand pro Sitzplatz für einen Verlaufsgraphen.
///
/// Der erste Eintrag ist der Start bei 0, danach folgt der Stand nach jedem Spiel. Der Graph hat
/// also eine Position mehr als es Spiele gibt, und Kontostände können negativ sein.
pub fn balance_history(table: &Table) -> Vec<Vec<Cents>> {
    let mut sum = vec![0; table.players.len()];
    let mut history = vec![sum.clone()];
    for g in table.games() {
        for (s, p) in sum.iter_mut().zip(&g.payouts) {
            *s += p;
        }
        history.push(sum.clone());
    }
    history
}
