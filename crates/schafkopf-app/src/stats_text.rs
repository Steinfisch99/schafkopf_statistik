//! Aufbereitung der Statistik für die Anzeige: Texte und Sortierungen ohne Oberfläche.

use schafkopf_core::{Cents, GameKind, KindStats, PlayerStats, RoleStats, TableStats};

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
    fn position_labels() {
        let labels: Vec<&str> = (0..4).map(position_label).collect();
        assert_eq!(labels, ["Vorhand", "Mittelhand", "Hinterhand", "Letzter"]);
    }
}
