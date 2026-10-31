use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::model::{Cents, GameKind, PriceMistake, Seat, Suit};
use crate::scoring::payouts;
use crate::scoring::price_doublings;
use crate::table::{Game, Table};
use crate::table::{OffenceKind, StockMoveKind};

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

/// Spiele mit genau so vielen Laufenden: die der Spielerpartei (Lauf) und die der Gegenpartei (Gegenlauf),
/// jeweils mit den Siegen der Spielerpartei. Gezählt wird nur ab der Mindestzahl des Tarifs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaufCount {
    pub lauf: RoleStats,
    pub gegenlauf: RoleStats,
}

impl LaufCount {
    pub fn add(&mut self, other: &LaufCount) {
        self.lauf.games += other.lauf.games;
        self.lauf.wins += other.lauf.wins;
        self.gegenlauf.games += other.gegenlauf.games;
        self.gegenlauf.wins += other.gegenlauf.wins;
    }
}

/// Zahlen zum Stock eines Abends (oder mehrerer Abende zusammen), alles in Cent.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StockStats {
    /// Wie oft keiner spielen wollte (Zusammenwerfen und Stock einzahlen zusammen).
    pub no_game_rounds: u32,
    /// Davon: nur zusammengeworfen (kein Geld bewegt) und mit Einlage in den Stock.
    #[serde(default)]
    pub reshuffles: u32,
    #[serde(default)]
    pub deposit_rounds: u32,
    /// Wie oft der Stock an alle aufgeteilt wurde und wie viel dabei verteilt wurde.
    #[serde(default)]
    pub splits: u32,
    #[serde(default)]
    pub split_out: Cents,
    /// Der höchste Stand, den der Stock erreicht hat (bei mehreren Abenden: der höchste aller Abende).
    pub peak: Cents,
    /// Was als Einlage hineinkam, und was durch das Doppeln verlorener Sauspiele dazukam.
    pub deposited: Cents,
    pub doubled: Cents,
    /// Was an Gewinner ausgezahlt wurde.
    pub paid_out: Cents,
    /// Wie oft ausgezahlt und wie oft gedoppelt wurde.
    pub payouts: u32,
    pub doublings: u32,
    /// Was am Ende noch im Stock lag (bei mehreren Abenden: die Summe der Reste).
    pub left: Cents,
}

impl StockStats {
    /// Alles Geld, das in den Stock floss.
    pub fn flowed_in(&self) -> Cents {
        self.deposited + self.doubled
    }

    /// Wurde der Stock je benutzt?
    pub fn is_empty(&self) -> bool {
        self == &StockStats::default()
    }

    pub(crate) fn add(&mut self, other: &StockStats) {
        self.no_game_rounds += other.no_game_rounds;
        self.reshuffles += other.reshuffles;
        self.deposit_rounds += other.deposit_rounds;
        self.splits += other.splits;
        self.split_out += other.split_out;
        self.peak = self.peak.max(other.peak);
        self.deposited += other.deposited;
        self.doubled += other.doubled;
        self.paid_out += other.paid_out;
        self.payouts += other.payouts;
        self.doublings += other.doublings;
        self.left += other.left;
    }
}

/// Was ein Spieler im Stock gezahlt und bekommen hat, in Cent (alles positiv).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerStock {
    /// Einlagen, wenn keiner spielte.
    pub deposited: Cents,
    /// Was er beim Doppeln eines verlorenen Sauspiels nachlegen musste.
    pub doubled: Cents,
    /// Was er aus dem Stock bekommen hat.
    pub received: Cents,
    /// Was er beim Aufteilen des Stocks bekommen hat.
    #[serde(default)]
    pub split: Cents,
    /// Wie oft er den Stock gewonnen und wie oft er ihn gedoppelt hat.
    pub wins: u32,
    pub doublings: u32,
}

impl PlayerStock {
    /// Alles, was er in den Stock gezahlt hat.
    pub fn paid(&self) -> Cents {
        self.deposited + self.doubled
    }

    /// Unterm Strich aus dem Stock: bekommen minus gezahlt.
    pub fn net(&self) -> Cents {
        self.received + self.split - self.paid()
    }

    pub fn is_empty(&self) -> bool {
        self == &PlayerStock::default()
    }

    pub(crate) fn add(&mut self, other: &PlayerStock) {
        self.deposited += other.deposited;
        self.doubled += other.doubled;
        self.received += other.received;
        self.split += other.split;
        self.wins += other.wins;
        self.doublings += other.doublings;
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
    /// Wie oft er gegeben hat, ohne dass gespielt wurde (Zusammenwerfen oder Stock einzahlen).
    #[serde(default)]
    pub deals_without_game: u32,
    pub as_declarer: RoleStats,
    pub as_partner: RoleStats,
    /// Gegenspieler (beim Ramsch alle, die nicht verloren haben).
    pub as_defender: RoleStats,
    /// Größter Gewinn und größter Verlust in einem einzelnen Spiel.
    pub best_game: Cents,
    pub worst_game: Cents,
    /// Geklopft: wie oft, und in wie vielen dieser Spiele hat man gewonnen?
    pub knocks: RoleStats,
    /// Wie oft der Spieler den Preis falsch verlangt hat (der Preis wurde danach korrigiert), und was dabei
    /// schiefging. Ein Spiel mit mehreren Fehlerarten zählt bei jeder.
    pub wrong_demands: u32,
    pub wrong_by_kind: BTreeMap<PriceMistake, u32>,
    /// Wie oft er als Gewinner zu viel und zu wenig verlangt hat.
    pub over_demands: u32,
    pub under_demands: u32,
    /// Wie oft er als Verlierer zu viel Verlangtes zurückverlangt hat.
    pub reclaims: u32,
    /// Geld durch falsch verlangte Preise, in Cent: als Gewinner mehr bekommen (nicht aufgefallen),
    /// zurückgeben müssen (zurückverlangt) oder weniger bekommen als zustand; als Verlierer zu viel
    /// gezahlt, durch Zurückverlangen gespart oder weniger gezahlt als zustand.
    pub overcharge_gain: Cents,
    pub reclaim_returned: Cents,
    pub undercharge_missed: Cents,
    pub overcharge_loss: Cents,
    pub reclaim_saved: Cents,
    pub undercharge_benefit: Cents,
    pub kontra_given: RoleStats,
    pub re_given: RoleStats,
    /// Tout gespielt (auch Sie): wie oft, und wie oft gewonnen. Sie zählt zusätzlich einzeln.
    #[serde(default)]
    pub touts: RoleStats,
    #[serde(default)]
    pub sies: RoleStats,
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
    /// In wie vielen seiner Spiele der Preis genau so oft verdoppelt war (0 = gar nicht, 1 = mal 2, 2 = mal 4, ...).
    pub by_doublings: BTreeMap<u32, u32>,
    /// Vergehen (ohne Neumischen) und Neumischen des Spielers, und alle Arten zusammen.
    pub offences: u32,
    pub misdeals: u32,
    pub offences_by_kind: BTreeMap<OffenceKind, u32>,
    /// Strafen, die er gezahlt und bekommen hat, in Cent (beides positiv).
    pub penalty_paid: Cents,
    pub penalty_received: Cents,
    /// Was er in den Stock gezahlt und daraus bekommen hat.
    #[serde(default)]
    pub stock: PlayerStock,
}

impl PlayerStats {
    /// Wie oft er selbst gedoppelt hat: geklopft, Kontra oder Re gegeben.
    pub fn doubles_given(&self) -> u32 {
        self.knocks.games + self.kontra_given.games + self.re_given.games
    }

    fn new(seat: Seat, name: &str) -> Self {
        PlayerStats {
            seat,
            name: name.to_owned(),
            balance: 0,
            games_played: 0,
            games_sat_out: 0,
            games_dealt: 0,
            deals_without_game: 0,
            as_declarer: RoleStats::default(),
            as_partner: RoleStats::default(),
            as_defender: RoleStats::default(),
            best_game: 0,
            worst_game: 0,
            knocks: RoleStats::default(),
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
            kontra_given: RoleStats::default(),
            re_given: RoleStats::default(),
            touts: RoleStats::default(),
            sies: RoleStats::default(),
            schneider_wins: 0,
            schwarz_wins: 0,
            lauf_own_side: 0,
            lauf_other_side: 0,
            longest_win_streak: 0,
            longest_loss_streak: 0,
            current_streak: 0,
            by_kind: BTreeMap::new(),
            by_position: [RoleStats::default(); 4],
            by_doublings: BTreeMap::new(),
            offences: 0,
            misdeals: 0,
            offences_by_kind: BTreeMap::new(),
            penalty_paid: 0,
            penalty_received: 0,
            stock: PlayerStock::default(),
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
    /// Summe aller Gewinne (auch aus dem Stock), also das insgesamt bewegte Geld.
    pub turnover: Cents,
    /// Teuerstes Spiel als (Index in `Table::games`, Einzelwert).
    pub most_expensive: Option<(usize, Cents)>,
    /// Teuerstes Spiel je Spielart als (Index in `Table::games`, Einzelwert).
    #[serde(default)]
    pub most_expensive_by_kind: BTreeMap<GameKind, (usize, Cents)>,
    /// Das Spiel mit den meisten Laufenden (Lauf oder Gegenlauf) als (Index, Anzahl).
    #[serde(default)]
    pub most_laufende: Option<(usize, u8)>,
    /// Je Spielart: wie oft es wie viele Laufende gab (Schlüssel = Anzahl).
    #[serde(default)]
    pub laufende_counts: BTreeMap<GameKind, BTreeMap<u8, LaufCount>>,
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
    /// Wie oft welche Fehlerart bei korrigierten Preisen vorkam.
    pub mistake_counts: BTreeMap<PriceMistake, u32>,
    /// Spiele, in denen zu viel (zu wenig) verlangt wurde, und wie oft davon zurückverlangt wurde.
    pub overcharged_games: u32,
    pub undercharged_games: u32,
    pub reclaimed_games: u32,
    /// Das zu viel verlangte Geld insgesamt und das davon zurückverlangte, in Cent.
    pub overcharge_cents: Cents,
    pub reclaimed_cents: Cents,
    /// Unterm Strich für die Gewinner bei zu hohem Preis: mehr bekommen (plus) oder durch das
    /// Zurückverlangen weniger als zustand (minus), summiert über alle Gewinner.
    pub overcharge_net_cents: Cents,
    /// Spiele, die Bockspiele waren (doppelter Preis), und Spiele, die eine Bockrunde ausgelöst haben.
    pub bock_games: u32,
    pub bock_started: u32,
    /// Läufe und Gegenläufe je Spielart, nur für Spielarten mit Zuschlägen.
    pub laufende_by_kind: BTreeMap<GameKind, LaufendeStats>,
    /// Wie viele Spiele wie oft verdoppelt waren: 0 = gar nicht, 1 = mal 2, 2 = mal 4, 3 = mal 8, ...
    pub by_doublings: BTreeMap<u32, u32>,
    /// Vergehen (ohne Neumischen), Neumischen, die Arten und das Geld, das als Strafe bewegt wurde.
    pub offences: u32,
    pub misdeals: u32,
    pub offences_by_kind: BTreeMap<OffenceKind, u32>,
    pub penalty_cents: Cents,
    /// Der Stock: Einlagen, Höchststand, Auszahlungen.
    pub stock: StockStats,
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
        most_expensive_by_kind: BTreeMap::new(),
        most_laufende: None,
        laufende_counts: BTreeMap::new(),
        by_kind: BTreeMap::new(),
        by_called_ace: BTreeMap::new(),
        knocked_games: 0,
        kontra_games: 0,
        re_games: 0,
        schneider_games: 0,
        schwarz_games: 0,
        corrected_games: 0,
        mistake_counts: BTreeMap::new(),
        overcharged_games: 0,
        undercharged_games: 0,
        reclaimed_games: 0,
        overcharge_cents: 0,
        reclaimed_cents: 0,
        overcharge_net_cents: 0,
        bock_games: 0,
        bock_started: 0,
        laufende_by_kind: BTreeMap::new(),
        by_doublings: BTreeMap::new(),
        offences: 0,
        misdeals: 0,
        offences_by_kind: BTreeMap::new(),
        penalty_cents: 0,
        stock: StockStats::default(),
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
        let priciest = stats
            .most_expensive_by_kind
            .entry(g.kind)
            .or_insert((idx, game.value));
        if game.value > priciest.1 {
            *priciest = (idx, game.value);
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
        stats.corrected_games += u32::from(game.demanded_wrong());
        if game.demanded_wrong() {
            for &m in &g.mistakes {
                *stats.mistake_counts.entry(m).or_default() += 1;
            }
            for &by in &g.price_error_by {
                let p = &mut players[by];
                p.wrong_demands += 1;
                for &m in &g.mistakes {
                    *p.wrong_by_kind.entry(m).or_default() += 1;
                }
            }
            let over = game.overcharged();
            let reclaimed = game.input.reclaimed;
            stats.overcharged_games += u32::from(over);
            stats.undercharged_games += u32::from(!over);
            stats.reclaimed_games += u32::from(reclaimed);
            for &by in &g.price_error_by {
                let p = &mut players[by];
                if over {
                    p.over_demands += 1;
                } else {
                    p.under_demands += 1;
                }
            }
            for &by in &g.reclaimed_by {
                players[by].reclaims += 1;
            }
            // Was jeder dadurch mehr (plus) oder weniger (minus) bekam, als er sollte: verlangt gegen
            // berechnet, und beim Zurückverlangen der tatsächlich gezahlte gegen den verlangten Preis.
            let (asked, right) = demand_payouts(table, game);
            let winners = g.winners();
            for (seat, (&a, &r)) in asked.iter().zip(&right).enumerate() {
                let p = &mut players[seat];
                let diff = a - r;
                let winner = winners.contains(&seat);
                if over {
                    if winner && diff > 0 {
                        stats.overcharge_cents += diff;
                    }
                    if reclaimed {
                        let back = a - game.payouts[seat];
                        match back.signum() {
                            1 => {
                                p.reclaim_returned += back;
                                stats.reclaimed_cents += back;
                            }
                            -1 => p.reclaim_saved -= back,
                            _ => {}
                        }
                        if winner {
                            stats.overcharge_net_cents += game.payouts[seat] - r;
                        }
                    } else {
                        match diff.signum() {
                            1 => p.overcharge_gain += diff,
                            -1 => p.overcharge_loss -= diff,
                            _ => {}
                        }
                        if winner {
                            stats.overcharge_net_cents += diff;
                        }
                    }
                } else {
                    match diff.signum() {
                        -1 => p.undercharge_missed -= diff,
                        1 => p.undercharge_benefit += diff,
                        _ => {}
                    }
                }
            }
        }
        let times = price_doublings(game.tariff.as_ref().unwrap_or(&table.tariff), g);
        *stats.by_doublings.entry(times).or_default() += 1;
        stats.bock_games += u32::from(g.bock);
        stats.bock_started += g.bock_triggers();
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
            let counts = stats
                .laufende_counts
                .entry(g.kind)
                .or_default()
                .entry(if lauf_counted { g.laufende } else { 0 })
                .or_default();
            if lauf_counted && g.gegenlauf {
                entry.gegenlauf += 1;
                entry.gegenlauf_sum += u32::from(g.laufende);
                counts.gegenlauf.record(g.won);
            } else if lauf_counted {
                entry.lauf += 1;
                entry.lauf_sum += u32::from(g.laufende);
                counts.lauf.record(g.won);
            } else {
                counts.lauf.record(g.won);
            }
            if lauf_counted && stats.most_laufende.is_none_or(|(_, n)| g.laufende > n) {
                stats.most_laufende = Some((idx, g.laufende));
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
            *p.by_doublings.entry(times).or_default() += 1;
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
            if g.tout && seat == g.declarer {
                p.touts.record(won);
                if g.sie {
                    p.sies.record(won);
                }
            }
            if g.re_giver() == Some(seat) {
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
    for o in &table.offences {
        if o.kind == OffenceKind::Misdeal {
            stats.misdeals += 1;
        } else {
            stats.offences += 1;
        }
        *stats.offences_by_kind.entry(o.kind).or_default() += 1;
        stats.penalty_cents += o.payouts.iter().filter(|&&p| p > 0).sum::<Cents>();
        if let Some(p) = players.get_mut(o.offender) {
            if o.kind == OffenceKind::Misdeal {
                p.misdeals += 1;
            } else {
                p.offences += 1;
            }
            *p.offences_by_kind.entry(o.kind).or_default() += 1;
        }
        for (p, &payout) in players.iter_mut().zip(&o.payouts) {
            p.balance += payout;
            if payout < 0 {
                p.penalty_paid -= payout;
            } else {
                p.penalty_received += payout;
            }
        }
    }
    for m in &table.stock_moves {
        for (p, &payout) in players.iter_mut().zip(&m.payouts) {
            p.balance += payout;
        }
    }
    let mut pot: Cents = 0;
    for m in &table.stock_moves {
        pot += m.pot_change;
        stats.stock.peak = stats.stock.peak.max(pot);
        stats.turnover += m.payouts.iter().filter(|&&p| p > 0).sum::<Cents>();
        match m.kind {
            StockMoveKind::NoGame => {
                stats.stock.no_game_rounds += 1;
                stats.stock.deposited += m.pot_change;
                if m.is_reshuffle() {
                    stats.stock.reshuffles += 1;
                } else {
                    stats.stock.deposit_rounds += 1;
                }
            }
            StockMoveKind::Payout => {
                stats.stock.payouts += 1;
                stats.stock.paid_out -= m.pot_change;
            }
            StockMoveKind::Double => {
                stats.stock.doublings += 1;
                stats.stock.doubled += m.pot_change;
            }
            StockMoveKind::Split => {
                stats.stock.splits += 1;
                stats.stock.split_out -= m.pot_change;
            }
        }
        for (p, &payout) in players.iter_mut().zip(&m.payouts) {
            match (m.kind, payout) {
                (_, 0) => {}
                (StockMoveKind::NoGame, payout) => p.stock.deposited -= payout,
                (StockMoveKind::Payout, payout) => {
                    p.stock.received += payout;
                    p.stock.wins += 1;
                }
                (StockMoveKind::Double, payout) => {
                    p.stock.doubled -= payout;
                    p.stock.doublings += 1;
                }
                (StockMoveKind::Split, payout) => p.stock.split += payout,
            }
        }
    }
    for dealer in table.pass_dealers() {
        if let Some(p) = players.get_mut(dealer) {
            p.deals_without_game += 1;
        }
    }
    stats.stock.left = pot;
    stats.players = players;
    stats
}

/// Kontostand pro Sitzplatz für einen Verlaufsgraphen.
///
/// Der erste Eintrag ist der Start bei 0, danach folgt der Stand nach jedem Spiel. Der Graph hat
/// also eine Position mehr als es Spiele gibt, und Kontostände können negativ sein. Strafzahlungen
/// stecken in dem Stand, der zu dem Zeitpunkt gehört, an dem sie anfielen: Vor dem ersten Spiel im
/// Startwert, sonst im Stand nach dem letzten Spiel davor.
pub fn balance_history(table: &Table) -> Vec<Vec<Cents>> {
    let games = table.games().len();
    let mut sum = vec![0; table.players.len()];
    let mut history = Vec::with_capacity(games + 1);
    for step in 0..=games {
        if step > 0 {
            for (s, p) in sum.iter_mut().zip(&table.games()[step - 1].payouts) {
                *s += p;
            }
        }
        for o in table
            .offences
            .iter()
            .filter(|o| o.after_games.min(games) == step)
        {
            for (s, p) in sum.iter_mut().zip(&o.payouts) {
                *s += p;
            }
        }
        for m in table
            .stock_moves
            .iter()
            .filter(|m| m.after_games.min(games) == step)
        {
            for (s, p) in sum.iter_mut().zip(&m.payouts) {
                *s += p;
            }
        }
        history.push(sum.clone());
    }
    history
}

/// Die Auszahlungen eines Spiels mit dem verlangten Preis und mit dem berechneten Preis, beide ohne
/// Zurückverlangen. Leer, wenn der Preis nicht abwich.
fn demand_payouts(table: &Table, game: &Game) -> (Vec<Cents>, Vec<Cents>) {
    if !game.demanded_wrong() {
        return (Vec::new(), Vec::new());
    }
    let tariff = game.tariff.as_ref().unwrap_or(&table.tariff);
    let n = table.players.len();
    let mut asked = game.input.clone();
    asked.reclaimed = false;
    let mut right = asked.clone();
    right.price_override = None;
    (payouts(tariff, &asked, n), payouts(tariff, &right, n))
}
