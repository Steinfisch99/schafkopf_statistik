//! Vergleich der Spieler: Tippt man in der Karte eines Spielers auf eine Zahl, erscheint darunter die Rangliste
//! dieser Zahl für alle, die an den gewählten Abenden dabei waren. Wer gewählt hat, ist hervorgehoben.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use schafkopf_charts::format_euro;
use schafkopf_core::{Cents, OverallPlayer, PlayerStats, RoleStats};

use super::stats_screen::Kv;
use crate::stats_text::{doubling_label, percent, share};
use crate::summary::signed_euro;

/// Eine Zahl, die sich zwischen den Spielern vergleichen lässt.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Metric {
    Balance,
    AveragePerGame,
    BestGame,
    WorstGame,
    Played,
    SatOut,
    Dealt,
    WinRate,
    AsDeclarer,
    AsPartner,
    AsDefender,
    Knocks,
    Kontra,
    Re,
    /// Wie oft er selbst gedoppelt hat (Klopfen, Kontra und Re zusammen).
    Doubles,
    Touts,
    Sies,
    SchneiderWins,
    SchwarzWins,
    LaufOwn,
    LaufOther,
    LongestWinStreak,
    LongestLossStreak,
    Offences,
    Misdeals,
    PenaltyPaid,
    PenaltyReceived,
    /// Was er im Stock gezahlt, bekommen und unterm Strich gewonnen oder verloren hat.
    StockPaid,
    StockReceived,
    StockNet,
    WrongDemands,
    /// Spiele, in denen der Preis genau so oft verdoppelt war.
    Doubled(u32),
    Evenings,
    EveningsWon,
    AveragePerEvening,
    BestEvening,
    WorstEvening,
}

impl Metric {
    pub fn label(self) -> String {
        match self {
            Metric::Balance => "Bilanz".into(),
            Metric::AveragePerGame => "Ø pro Spiel".into(),
            Metric::BestGame => "Bestes Spiel".into(),
            Metric::WorstGame => "Schlechtestes Spiel".into(),
            Metric::Played => "Gespielt".into(),
            Metric::SatOut => "Ausgesetzt".into(),
            Metric::Dealt => "Gegeben".into(),
            Metric::WinRate => "Siegquote".into(),
            Metric::AsDeclarer => "Siege als Spieler".into(),
            Metric::AsPartner => "Siege als Mitspieler".into(),
            Metric::AsDefender => "Siege als Gegenspieler".into(),
            Metric::Knocks => "Geklopft, Siegquote".into(),
            Metric::Kontra => "Kontra, Siegquote".into(),
            Metric::Re => "Re, Siegquote".into(),
            Metric::Doubles => "Selbst gedoppelt".into(),
            Metric::Touts => "Tout gespielt".into(),
            Metric::Sies => "Sie gespielt".into(),
            Metric::SchneiderWins => "Siege mit Schneider".into(),
            Metric::SchwarzWins => "Siege mit Schwarz".into(),
            Metric::LaufOwn => "Lauf auf der eigenen Seite".into(),
            Metric::LaufOther => "Lauf auf der Gegenseite".into(),
            Metric::LongestWinStreak => "Längste Siegesserie".into(),
            Metric::LongestLossStreak => "Längste Verlustserie".into(),
            Metric::Offences => "Vergehen".into(),
            Metric::Misdeals => "Neu gemischt".into(),
            Metric::PenaltyPaid => "Strafen gezahlt".into(),
            Metric::PenaltyReceived => "Strafen bekommen".into(),
            Metric::StockPaid => "Stock eingezahlt".into(),
            Metric::StockReceived => "Stock bekommen".into(),
            Metric::StockNet => "Stock unterm Strich".into(),
            Metric::WrongDemands => "Preis falsch verlangt".into(),
            Metric::Doubled(n) => format!("Spiele: {}", doubling_label(n)),
            Metric::Evenings => "Abende".into(),
            Metric::EveningsWon => "Abende im Plus".into(),
            Metric::AveragePerEvening => "Ø pro Abend".into(),
            Metric::BestEvening => "Bester Abend".into(),
            Metric::WorstEvening => "Schlechtester Abend".into(),
        }
    }
}

/// Die vergleichbaren Zahlen eines Spielers: je Zahl der Wert zum Sortieren und der Text dazu.
#[derive(Clone, Debug, PartialEq)]
pub struct PeerRow {
    pub name: String,
    pub values: BTreeMap<Metric, (f64, String)>,
}

impl PeerRow {
    fn new(name: &str) -> Self {
        PeerRow {
            name: name.to_owned(),
            values: BTreeMap::new(),
        }
    }

    fn count(&mut self, metric: Metric, n: u32) {
        self.values.insert(metric, (f64::from(n), n.to_string()));
    }

    fn cents(&mut self, metric: Metric, c: Cents) {
        self.values.insert(metric, (c as f64, signed_euro(c)));
    }

    /// Wie oft er gedoppelt hat, mit dem Anteil an seinen Spielen; ohne Spiele gibt es nichts zu vergleichen.
    fn doubles(&mut self, count: u32, played: u32) {
        if played > 0 {
            self.values.insert(
                Metric::Doubles,
                (
                    f64::from(count),
                    format!("{count} ({})", percent(count, played)),
                ),
            );
        }
    }

    /// Eine seltene Ansage: sortiert nach der Zahl der Spiele, der Text nennt die Siege dazu.
    fn played_won(&mut self, metric: Metric, role: RoleStats) {
        if role.games > 0 {
            self.values.insert(
                metric,
                (
                    f64::from(role.games),
                    format!("{} (gewonnen: {})", role.games, role.wins),
                ),
            );
        }
    }

    fn money(&mut self, metric: Metric, c: Cents) {
        self.values.insert(metric, (c as f64, format_euro(c)));
    }

    /// Eine Quote, sortiert nach dem Anteil; ohne Spiele gibt es nichts zu vergleichen.
    fn rate(&mut self, metric: Metric, role: RoleStats) {
        if role.games > 0 {
            let fraction = f64::from(role.wins) / f64::from(role.games);
            self.values.insert(
                metric,
                (
                    fraction,
                    format!("{} ({} von {})", share(role), role.wins, role.games),
                ),
            );
        }
    }
}

/// Die Zahlen eines Spielers an einem Abend.
pub fn evening_peer(p: &PlayerStats) -> PeerRow {
    let mut r = PeerRow::new(&p.name);
    if p.games_played > 0 {
        r.cents(
            Metric::AveragePerGame,
            p.balance / i64::from(p.games_played),
        );
        r.rate(
            Metric::WinRate,
            RoleStats {
                games: p.games_played,
                wins: p.as_declarer.wins + p.as_partner.wins + p.as_defender.wins,
            },
        );
    }
    r.cents(Metric::Balance, p.balance);
    r.cents(Metric::BestGame, p.best_game);
    r.cents(Metric::WorstGame, p.worst_game);
    r.count(Metric::Played, p.games_played);
    r.count(Metric::SatOut, p.games_sat_out);
    r.count(Metric::Dealt, p.games_dealt + p.deals_without_game);
    r.rate(Metric::AsDeclarer, p.as_declarer);
    r.rate(Metric::AsPartner, p.as_partner);
    r.rate(Metric::AsDefender, p.as_defender);
    r.rate(Metric::Knocks, p.knocks);
    r.rate(Metric::Kontra, p.kontra_given);
    r.rate(Metric::Re, p.re_given);
    r.doubles(p.doubles_given(), p.games_played);
    r.played_won(Metric::Touts, p.touts);
    r.played_won(Metric::Sies, p.sies);
    r.count(Metric::SchneiderWins, p.schneider_wins);
    r.count(Metric::SchwarzWins, p.schwarz_wins);
    r.count(Metric::LaufOwn, p.lauf_own_side);
    r.count(Metric::LaufOther, p.lauf_other_side);
    r.count(Metric::LongestWinStreak, p.longest_win_streak);
    r.count(Metric::LongestLossStreak, p.longest_loss_streak);
    r.count(Metric::Offences, p.offences);
    r.count(Metric::Misdeals, p.misdeals);
    r.money(Metric::PenaltyPaid, p.penalty_paid);
    r.money(Metric::PenaltyReceived, p.penalty_received);
    if !p.stock.is_empty() {
        r.money(Metric::StockPaid, p.stock.paid());
        r.money(Metric::StockReceived, p.stock.received);
        r.cents(Metric::StockNet, p.stock.net());
    }
    r.count(Metric::WrongDemands, p.wrong_demands);
    for (&times, &n) in &p.by_doublings {
        r.count(Metric::Doubled(times), n);
    }
    r
}

/// Die Zahlen eines Spielers über alle gewählten Abende.
pub fn overall_peer(p: &OverallPlayer) -> PeerRow {
    let mut r = PeerRow::new(&p.name);
    if p.games_played > 0 {
        r.cents(
            Metric::AveragePerGame,
            p.balance / i64::from(p.games_played),
        );
        r.rate(Metric::WinRate, p.overall());
    }
    if p.evenings > 0 {
        r.cents(Metric::AveragePerEvening, p.balance / i64::from(p.evenings));
    }
    r.cents(Metric::Balance, p.balance);
    r.count(Metric::Evenings, p.evenings);
    r.count(Metric::EveningsWon, p.evenings_won());
    if let Some(best) = p.best_evening() {
        r.cents(Metric::BestEvening, best);
    }
    if let Some(worst) = p.worst_evening() {
        r.cents(Metric::WorstEvening, worst);
    }
    r.cents(Metric::BestGame, p.best_game);
    r.cents(Metric::WorstGame, p.worst_game);
    r.count(Metric::Played, p.games_played);
    r.count(Metric::SatOut, p.games_sat_out);
    r.count(Metric::Dealt, p.games_dealt + p.deals_without_game);
    r.rate(Metric::AsDeclarer, p.as_declarer);
    r.rate(Metric::AsPartner, p.as_partner);
    r.rate(Metric::AsDefender, p.as_defender);
    r.rate(Metric::Knocks, p.knocks);
    r.rate(Metric::Kontra, p.kontra_given);
    r.rate(Metric::Re, p.re_given);
    r.doubles(p.doubles_given(), p.games_played);
    r.played_won(Metric::Touts, p.touts);
    r.played_won(Metric::Sies, p.sies);
    r.count(Metric::SchneiderWins, p.schneider_wins);
    r.count(Metric::SchwarzWins, p.schwarz_wins);
    r.count(Metric::LaufOwn, p.lauf_own_side);
    r.count(Metric::LaufOther, p.lauf_other_side);
    r.count(Metric::LongestWinStreak, p.longest_win_streak);
    r.count(Metric::LongestLossStreak, p.longest_loss_streak);
    r.count(Metric::Offences, p.offences);
    r.count(Metric::Misdeals, p.misdeals);
    r.money(Metric::PenaltyPaid, p.penalty_paid);
    r.money(Metric::PenaltyReceived, p.penalty_received);
    if !p.stock.is_empty() {
        r.money(Metric::StockPaid, p.stock.paid());
        r.money(Metric::StockReceived, p.stock.received);
        r.cents(Metric::StockNet, p.stock.net());
    }
    r.count(Metric::WrongDemands, p.wrong_demands);
    for (&times, &n) in &p.by_doublings {
        r.count(Metric::Doubled(times), n);
    }
    r
}

/// Die Rangliste einer Zahl: Platz, Name und Text, der größte Wert zuerst. Gleiche Werte teilen sich den
/// Platz und stehen nach Namen geordnet.
pub fn ranking(peers: &[PeerRow], metric: Metric) -> Vec<(usize, String, String)> {
    let mut rows: Vec<(&str, f64, &str)> = peers
        .iter()
        .filter_map(|p| {
            p.values
                .get(&metric)
                .map(|(v, t)| (p.name.as_str(), *v, t.as_str()))
        })
        .collect();
    rows.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    let mut out = Vec::with_capacity(rows.len());
    let mut place = 0;
    let mut last = None;
    for (i, (name, value, text)) in rows.into_iter().enumerate() {
        if last != Some(value) {
            place = i + 1;
            last = Some(value);
        }
        out.push((place, name.to_owned(), text.to_owned()));
    }
    out
}

/// Macht den Inhalt antippbar: Ein Tipp öffnet darunter die Rangliste der Zahl, ein zweiter schließt sie.
/// Es ist immer nur eine Rangliste je Spielerkarte offen.
#[component]
pub fn Tap(
    metric: Metric,
    me: String,
    peers: Vec<PeerRow>,
    open: Signal<Option<Metric>>,
    children: Element,
) -> Element {
    let mut open = open;
    let is_open = open() == Some(metric);
    let rows = if is_open {
        ranking(&peers, metric)
    } else {
        Vec::new()
    };
    rsx! {
        div {
            class: if is_open { "tap on" } else { "tap" },
            role: "button",
            tabindex: "0",
            "aria-expanded": "{is_open}",
            onclick: move |_| open.set(if is_open { None } else { Some(metric) }),
            {children}
        }
        if is_open {
            div { class: "rank-box",
                p { class: "rank-title", "Rangliste: {metric.label()}" }
                if rows.len() < 2 {
                    p { class: "hint", "Zum Vergleichen fehlen weitere Spieler mit dieser Zahl." }
                }
                ol { class: "rank-list",
                    for (place, name, text) in rows {
                        li { key: "{name}", class: if name == me { "me" } else { "" },
                            span { class: "place", "{place}." }
                            span { class: "who", "{name}" }
                            b { "{text}" }
                        }
                    }
                }
                p { class: "legend-note", "Der größte Wert steht oben. Tippe die Zahl noch einmal, um die Rangliste zu schließen." }
            }
        }
    }
}

/// Eine Zeile "Name: Wert", antippbar für die Rangliste.
#[component]
pub fn TapKv(
    label: String,
    value: String,
    metric: Metric,
    me: String,
    peers: Vec<PeerRow>,
    open: Signal<Option<Metric>>,
) -> Element {
    rsx! {
        Tap { metric, me, peers, open,
            Kv { label, value }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, balance: Cents, played: u32) -> PeerRow {
        let mut r = PeerRow::new(name);
        r.cents(Metric::Balance, balance);
        r.count(Metric::Played, played);
        r
    }

    #[test]
    fn the_ranking_puts_the_largest_value_first_and_ties_share_the_place() {
        let peers = vec![
            row("Cem", 50, 4),
            row("Anna", 120, 4),
            row("Ben", 50, 3),
            row("Dora", -30, 4),
        ];
        let by_money: Vec<(usize, String)> = ranking(&peers, Metric::Balance)
            .into_iter()
            .map(|(p, n, _)| (p, n))
            .collect();
        assert_eq!(
            by_money,
            vec![
                (1, "Anna".into()),
                (2, "Ben".into()),
                (2, "Cem".into()),
                (4, "Dora".into())
            ]
        );
        let played = ranking(&peers, Metric::Played);
        assert_eq!(played[0].0, 1);
        assert_eq!(played.iter().filter(|r| r.0 == 1).count(), 3);
        assert_eq!(played[3], (4, "Ben".into(), "3".into()));
    }

    #[test]
    fn the_stock_net_is_ranked_for_players_who_used_the_stock() {
        use schafkopf_core::{GameInput, GameKind, NoGameRule, Suit, Table, Tariff};
        let names = ["Anna", "Ben", "Cem", "Dora"].map(String::from).to_vec();
        let mut t = Table::new("T", names, 0, Tariff::default()).unwrap();
        t.no_game_rule = NoGameRule::Stock;
        t.add_no_game(None).unwrap();
        let game = GameInput {
            suit: Some(Suit::Eichel),
            partner: Some(3),
            ..GameInput::new(1, GameKind::Sauspiel, 2)
        };
        t.add_game(game).unwrap();
        let stats = schafkopf_core::table_stats(&t);
        let peers: Vec<PeerRow> = stats.players.iter().map(evening_peer).collect();
        let net = ranking(&peers, Metric::StockNet);
        // Cem und Dora gewinnen den Stock (je 40) und haben 20 eingelegt: +20, Anna und Ben -20.
        assert_eq!(net[0], (1, "Cem".into(), "+0,20\u{a0}€".into()));
        assert_eq!(net[1].0, 1);
        assert_eq!(net[3].0, 3);
        let plain = Table::new("T", vec!["A".into(), "B".into()], 0, Tariff::default()).unwrap();
        let none = schafkopf_core::table_stats(&plain);
        assert!(ranking(
            &none.players.iter().map(evening_peer).collect::<Vec<_>>(),
            Metric::StockNet
        )
        .is_empty());
    }

    #[test]
    fn players_without_the_number_are_left_out() {
        let mut anna = row("Anna", 10, 2);
        anna.rate(Metric::Knocks, RoleStats { games: 2, wins: 1 });
        let ben = row("Ben", 5, 2);
        let ranked = ranking(&[anna, ben], Metric::Knocks);
        assert_eq!(ranked.len(), 1);
        assert!(ranked[0].2.contains("1 von 2"));
    }

    #[test]
    fn doubles_and_touts_can_be_compared() {
        let players = ["Anna", "Ben", "Cem", "Dora"].map(String::from).to_vec();
        let mut t =
            schafkopf_core::Table::new("T", players, 0, schafkopf_core::Tariff::default()).unwrap();
        t.add_game(schafkopf_core::GameInput {
            suit: Some(schafkopf_core::Suit::Eichel),
            tout: true,
            knockers: vec![2],
            ..schafkopf_core::GameInput::new(0, schafkopf_core::GameKind::Farbsolo, 1)
        })
        .unwrap();
        let stats = schafkopf_core::table_stats(&t);
        let rows: Vec<PeerRow> = stats.players.iter().map(evening_peer).collect();
        let doubles = ranking(&rows, Metric::Doubles);
        assert_eq!(doubles.len(), 4, "alle haben mitgespielt");
        assert_eq!(
            (doubles[0].1.as_str(), doubles[0].2.as_str()),
            ("Cem", "1 (100\u{a0}%)")
        );
        let touts = ranking(&rows, Metric::Touts);
        assert_eq!(touts.len(), 1, "nur Ben hat Tout gespielt");
        assert_eq!(touts[0].2, "1 (gewonnen: 1)");
    }
    #[test]
    fn evening_and_overall_rows_cover_the_same_core_numbers() {
        let players = ["Anna", "Ben", "Cem", "Dora"].map(String::from).to_vec();
        let mut t =
            schafkopf_core::Table::new("T", players, 0, schafkopf_core::Tariff::default()).unwrap();
        t.add_game(schafkopf_core::GameInput {
            suit: Some(schafkopf_core::Suit::Eichel),
            partner: Some(3),
            ..schafkopf_core::GameInput::new(0, schafkopf_core::GameKind::Sauspiel, 1)
        })
        .unwrap();
        let stats = schafkopf_core::table_stats(&t);
        let evening: Vec<PeerRow> = stats.players.iter().map(evening_peer).collect();
        let all = schafkopf_core::overall_stats(&[&t]);
        let overall: Vec<PeerRow> = all.players.iter().map(overall_peer).collect();
        for metric in [
            Metric::Balance,
            Metric::Played,
            Metric::WinRate,
            Metric::Doubled(0),
        ] {
            assert_eq!(
                ranking(&evening, metric).len(),
                ranking(&overall, metric).len(),
                "{metric:?}"
            );
        }
        assert_eq!(ranking(&evening, Metric::Balance)[0].2, "+0,20\u{a0}€");
    }
}
