//! Aufbereitung der Statistik für die Anzeige: Texte und Sortierungen ohne Oberfläche.

use schafkopf_core::{
    Cents, GameKind, KindStats, LaufendeStats, PlayerStats, RoleStats, Table, TableStats,
};

/// Prozent als ganze Zahl mit geschütztem Leerzeichen, "–" ohne Spiele.
pub fn percent(wins: u32, games: u32) -> String {
    if games == 0 {
        return "–".to_owned();
    }
    let pct = (u64::from(wins) * 100 + u64::from(games) / 2) / u64::from(games);
    format!("{pct}\u{a0}%")
}

/// "3 von 5 (60 %)", oder "–" ohne Spiele.
pub fn share(stats: RoleStats) -> String {
    if stats.games == 0 {
        return "–".to_owned();
    }
    format!(
        "{} von {} ({})",
        stats.wins,
        stats.games,
        percent(stats.wins, stats.games)
    )
}

fn plural(n: u32, one: &str, many: &str) -> String {
    format!("{n} {}", if n == 1 { one } else { many })
}

/// Die aktuelle Serie: "3 Siege in Folge", "1 Niederlage in Folge" oder "–".
pub fn streak(current: i32) -> String {
    match current {
        0 => "–".to_owned(),
        n if n > 0 => format!("{} in Folge", plural(n as u32, "Sieg", "Siege")),
        n => format!(
            "{} in Folge",
            plural(n.unsigned_abs(), "Niederlage", "Niederlagen")
        ),
    }
}

/// Wie oft ein Spiel verdoppelt war, als Text: 0 "Ohne Doppler", 1 "Nur verdoppelt (mal 2)", 2 "Vervierfacht (mal 4)", 3 "Verachtfacht (mal 8)", ...
pub fn doubling_label(times: u32) -> String {
    match times {
        0 => "Ohne Doppler".to_owned(),
        1 => "Nur verdoppelt (mal 2)".to_owned(),
        2 => "Vervierfacht (mal 4)".to_owned(),
        3 => "Verachtfacht (mal 8)".to_owned(),
        n => format!("Mal {} ({n} Doppler)", 1u64 << n),
    }
}

/// Bezeichnung des Platzes relativ zum Geber, siehe `PlayerStats::by_position`.
pub fn position_label(index: usize) -> &'static str {
    match index {
        0 => "Vorhand",
        1 => "Mittelhand",
        2 => "Hinterhand",
        _ => "Letzter",
    }
}

/// Spieler nach Kontostand, der Beste zuerst; bei Gleichstand nach Sitzplatz.
pub fn ranking(stats: &TableStats) -> Vec<&PlayerStats> {
    let mut players: Vec<&PlayerStats> = stats.players.iter().collect();
    players.sort_by_key(|p| (std::cmp::Reverse(p.balance), p.seat));
    players
}

/// Durchschnittlicher Gewinn oder Verlust pro gespieltem Spiel in Cent, gerundet.
pub fn average_per_game(player: &PlayerStats) -> Option<Cents> {
    let games = Cents::from(player.games_played);
    (games > 0).then(|| (player.balance as f64 / games as f64).round() as Cents)
}

/// Spielarten, die am Tisch vorkamen, in der üblichen Reihenfolge.
pub fn kind_rows(
    by_kind: &std::collections::BTreeMap<GameKind, KindStats>,
) -> Vec<(GameKind, KindStats)> {
    GameKind::ALL
        .into_iter()
        .filter_map(|kind| {
            by_kind
                .get(&kind)
                .filter(|k| k.games > 0)
                .map(|k| (kind, *k))
        })
        .collect()
}

/// Alle Spiele eines Spielers und wie viele davon gewonnen, unabhängig von der Rolle.
pub fn overall(player: &PlayerStats) -> RoleStats {
    let roles = [player.as_declarer, player.as_partner, player.as_defender];
    RoleStats {
        games: roles.iter().map(|r| r.games).sum(),
        wins: roles.iter().map(|r| r.wins).sum(),
    }
}

/// Wie oft hat die Spielerpartei gewonnen? Ohne Ramsch, dort bedeutet "gewonnen" etwas anderes.
pub fn declarer_rate(stats: &TableStats) -> RoleStats {
    stats
        .by_kind
        .iter()
        .filter(|(kind, _)| **kind != GameKind::Ramsch)
        .fold(RoleStats::default(), |acc, (_, k)| RoleStats {
            games: acc.games + k.games,
            wins: acc.wins + k.wins,
        })
}

/// Zwei Spieler, die zusammen gespielt haben (Sauspiel oder Hochzeit).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PairRow {
    pub first: String,
    pub second: String,
    pub stats: RoleStats,
}

/// Alle Paare mit mindestens einem gemeinsamen Spiel: die häufigsten zuerst, dann die erfolgreichsten.
pub fn pair_rows(stats: &TableStats) -> Vec<PairRow> {
    let mut rows = Vec::new();
    for (a, row) in stats.partner_matrix.iter().enumerate() {
        for (b, cell) in row.iter().enumerate().skip(a + 1) {
            if cell.games > 0 {
                rows.push(PairRow {
                    first: stats.players[a].name.clone(),
                    second: stats.players[b].name.clone(),
                    stats: *cell,
                });
            }
        }
    }
    rows.sort_by(|x, y| {
        (y.stats.games, y.stats.wins, &x.first, &x.second).cmp(&(
            x.stats.games,
            x.stats.wins,
            &y.first,
            &y.second,
        ))
    });
    rows
}

/// Eine Kommazahl mit einer Nachkommastelle und deutschem Komma: `3.5` wird zu `3,5`.
pub fn decimal(value: f64) -> String {
    format!("{value:.1}").replace('.', ",")
}

/// Durchschnittliche Zahl Laufende über alle Läufe und Gegenläufe einer Spielart, "–" ohne.
pub fn average_laufende(stats: &LaufendeStats) -> String {
    let count = stats.lauf + stats.gegenlauf;
    if count == 0 {
        return "–".to_owned();
    }
    decimal(f64::from(stats.lauf_sum + stats.gegenlauf_sum) / f64::from(count))
}

/// Breite eines Balkens in Prozent der Spur, zwischen 0 und 100 (auf eine Stelle gerundet).
pub fn bar_percent(value: u32, max: u32) -> f64 {
    if max == 0 {
        return 0.0;
    }
    let pct = f64::from(value) / f64::from(max) * 100.0;
    (pct.clamp(0.0, 100.0) * 10.0).round() / 10.0
}

/// Lage und Breite eines Balkens, der von einer Mittellinie aus nach links (Verlust) oder rechts
/// (Gewinn) wächst. Beide Werte in Prozent der Spur: `(links, Breite)`. Der größte Betrag füllt
/// eine Hälfte. Ohne Maßstab (`max_abs` 0) gibt es keinen Balken.
pub fn diverging(value: Cents, max_abs: Cents) -> (f64, f64) {
    if max_abs <= 0 {
        return (50.0, 0.0);
    }
    let half = (value.unsigned_abs() as f64 / max_abs as f64).min(1.0) * 50.0;
    let half = (half * 10.0).round() / 10.0;
    if value >= 0 {
        (50.0, half)
    } else {
        (50.0 - half, half)
    }
}

/// Der größte Betrag unter den Werten, als Maßstab für [`diverging`].
pub fn max_abs(values: impl IntoIterator<Item = Cents>) -> Cents {
    values.into_iter().map(Cents::abs).max().unwrap_or(0)
}

/// Pro Sitzplatz die Ergebnisse aller Spiele in Reihenfolge: `Some(true)` Sieg, `Some(false)`
/// Niederlage, `None` ausgesetzt. Ein Spieler gewinnt, wenn seine Partei gewinnt.
pub fn results_per_player(table: &Table) -> Vec<Vec<Option<bool>>> {
    let mut rows = vec![Vec::new(); table.players.len()];
    for game in table.games() {
        let input = &game.input;
        for (seat, row) in rows.iter_mut().enumerate() {
            // Wer noch nicht (oder nicht mehr) am Tisch saß, hat in dem Spiel keinen Eintrag.
            if !table.was_at_table(game, seat) {
                continue;
            }
            row.push(
                input
                    .participants
                    .contains(&seat)
                    .then(|| input.is_on_declarer_team(seat) == input.won),
            );
        }
    }
    rows
}

/// Wie oft ein Spieler gedoppelt hat: Klopfen, Kontra und Re einzeln, dazu die Zahl seiner Spiele.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoublerRow {
    pub name: String,
    pub knocks: u32,
    pub kontra: u32,
    pub re: u32,
    pub played: u32,
}

impl DoublerRow {
    pub fn total(&self) -> u32 {
        self.knocks + self.kontra + self.re
    }
}

/// Wer am häufigsten gedoppelt hat: Platz und Zeile, die meisten zuerst. Gleiche Summen teilen sich den
/// Platz und stehen nach Namen geordnet. Wer nicht mitgespielt hat, steht nicht in der Liste.
pub fn doubler_ranking(rows: Vec<DoublerRow>) -> Vec<(usize, DoublerRow)> {
    let mut rows: Vec<DoublerRow> = rows.into_iter().filter(|r| r.played > 0).collect();
    rows.sort_by(|a, b| b.total().cmp(&a.total()).then_with(|| a.name.cmp(&b.name)));
    let mut place = 0;
    let mut last = None;
    rows.into_iter()
        .enumerate()
        .map(|(i, r)| {
            if last != Some(r.total()) {
                place = i + 1;
                last = Some(r.total());
            }
            (place, r)
        })
        .collect()
}

/// Tout (mit Sie) eines Spielers: wie oft gespielt und wie oft gewonnen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToutRow {
    pub name: String,
    pub touts: RoleStats,
    pub sies: RoleStats,
}

/// Wer wie oft Tout gespielt hat, die meisten zuerst; gleiche Zahlen teilen sich den Platz. Wer nie Tout
/// gespielt hat, steht nicht in der Liste.
pub fn tout_ranking(rows: Vec<ToutRow>) -> Vec<(usize, ToutRow)> {
    let mut rows: Vec<ToutRow> = rows.into_iter().filter(|r| r.touts.games > 0).collect();
    rows.sort_by(|a, b| {
        b.touts
            .games
            .cmp(&a.touts.games)
            .then_with(|| b.touts.wins.cmp(&a.touts.wins))
            .then_with(|| a.name.cmp(&b.name))
    });
    let mut place = 0;
    let mut last = None;
    rows.into_iter()
        .enumerate()
        .map(|(i, r)| {
            let key = (r.touts.games, r.touts.wins);
            if last != Some(key) {
                place = i + 1;
                last = Some(key);
            }
            (place, r)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use schafkopf_core::{table_stats, GameInput, GameKind, Suit, Table, Tariff};

    fn table() -> Table {
        let players = ["Anna", "Ben", "Cem", "Dora"].map(String::from).to_vec();
        Table::new("T", players, 0, Tariff::default()).unwrap()
    }

    fn sau(dealer: usize, declarer: usize, partner: usize, won: bool) -> GameInput {
        GameInput {
            suit: Some(Suit::Eichel),
            partner: Some(partner),
            won,
            ..GameInput::new(dealer, GameKind::Sauspiel, declarer)
        }
    }

    #[test]
    fn percent_rounds_and_handles_no_games() {
        assert_eq!(percent(0, 0), "–");
        assert_eq!(percent(1, 3), "33\u{a0}%");
        assert_eq!(percent(2, 3), "67\u{a0}%");
        assert_eq!(percent(1, 8), "13\u{a0}%");
        assert_eq!(percent(5, 5), "100\u{a0}%");
        assert_eq!(
            share(RoleStats { games: 5, wins: 3 }),
            "3 von 5 (60\u{a0}%)"
        );
        assert_eq!(share(RoleStats::default()), "–");
    }

    #[test]
    fn names_the_doubling_levels() {
        assert_eq!(doubling_label(0), "Ohne Doppler");
        assert_eq!(doubling_label(1), "Nur verdoppelt (mal 2)");
        assert_eq!(doubling_label(2), "Vervierfacht (mal 4)");
        assert_eq!(doubling_label(3), "Verachtfacht (mal 8)");
        assert_eq!(doubling_label(5), "Mal 32 (5 Doppler)");
    }

    #[test]
    fn streak_text_uses_the_right_plural() {
        assert_eq!(streak(0), "–");
        assert_eq!(streak(1), "1 Sieg in Folge");
        assert_eq!(streak(4), "4 Siege in Folge");
        assert_eq!(streak(-1), "1 Niederlage in Folge");
        assert_eq!(streak(-3), "3 Niederlagen in Folge");
    }

    #[test]
    fn ranking_sorts_by_balance_then_seat() {
        let mut t = table();
        t.add_game(sau(0, 1, 3, true)).unwrap();
        t.add_game(sau(1, 1, 3, true)).unwrap();
        let stats = table_stats(&t);
        let names: Vec<&str> = ranking(&stats).iter().map(|p| p.name.as_str()).collect();
        // Ben und Dora +40, Anna und Cem -40, jeweils nach Sitzplatz geordnet.
        assert_eq!(names, ["Ben", "Dora", "Anna", "Cem"]);
    }

    #[test]
    fn averages_per_game() {
        let mut t = table();
        assert_eq!(average_per_game(&table_stats(&t).players[0]), None);
        t.add_game(sau(0, 1, 3, true)).unwrap();
        t.add_game(sau(1, 1, 3, false)).unwrap();
        t.add_game(sau(2, 1, 3, true)).unwrap();
        let stats = table_stats(&t);
        // Ben: +20 -20 +20 = +20 in drei Spielen, also gerundet 7.
        assert_eq!(average_per_game(&stats.players[1]), Some(7));
        assert_eq!(average_per_game(&stats.players[0]), Some(-7));
    }

    #[test]
    fn pairs_are_sorted_by_games_then_wins() {
        let mut t = table();
        t.add_game(sau(0, 1, 3, true)).unwrap();
        t.add_game(sau(1, 3, 1, false)).unwrap();
        t.add_game(sau(2, 0, 2, true)).unwrap();
        let rows = pair_rows(&table_stats(&t));
        assert_eq!(rows.len(), 2);
        assert_eq!(
            (rows[0].first.as_str(), rows[0].second.as_str()),
            ("Ben", "Dora")
        );
        assert_eq!(rows[0].stats, RoleStats { games: 2, wins: 1 });
        assert_eq!(
            (rows[1].first.as_str(), rows[1].second.as_str()),
            ("Anna", "Cem")
        );
        assert!(pair_rows(&table_stats(&table())).is_empty());
    }

    #[test]
    fn kinds_and_declarer_rate_skip_ramsch_in_the_rate() {
        let mut t = table();
        t.add_game(sau(0, 1, 3, true)).unwrap();
        t.add_game(GameInput {
            won: false,
            ..GameInput::new(1, GameKind::Wenz, 2)
        })
        .unwrap();
        t.add_game(GameInput {
            won: false,
            ..GameInput::new(2, GameKind::Ramsch, 3)
        })
        .unwrap();
        let stats = table_stats(&t);
        let kinds: Vec<GameKind> = kind_rows(&stats.by_kind).iter().map(|(k, _)| *k).collect();
        assert_eq!(
            kinds,
            [GameKind::Sauspiel, GameKind::Wenz, GameKind::Ramsch]
        );
        assert_eq!(declarer_rate(&stats), RoleStats { games: 2, wins: 1 });
    }

    #[test]
    fn overall_adds_up_all_roles() {
        let mut t = table();
        t.add_game(sau(0, 1, 3, true)).unwrap();
        t.add_game(sau(1, 2, 0, false)).unwrap();
        let stats = table_stats(&t);
        // Ben: als Spieler gewonnen, im zweiten Spiel als Gegenspieler gewonnen.
        assert_eq!(overall(&stats.players[1]), RoleStats { games: 2, wins: 2 });
        assert_eq!(overall(&stats.players[2]), RoleStats { games: 2, wins: 0 });
        assert_eq!(
            overall(&stats.players[0]).games,
            stats.players[0].games_played
        );
    }

    #[test]
    fn bar_percent_scales_and_clamps() {
        assert_eq!(bar_percent(0, 0), 0.0);
        assert_eq!(bar_percent(3, 4), 75.0);
        assert_eq!(bar_percent(1, 3), 33.3);
        assert_eq!(bar_percent(9, 4), 100.0);
        assert_eq!(bar_percent(0, 5), 0.0);
    }

    #[test]
    fn diverging_bars_grow_from_the_middle() {
        assert_eq!(
            diverging(100, 100),
            (50.0, 50.0),
            "größter Gewinn: rechte Hälfte voll"
        );
        assert_eq!(
            diverging(-100, 100),
            (0.0, 50.0),
            "größter Verlust: linke Hälfte voll"
        );
        assert_eq!(diverging(50, 100), (50.0, 25.0));
        assert_eq!(diverging(-25, 100), (37.5, 12.5));
        assert_eq!(diverging(0, 100), (50.0, 0.0));
        assert_eq!(diverging(70, 0), (50.0, 0.0), "ohne Maßstab kein Balken");
        assert_eq!(
            diverging(500, 100),
            (50.0, 50.0),
            "nie über die Hälfte hinaus"
        );
        assert_eq!(max_abs([30, -120, 80]), 120);
        assert_eq!(max_abs([]), 0);
    }

    #[test]
    fn results_follow_the_players_party_and_mark_sat_out_games() {
        let players: Vec<String> = (0..5).map(|i| format!("S{i}")).collect();
        let mut t = Table::new("T", players, 0, Tariff::default()).unwrap();
        // Geber 0: es spielen 1, 2, 4, 0; Platz 3 setzt aus.
        t.add_game(sau(0, 1, 4, true)).unwrap();
        t.add_game(GameInput {
            won: false,
            ..GameInput::new(1, GameKind::Wenz, 2)
        })
        .unwrap();
        let rows = results_per_player(&t);
        assert_eq!(rows.len(), 5);
        // Spiel 1: Spieler 1 und Mitspieler 4 gewinnen, 2 und 0 verlieren, 3 setzt aus.
        let first: Vec<Option<bool>> = rows.iter().map(|r| r[0]).collect();
        assert_eq!(
            first,
            [Some(false), Some(true), Some(false), None, Some(true)]
        );
        // Spiel 2 (Geber 1, es spielen 2, 3, 0, 1): Spieler 2 verliert allein, die anderen gewinnen.
        let second: Vec<Option<bool>> = rows.iter().map(|r| r[1]).collect();
        assert_eq!(
            second,
            [Some(true), Some(true), Some(false), Some(true), None]
        );
        assert!(results_per_player(&table()).iter().all(Vec::is_empty));
    }

    #[test]
    fn decimals_and_average_laufende() {
        assert_eq!(decimal(3.0), "3,0");
        assert_eq!(decimal(3.46), "3,5");
        assert_eq!(decimal(0.04), "0,0");
        let stats = LaufendeStats {
            games: 5,
            lauf: 2,
            gegenlauf: 1,
            lauf_sum: 7,
            gegenlauf_sum: 3,
        };
        assert_eq!(average_laufende(&stats), "3,3", "(7 + 3) von 3 Spielen");
        assert_eq!(
            average_laufende(&LaufendeStats {
                games: 2,
                ..Default::default()
            }),
            "–"
        );
    }

    #[test]
    fn position_labels() {
        let labels: Vec<&str> = (0..4).map(position_label).collect();
        assert_eq!(labels, ["Vorhand", "Mittelhand", "Hinterhand", "Letzter"]);
    }

    fn doubler(name: &str, knocks: u32, kontra: u32, re: u32, played: u32) -> DoublerRow {
        DoublerRow {
            name: name.to_owned(),
            knocks,
            kontra,
            re,
            played,
        }
    }

    #[test]
    fn the_doubler_ranking_sums_the_calls_and_ties_share_the_place() {
        let ranked = doubler_ranking(vec![
            doubler("Cem", 1, 1, 0, 5),
            doubler("Anna", 3, 0, 1, 5),
            doubler("Ben", 2, 0, 0, 5),
            doubler("Dora", 0, 0, 0, 0),
            doubler("Emil", 0, 2, 0, 4),
        ]);
        let names: Vec<(usize, &str)> = ranked.iter().map(|(p, r)| (*p, r.name.as_str())).collect();
        assert_eq!(
            names,
            vec![(1, "Anna"), (2, "Ben"), (2, "Cem"), (2, "Emil")],
            "Dora saß nicht mit am Tisch"
        );
        assert_eq!(ranked[0].1.total(), 4);
    }

    #[test]
    fn the_tout_ranking_leaves_out_players_without_a_tout() {
        let row = |name: &str, games, wins| ToutRow {
            name: name.to_owned(),
            touts: RoleStats { games, wins },
            sies: RoleStats::default(),
        };
        let ranked = tout_ranking(vec![
            row("Anna", 0, 0),
            row("Ben", 2, 1),
            row("Cem", 2, 2),
            row("Dora", 1, 1),
        ]);
        let names: Vec<(usize, &str)> = ranked.iter().map(|(p, r)| (*p, r.name.as_str())).collect();
        assert_eq!(names, vec![(1, "Cem"), (2, "Ben"), (3, "Dora")]);
    }
}
