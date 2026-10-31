//! Aufbereitung der Statistik für die Anzeige: Texte und Sortierungen ohne Oberfläche.

use schafkopf_core::{
    format_clock, format_minutes_after_noon, time_spread, Cents, EveningFact, GameKind, KindStats,
    LaufendeStats, PlayerStats, Presence, RoleStats, Table, TableStats,
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

/// Ein Kästchen der Ergebnisreihe: Ergebnis, Beschreibung zum Antippen und ob es zur längsten Sieg- oder
/// Verlustserie gehört.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StripCell {
    /// `Some(true)` Sieg, `Some(false)` Niederlage, `None` ausgesetzt.
    pub result: Option<bool>,
    pub info: String,
    pub longest_win: bool,
    pub longest_loss: bool,
}

/// Erstes und letztes Kästchen einer Serie.
pub type Run = (usize, usize);

/// Die längste Siegserie und die längste Verlustserie als Bereiche `(erstes, letztes)` von Positionen in
/// der Reihe. Ausgesetzte Spiele unterbrechen eine Serie nicht (wie bei `PlayerStats`), bei Gleichstand
/// zählt die erste.
pub fn longest_runs(results: &[Option<bool>]) -> (Option<Run>, Option<Run>) {
    let mut best: [Option<Run>; 2] = [None, None];
    let mut best_len = [0usize; 2];
    let mut run: Option<(bool, usize, usize, usize)> = None;
    for (i, r) in results.iter().enumerate() {
        let Some(won) = *r else { continue };
        run = match run {
            Some((w, first, _, len)) if w == won => Some((w, first, i, len + 1)),
            _ => Some((won, i, i, 1)),
        };
        if let Some((w, first, last, len)) = run {
            let slot = usize::from(!w);
            if len > best_len[slot] {
                best_len[slot] = len;
                best[slot] = Some((first, last));
            }
        }
    }
    (best[0], best[1])
}

/// Pro Sitzplatz die Kästchen aller Spiele in Reihenfolge, mit Beschreibung (Spiel, Uhrzeit, Ausgang,
/// Betrag) und Markierung der längsten Serien.
pub fn strip_cells(table: &Table) -> Vec<Vec<StripCell>> {
    let results = results_per_player(table);
    let mut rows: Vec<Vec<StripCell>> = Vec::new();
    for (seat, row) in results.iter().enumerate() {
        let mut games = table
            .games()
            .iter()
            .enumerate()
            .filter(|(_, g)| table.was_at_table(g, seat));
        let (win, loss) = longest_runs(row);
        let inside = |range: Option<(usize, usize)>, i: usize| {
            range.is_some_and(|(a, b)| (a..=b).contains(&i))
        };
        let mut cells = Vec::with_capacity(row.len());
        for (i, result) in row.iter().enumerate() {
            let info = games
                .next()
                .map(|(index, game)| {
                    let mut text = format!("Spiel {}", index + 1);
                    if let Some(at) = game.input.played_at {
                        text.push_str(&format!(
                            " · {} Uhr",
                            format_clock(at, table.utc_offset_min)
                        ));
                    }
                    text.push_str(&format!(
                        ": {}, {}",
                        crate::summary::title(game),
                        crate::summary::detail(table, game)
                    ));
                    match result {
                        Some(_) => text.push_str(&format!(
                            " · {}",
                            crate::summary::signed_euro(
                                game.payouts.get(seat).copied().unwrap_or(0)
                            )
                        )),
                        None => text.push_str(" · ausgesetzt"),
                    }
                    text
                })
                .unwrap_or_default();
            cells.push(StripCell {
                result: *result,
                info,
                longest_win: inside(win, i) && *result == Some(true),
                longest_loss: inside(loss, i) && *result == Some(false),
            });
        }
        rows.push(cells);
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

/// Wann ein Spieler kam: "von Anfang an (19:30)" oder "20:45, nach Spiel 4".
pub fn arrival_text(p: &Presence, offset_min: i32) -> String {
    let time = p.arrived.map(|t| format_clock(t, offset_min));
    if p.from_start {
        return match time {
            Some(t) => format!("von Anfang an ({t})"),
            None => "von Anfang an".to_owned(),
        };
    }
    let when = match p.arrived_after_games {
        0 => "vor dem ersten Spiel".to_owned(),
        n => format!("nach Spiel {n}"),
    };
    match time {
        Some(t) => format!("{t}, {when}"),
        None => when,
    }
}

/// Wann ein Spieler ging: "21:50, nach Spiel 8" oder "blieb bis zum Ende"; war er zwischendurch weg, steht
/// das dahinter.
pub fn departure_text(p: &Presence, offset_min: i32) -> String {
    let mut text = if p.left_early {
        let time = p.left.map(|t| format_clock(t, offset_min));
        let when = match p.left_after_games {
            Some(0) => Some("vor dem ersten Spiel".to_owned()),
            Some(n) => Some(format!("nach Spiel {n}")),
            None => None,
        };
        match (time, when) {
            (Some(t), Some(w)) => format!("{t}, {w}"),
            (Some(t), None) => t,
            (None, Some(w)) => w,
            (None, None) => "ging".to_owned(),
        }
    } else {
        "blieb bis zum Ende".to_owned()
    };
    if p.comebacks > 0 {
        text.push_str(&format!(" (zwischendurch {}× weg)", p.comebacks));
    }
    text
}

/// Mittel und Spanne mehrerer Uhrzeiten (Minuten seit 12:00 Uhr): "Ø 19:40 (19:00 bis 20:30)".
pub fn spread_text(times: &[i32]) -> String {
    match time_spread(times) {
        None => "–".to_owned(),
        Some((avg, min, max)) if min == max => {
            format!("{} (immer)", format_minutes_after_noon(avg))
        }
        Some((avg, min, max)) => format!(
            "Ø {} ({} bis {})",
            format_minutes_after_noon(avg),
            format_minutes_after_noon(min),
            format_minutes_after_noon(max)
        ),
    }
}

/// "17.10.2026" in der Ortszeit des Abends.
pub fn date_text(unix_secs: i64, offset_min: i32) -> String {
    let (y, mo, d, _, _) = schafkopf_core::civil(unix_secs, offset_min);
    format!("{d:02}.{mo:02}.{y:04}")
}

/// Der Abend mit dem kleinsten oder größten Wert: der Wert und das Datum ("ohne Datum", wenn unbekannt).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Extreme {
    pub value: i64,
    pub date: String,
}

/// Mittel, Kleinstes und Größtes einer Zahl über die Abende.
#[derive(Debug, Clone, PartialEq)]
pub struct Spread {
    pub average: f64,
    pub min: Extreme,
    pub max: Extreme,
}

/// Überblick über mehrere Abende: wie viele Spiele und wie lange.
#[derive(Debug, Clone, PartialEq)]
pub struct EveningsSummary {
    pub games: Spread,
    /// Nur über Abende, deren Dauer bekannt ist.
    pub length: Option<Spread>,
    /// Spiele pro Stunde, alle Abende mit bekannter Dauer zusammen.
    pub games_per_hour: Option<f64>,
}

fn spread(values: &[(i64, String)]) -> Option<Spread> {
    // Bei Gleichstand gilt der erste Abend (kleinster: früher, größter: früher).
    let min = values.iter().min_by_key(|(v, _)| *v)?;
    let max = values.iter().rev().max_by_key(|(v, _)| *v)?;
    let sum: i64 = values.iter().map(|(v, _)| v).sum();
    Some(Spread {
        average: sum as f64 / values.len() as f64,
        min: Extreme {
            value: min.0,
            date: min.1.clone(),
        },
        max: Extreme {
            value: max.0,
            date: max.1.clone(),
        },
    })
}

pub fn fact_date(f: &EveningFact) -> String {
    f.started.map_or_else(
        || "ohne Datum".to_owned(),
        |t| date_text(t, f.utc_offset_min),
    )
}

/// Wertet Spiele und Dauer der Abende aus. `None` ohne Abende.
pub fn evenings_summary(facts: &[EveningFact]) -> Option<EveningsSummary> {
    let games: Vec<(i64, String)> = facts
        .iter()
        .map(|f| (i64::from(f.games), fact_date(f)))
        .collect();
    let lengths: Vec<(i64, String)> = facts
        .iter()
        .filter_map(|f| f.length.map(|l| (l, fact_date(f))))
        .collect();
    let timed_games: i64 = facts
        .iter()
        .filter(|f| f.length.is_some())
        .map(|f| i64::from(f.games))
        .sum();
    let timed_secs: i64 = lengths.iter().map(|(l, _)| l).sum();
    Some(EveningsSummary {
        games: spread(&games)?,
        length: spread(&lengths),
        games_per_hour: (timed_secs > 0).then(|| timed_games as f64 * 3_600.0 / timed_secs as f64),
    })
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
    fn longest_runs_skip_sat_out_games_and_take_the_first_on_a_tie() {
        let r = [
            Some(true),
            Some(true),
            None,
            Some(true),
            Some(false),
            Some(false),
            Some(true),
            Some(false),
            Some(false),
        ];
        assert_eq!(longest_runs(&r), (Some((0, 3)), Some((4, 5))));
        assert_eq!(longest_runs(&[None, None]), (None, None));
        assert_eq!(longest_runs(&[Some(false)]), (None, Some((0, 0))));
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

    fn presence(from_start: bool, left_early: bool) -> Presence {
        Presence {
            seat: 0,
            name: "Anna".into(),
            from_start,
            arrived: Some(1_700_000_000),
            arrived_after_games: if from_start { 0 } else { 4 },
            left: left_early.then_some(1_700_003_600),
            left_early,
            left_after_games: left_early.then_some(8),
            comebacks: 0,
        }
    }

    #[test]
    fn arrival_and_departure_read_as_sentences() {
        assert_eq!(
            arrival_text(&presence(true, false), 0),
            "von Anfang an (22:13)"
        );
        assert_eq!(
            arrival_text(&presence(false, false), 0),
            "22:13, nach Spiel 4"
        );
        assert_eq!(
            departure_text(&presence(true, false), 0),
            "blieb bis zum Ende"
        );
        let mut p = presence(true, true);
        assert_eq!(departure_text(&p, 0), "23:13, nach Spiel 8");
        p.comebacks = 2;
        assert!(departure_text(&p, 0).ends_with("(zwischendurch 2× weg)"));
    }

    #[test]
    fn spreads_of_times_show_average_and_range() {
        assert_eq!(spread_text(&[]), "–");
        assert_eq!(spread_text(&[600]), "22:00 (immer)");
        assert_eq!(spread_text(&[540, 660]), "Ø 22:00 (21:00 bis 23:00)");
    }

    fn fact(day: i64, games: u32, length: Option<i64>) -> EveningFact {
        EveningFact {
            started: Some(1_700_000_000 + day * 86_400),
            utc_offset_min: 0,
            games,
            length,
        }
    }

    #[test]
    fn the_evenings_summary_finds_average_min_and_max_with_dates() {
        let facts = [
            fact(0, 20, Some(3 * 3_600)),
            fact(7, 30, Some(5 * 3_600)),
            fact(14, 10, None),
        ];
        let s = evenings_summary(&facts).unwrap();
        assert_eq!(s.games.average, 20.0);
        assert_eq!(
            (s.games.min.value, s.games.min.date.as_str()),
            (10, "28.11.2023")
        );
        assert_eq!(
            (s.games.max.value, s.games.max.date.as_str()),
            (30, "21.11.2023")
        );
        let length = s.length.unwrap();
        assert_eq!(
            length.average,
            4.0 * 3_600.0,
            "der Abend ohne Dauer zählt nicht"
        );
        assert_eq!(length.min.date, "14.11.2023");
        assert_eq!(s.games_per_hour, Some(50.0 / 8.0));
        assert!(evenings_summary(&[]).is_none());
        assert_eq!(decimal(12.46), "12,5");
        assert_eq!(
            fact_date(&EveningFact {
                started: None,
                ..fact(0, 1, None)
            }),
            "ohne Datum"
        );
    }
}
