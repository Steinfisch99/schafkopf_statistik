//! Statistik: Rangliste, Überblick über den Abend, Spielarten, Paare und jeder Spieler im Detail.
//! Zu jeder Zahlenreihe, bei der es sich lohnt, gibt es eine Grafik. Die Zahlen bleiben daneben stehen.

use dioxus::document::eval;
use dioxus::prelude::*;
use schafkopf_charts::{format_euro, player_color, series_color};
use schafkopf_core::{
    table_meta, table_stats, Cents, GameKind, LaufendeStats, OffenceKind, PlayerStats, PlayerStock,
    RoleStats, StockStats, Suit, Table,
};

use super::compare::{evening_peer, Metric, PeerRow, Tap, TapKv};
use super::meta_sections::{EveningPresence, TariffSection};
use super::records::{evening_records, lauf_groups, RecordsSection};
use super::stat_widgets::{
    DivergingRow, DonutWithLegend, DoublerSection, ResultStrip, SegmentBar, ShareBar, StackedBar,
    ToutSection, WinDonut, GEGENLAUF_COLOR, LAUF_COLOR, NO_LAUF_COLOR,
};
use super::suit_icon::SuitIcon;
use crate::stats_text::{
    average_laufende, average_per_game, declarer_rate, doubling_label, kind_rows, max_abs, overall,
    pair_rows, percent, position_label, ranking, results_per_player, share, streak, DoublerRow,
    ToutRow,
};
use crate::summary::{detail, signed_euro, title};
use crate::Screen;

/// "3 (21 %)": eine Anzahl und ihr Anteil an allen Spielen.
pub(super) fn count_share(count: u32, total: u32) -> String {
    format!("{count} ({})", percent(count, total))
}

pub(super) fn amount_class(cents: Cents) -> &'static str {
    if cents < 0 {
        "amount neg"
    } else {
        "amount"
    }
}

pub(super) fn lost(role: RoleStats) -> u32 {
    role.games.saturating_sub(role.wins)
}

#[component]
pub(super) fn Kv(label: String, value: String) -> Element {
    rsx! {
        div { class: "kv",
            span { "{label}" }
            b { "{value}" }
        }
    }
}

#[component]
fn PlayerDetail(
    player: PlayerStats,
    results: Vec<Option<bool>>,
    open: bool,
    /// Die Zahlen aller Spieler des Abends für die Ranglisten beim Antippen.
    peers: Vec<PeerRow>,
) -> Element {
    let p = player;
    let compare = use_signal(|| None::<Metric>);
    let me = p.name.clone();
    let total = overall(&p);
    let kinds = kind_rows(&p.by_kind);
    let kind_scale = max_abs(kinds.iter().map(|(_, k)| k.balance));
    let positions: Vec<(usize, RoleStats)> = p
        .by_position
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, r)| r.games > 0)
        .collect();
    let position_scale = positions.iter().map(|(_, r)| r.games).max().unwrap_or(0);
    let roles = [
        ("Spieler", p.as_declarer, Metric::AsDeclarer),
        ("Mitspieler", p.as_partner, Metric::AsPartner),
        ("Gegenspieler", p.as_defender, Metric::AsDefender),
    ];
    let role_scale = roles.iter().map(|(_, r, _)| r.games).max().unwrap_or(0);
    let calls = [
        ("Geklopft", p.knocks, Metric::Knocks),
        ("Kontra", p.kontra_given, Metric::Kontra),
        ("Re", p.re_given, Metric::Re),
    ];
    let call_scale = calls.iter().map(|(_, r, _)| r.games).max().unwrap_or(0);
    let doubling_scale = p.by_doublings.values().copied().max().unwrap_or(0);
    let average = average_per_game(&p).map_or("–".to_string(), signed_euro);
    let win_label = format!("Siege und Niederlagen von {}", p.name);

    rsx! {
        details { class: "player-detail", id: "player-{p.seat}", open: open,
            summary {
                span { class: "dot", style: "background:{player_color(p.seat)}" }
                span { class: "name", "{p.name}" }
                span { class: amount_class(p.balance), "{format_euro(p.balance)}" }
            }

            WinDonut {
                wins: total.wins,
                games: total.games,
                win_name: "Siege".to_string(),
                lose_name: "Niederlagen".to_string(),
                label: win_label,
            }

            h4 { "Bilanz" }
            p { class: "legend-note", "Tippe auf eine Zahl: Dann siehst du die Rangliste aller Spieler dazu." }
            TapKv { label: "Bilanz", value: signed_euro(p.balance), metric: Metric::Balance, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Ø pro Spiel", value: average, metric: Metric::AveragePerGame, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Bestes Spiel", value: signed_euro(p.best_game), metric: Metric::BestGame, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Schlechtestes Spiel", value: signed_euro(p.worst_game), metric: Metric::WorstGame, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Gespielt", value: p.games_played.to_string(), metric: Metric::Played, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Ausgesetzt", value: p.games_sat_out.to_string(), metric: Metric::SatOut, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Gegeben", value: (p.games_dealt + p.deals_without_game).to_string(), metric: Metric::Dealt, me: me.clone(), peers: peers.clone(), open: compare }
            if p.deals_without_game > 0 {
                p { class: "muted small", "Davon {p.deals_without_game}× gegeben, ohne dass gespielt wurde (zusammengeworfen oder Stock)." }
            }

            h4 { "Siege nach Rolle" }
            p { class: "legend-note", "Grün gewonnen, orange verloren. Die Länge zeigt die Zahl der Spiele." }
            for (label, role, metric) in roles {
                Tap { key: "{label}", metric, me: me.clone(), peers: peers.clone(), open: compare,
                    StackedBar {
                        label: label.to_string(),
                        won: role.wins,
                        lost: lost(role),
                        max: role_scale,
                        value: share(role),
                    }
                }
            }

            h4 { "Ansagen" }
            for (label, role, metric) in calls {
                Tap { key: "{label}", metric, me: me.clone(), peers: peers.clone(), open: compare,
                    StackedBar {
                        label: label.to_string(),
                        won: role.wins,
                        lost: lost(role),
                        max: call_scale,
                        value: if role.games == 0 { "–".to_string() } else { share(role) },
                    }
                }
            }
            TapKv { label: "Selbst gedoppelt", value: count_share(p.doubles_given(), p.games_played), metric: Metric::Doubles, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Tout gespielt", value: if p.touts.games == 0 { "–".to_string() } else { share(p.touts) }, metric: Metric::Touts, me: me.clone(), peers: peers.clone(), open: compare }
            if p.sies.games > 0 {
                TapKv { label: "Davon Sie", value: share(p.sies), metric: Metric::Sies, me: me.clone(), peers: peers.clone(), open: compare }
            }
            h4 { "Wie oft gedoppelt?" }
            p { class: "legend-note", "Seine Spiele nach der Zahl der Doppler im Preis (Klopfen, Kontra, Re, Tout, Bock)." }
            for (times, count) in p.by_doublings.iter().map(|(t, c)| (*t, *c)) {
                Tap { key: "{times}", metric: Metric::Doubled(times), me: me.clone(), peers: peers.clone(), open: compare,
                    ShareBar {
                        label: doubling_label(times),
                        part: count,
                        total: doubling_scale,
                        value: count_share(count, p.games_played),
                    }
                }
            }
            TapKv { label: "Siege mit Schneider", value: p.schneider_wins.to_string(), metric: Metric::SchneiderWins, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Siege mit Schwarz", value: p.schwarz_wins.to_string(), metric: Metric::SchwarzWins, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Lauf auf der eigenen Seite", value: p.lauf_own_side.to_string(), metric: Metric::LaufOwn, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Lauf auf der Gegenseite", value: p.lauf_other_side.to_string(), metric: Metric::LaufOther, me: me.clone(), peers: peers.clone(), open: compare }
            if p.offences + p.misdeals > 0 || p.penalty_received > 0 {
                h4 { "Vergehen und Neumischen" }
                TapKv { label: "Vergehen", value: format!("{}×", p.offences), metric: Metric::Offences, me: me.clone(), peers: peers.clone(), open: compare }
                TapKv { label: "Neu gemischt", value: format!("{}×", p.misdeals), metric: Metric::Misdeals, me: me.clone(), peers: peers.clone(), open: compare }
                if p.penalty_paid > 0 {
                    TapKv { label: "Strafen gezahlt", value: format_euro(p.penalty_paid), metric: Metric::PenaltyPaid, me: me.clone(), peers: peers.clone(), open: compare }
                }
                if p.penalty_received > 0 {
                    TapKv { label: "Strafen bekommen", value: format_euro(p.penalty_received), metric: Metric::PenaltyReceived, me: me.clone(), peers: peers.clone(), open: compare }
                }
            }
            if !p.stock.is_empty() {
                h4 { "Stock" }
                TapKv { label: "Unterm Strich", value: signed_euro(p.stock.net()), metric: Metric::StockNet, me: me.clone(), peers: peers.clone(), open: compare }
                TapKv { label: "Eingezahlt", value: format_euro(p.stock.paid()), metric: Metric::StockPaid, me: me.clone(), peers: peers.clone(), open: compare }
                TapKv { label: "Bekommen", value: format!("{} ({}×)", format_euro(p.stock.received), p.stock.wins), metric: Metric::StockReceived, me: me.clone(), peers: peers.clone(), open: compare }
                if p.stock.doublings > 0 {
                    Kv { label: "Gedoppelt", value: format!("{}× ({})", p.stock.doublings, format_euro(p.stock.doubled)) }
                }
            }
            if p.wrong_demands > 0 {
                h4 { "Falsch verlangt" }
                TapKv { label: "Preis falsch verlangt", value: format!("{}×", p.wrong_demands), metric: Metric::WrongDemands, me: me.clone(), peers: peers.clone(), open: compare }
                for (mistake, count) in p.wrong_by_kind.iter() {
                    Kv { key: "{mistake.label()}", label: mistake.label(), value: format!("{count}×") }
                }
            }
            DemandNumbers {
                over_demands: p.over_demands,
                under_demands: p.under_demands,
                reclaims: p.reclaims,
                overcharge_gain: p.overcharge_gain,
                reclaim_returned: p.reclaim_returned,
                undercharge_missed: p.undercharge_missed,
                overcharge_loss: p.overcharge_loss,
                reclaim_saved: p.reclaim_saved,
                undercharge_benefit: p.undercharge_benefit,
            }

            h4 { "Serien" }
            p { class: "legend-note", "Spiel für Spiel: + Sieg, − Niederlage, gestrichelt ausgesetzt." }
            ResultStrip { results }
            TapKv { label: "Längste Siegesserie", value: p.longest_win_streak.to_string(), metric: Metric::LongestWinStreak, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Längste Verlustserie", value: p.longest_loss_streak.to_string(), metric: Metric::LongestLossStreak, me: me.clone(), peers: peers.clone(), open: compare }
            Kv { label: "Aktuell", value: streak(p.current_streak) }

            if !kinds.is_empty() {
                h4 { "Nach Spielart" }
                p { class: "legend-note", "Balken: Gewinn nach rechts, Verlust nach links." }
                for (kind, k) in kinds.iter().copied() {
                    DivergingRow {
                        key: "{kind.label()}",
                        label: kind.label().to_string(),
                        seat: None,
                        balance: k.balance,
                        max_abs: kind_scale,
                    }
                }
                table { class: "stat-table",
                    thead { tr { th { "Spielart" } th { "Spiele" } th { "Siege" } th { "Bilanz" } } }
                    tbody {
                        for (kind, k) in kinds {
                            tr { key: "{kind.label()}",
                                td { "{kind.label()}" }
                                td { "{k.games}" }
                                td { "{k.wins}" }
                                td { class: amount_class(k.balance), "{signed_euro(k.balance)}" }
                            }
                        }
                    }
                }
            }
            if !positions.is_empty() {
                h4 { "Nach Platz (ab links vom Geber)" }
                for (index, role) in positions.iter().copied() {
                    StackedBar {
                        key: "{index}",
                        label: position_label(index).to_string(),
                        won: role.wins,
                        lost: lost(role),
                        max: position_scale,
                        value: share(role),
                    }
                }
                table { class: "stat-table",
                    thead { tr { th { "Platz" } th { "Spiele" } th { "Siege" } } }
                    tbody {
                        for (index, r) in positions {
                            tr { key: "{index}",
                                td { "{position_label(index)}" }
                                td { "{r.games}" }
                                td { "{share(r)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn StatsScreen(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    back: Screen,
    focus: Option<usize>,
) -> Element {
    let mut screen = screen;
    // Kommt man über den Namen eines Spielers hierher, scrollt die Seite zu seiner Karte.
    use_effect(move || {
        if let Some(seat) = focus {
            let _ = eval(&format!(
                "setTimeout(() => {{ const e = document.getElementById('player-{seat}'); if (e) e.scrollIntoView({{ block: 'start' }}); }}, 60);"
            ));
        }
    });
    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    let stats = table_stats(t);
    let games = stats.games;

    let ranked = ranking(&stats);
    let money_scale = max_abs(ranked.iter().map(|p| p.balance));
    let kinds = kind_rows(&stats.by_kind);
    let kind_scale = kinds.iter().map(|(_, k)| k.games).max().unwrap_or(0);
    let kind_parts: Vec<(String, u32, String)> = kinds
        .iter()
        .enumerate()
        .map(|(i, (kind, k))| {
            (
                kind.label().to_string(),
                k.games,
                series_color(i).to_string(),
            )
        })
        .collect();
    let aces: Vec<(Suit, RoleStats)> = [Suit::Eichel, Suit::Gras, Suit::Schellen]
        .into_iter()
        .filter_map(|s| {
            stats
                .by_called_ace
                .get(&s)
                .filter(|r| r.games > 0)
                .map(|r| (s, *r))
        })
        .collect();
    let ace_scale = aces.iter().map(|(_, r)| r.games).max().unwrap_or(0);
    let pairs = pair_rows(&stats);
    let lauf_rows: Vec<(GameKind, LaufendeStats)> = GameKind::ALL
        .into_iter()
        .filter_map(|k| {
            stats
                .laufende_by_kind
                .get(&k)
                .filter(|s| s.games > 0)
                .map(|s| (k, *s))
        })
        .collect();
    let lauf_scale = lauf_rows.iter().map(|(_, s)| s.games).max().unwrap_or(0);
    let lauf_total = lauf_rows
        .iter()
        .fold(LaufendeStats::default(), |acc, (_, s)| LaufendeStats {
            games: acc.games + s.games,
            lauf: acc.lauf + s.lauf,
            gegenlauf: acc.gegenlauf + s.gegenlauf,
            lauf_sum: acc.lauf_sum + s.lauf_sum,
            gegenlauf_sum: acc.gegenlauf_sum + s.gegenlauf_sum,
        });
    let lauf_parts: Vec<(String, u32, String)> = vec![
        ("Lauf".to_string(), lauf_total.lauf, LAUF_COLOR.to_string()),
        (
            "Gegenlauf".to_string(),
            lauf_total.gegenlauf,
            GEGENLAUF_COLOR.to_string(),
        ),
        (
            "ohne Lauf".to_string(),
            lauf_total.without(),
            NO_LAUF_COLOR.to_string(),
        ),
    ];
    let pair_scale = pairs.iter().map(|r| r.stats.games).max().unwrap_or(0);
    let declarers = declarer_rate(&stats);
    let results = results_per_player(t);
    // Wer an diesem Abend dabei war, für die Ranglisten beim Antippen einer Zahl.
    let peers: Vec<PeerRow> = stats
        .players
        .iter()
        .filter(|p| p.games_played + p.games_sat_out > 0)
        .map(evening_peer)
        .collect();
    let meta = table_meta(t);
    let doubler_rows: Vec<DoublerRow> = stats
        .players
        .iter()
        .map(|p| DoublerRow {
            name: p.name.clone(),
            knocks: p.knocks.games,
            kontra: p.kontra_given.games,
            re: p.re_given.games,
            played: p.games_played,
        })
        .collect();
    let tout_rows: Vec<ToutRow> = stats
        .players
        .iter()
        .map(|p| ToutRow {
            name: p.name.clone(),
            touts: p.touts,
            sies: p.sies,
        })
        .collect();
    let priciest = stats.most_expensive.and_then(|(index, value)| {
        t.games().get(index).map(|g| {
            (
                format!("{}, {}", title(g), format_euro(value)),
                detail(t, g),
            )
        })
    });
    // Wer wie oft falsch verlangt hat, die meisten zuerst, ohne die, die nie falsch verlangt haben.
    let mut wrong_by_player: Vec<(&schafkopf_core::PlayerStats, u32)> = stats
        .players
        .iter()
        .filter(|p| p.wrong_demands > 0)
        .map(|p| (p, p.wrong_demands))
        .collect();
    wrong_by_player.sort_by_key(|(p, count)| (std::cmp::Reverse(*count), p.seat));
    let wrong_scale = wrong_by_player.iter().map(|(_, c)| *c).max().unwrap_or(0);
    let mut reclaimers: Vec<(&schafkopf_core::PlayerStats, u32)> = stats
        .players
        .iter()
        .filter(|p| p.reclaims > 0)
        .map(|p| (p, p.reclaims))
        .collect();
    reclaimers.sort_by_key(|(p, count)| (std::cmp::Reverse(*count), p.seat));
    let reclaim_scale = reclaimers.iter().map(|(_, c)| *c).max().unwrap_or(0);
    let mut mistakes_sorted: Vec<(schafkopf_core::PriceMistake, u32)> =
        stats.mistake_counts.iter().map(|(m, c)| (*m, *c)).collect();
    mistakes_sorted.sort_by_key(|(m, c)| (std::cmp::Reverse(*c), *m));
    let mut shares = vec![
        ("Mit Klopfen", stats.knocked_games),
        ("Mit Kontra", stats.kontra_games),
        ("Mit Re", stats.re_games),
        ("Mit Schneider", stats.schneider_games),
        ("Mit Schwarz", stats.schwarz_games),
        ("Preis korrigiert", stats.corrected_games),
    ];
    // Bock nur zeigen, wenn es an diesem Abend welchen gab.
    if stats.bock_games > 0 || stats.bock_started > 0 {
        shares.push(("Bockspiele", stats.bock_games));
        shares.push(("Bockrunde ausgelöst", stats.bock_started));
    }

    rsx! {
        h2 { "Statistik" }
        button { class: "secondary", r#type: "button", onclick: move |_| screen.set(back), "← Zurück" }

        if games == 0 {
            p { class: "hint empty", "Noch keine Spiele. Sobald ein Spiel eingetragen ist, erscheinen hier die Zahlen." }
        } else {
            section {
                h3 { "Rangliste" }
                p { class: "legend-note", "Geldstand: Gewinn nach rechts, Verlust nach links." }
                for p in ranked.iter() {
                    DivergingRow {
                        key: "{p.seat}",
                        label: p.name.clone(),
                        seat: Some(p.seat),
                        balance: p.balance,
                        max_abs: money_scale,
                    }
                }
                table { class: "stat-table ranking",
                    thead { tr { th { "" } th { "Spieler" } th { "Stand" } th { "Spiele" } th { "Siege" } } }
                    tbody {
                        for (place, p) in ranked.iter().enumerate() {
                            tr { key: "{p.seat}",
                                td { "{place + 1}." }
                                td { class: "name-cell",
                                    span { class: "dot", style: "background:{player_color(p.seat)}" }
                                    "{p.name}"
                                }
                                td { class: amount_class(p.balance), "{format_euro(p.balance)}" }
                                td { "{p.games_played}" }
                                td { "{percent(overall(p).wins, p.games_played)}" }
                            }
                        }
                    }
                }
            }

            DoublerSection { rows: doubler_rows, intro: "Klopfen, Kontra und Re zusammen, gezählt in den Spielen, bei denen man mitgespielt hat." }
            ToutSection { rows: tout_rows }

            section {
                h3 { "Dieser Abend" }
                Kv { label: "Spiele", value: games.to_string() }
                if stats.stock.no_game_rounds > 0 {
                    Kv { label: "Gegeben (mit Zusammenwerfen und Stock)", value: (games + stats.stock.no_game_rounds).to_string() }
                }
                if stats.bock_games > 0 || stats.bock_started > 0 {
                    Kv { label: "Bockrunden entstanden", value: stats.bock_started.to_string() }
                }
                Kv { label: "Bewegtes Geld", value: format_euro(stats.turnover) }
                if let Some((headline, description)) = priciest {
                    Kv { label: "Teuerstes Spiel", value: headline }
                    p { class: "muted small", "{description}" }
                }
                Kv { label: "Spielerpartei gewann", value: share(declarers) }
                WinDonut {
                    wins: declarers.wins,
                    games: declarers.games,
                    win_name: "Spielerpartei gewann".to_string(),
                    lose_name: "Spielerpartei verlor".to_string(),
                    label: "Wie oft die Spielerpartei gewann".to_string(),
                }
                h4 { "Wie oft wurde gedoppelt?" }
                p { class: "legend-note", "Die Spiele des Abends genau nach der Zahl der Doppler im Preis (Klopfen, Kontra, Re, Tout, Bock)." }
                DoublingTable { rows: stats.by_doublings.iter().map(|(t, c)| (*t, *c)).collect::<Vec<_>>(), games }
                for (times, count) in stats.by_doublings.iter().map(|(t, c)| (*t, *c)) {
                    ShareBar {
                        key: "{times}",
                        label: doubling_label(times),
                        part: count,
                        total: games,
                        value: count_share(count, games),
                    }
                }
                h4 { "Wie oft kam was vor?" }
                for (label, count) in shares {
                    ShareBar {
                        key: "{label}",
                        label: label.to_string(),
                        part: count,
                        total: games,
                        value: count_share(count, games),
                    }
                }
            }

            RecordsSection { rows: evening_records(t, &stats), groups: lauf_groups(&stats.laufende_counts) }

            section {
                h3 { "Spielarten" }
                DonutWithLegend {
                    parts: kind_parts,
                    center: games.to_string(),
                    sub: if games == 1 { "Spiel".to_string() } else { "Spiele".to_string() },
                    label: "Verteilung der Spielarten".to_string(),
                }
                p { class: "legend-note", "Grün gewonnen, orange verloren (Spielerpartei). Die Länge zeigt die Zahl der Spiele." }
                for (kind, k) in kinds.iter().copied() {
                    StackedBar {
                        key: "{kind.label()}",
                        label: kind.label().to_string(),
                        won: k.wins,
                        lost: k.games.saturating_sub(k.wins),
                        max: kind_scale,
                        value: format!("{} von {}", k.wins, k.games),
                    }
                }
                table { class: "stat-table",
                    thead { tr { th { "Spielart" } th { "Spiele" } th { "Gewonnen" } th { "Ø Preis" } } }
                    tbody {
                        for (kind, k) in kinds {
                            tr { key: "{kind.label()}",
                                td { "{kind.label()}" }
                                td { "{k.games}" }
                                td { "{percent(k.wins, k.games)}" }
                                td { "{format_euro(k.balance / Cents::from(k.games))}" }
                            }
                        }
                    }
                }
                p { class: "hint", "Gewonnen heißt: Die Spielerpartei hat gewonnen. Beim Ramsch zählt das nur bei Durchmarsch." }
            }

            if stats.corrected_games > 0 {
                section {
                    h3 { "Falsch verlangt" }
                    p { class: "hint",
                        "Bei {stats.corrected_games} von {games} Spielen wurde der Preis korrigiert. Hier steht, wer falsch verlangt hat und was dabei vergessen oder falsch gemacht wurde."
                    }
                    for (p, count) in wrong_by_player.iter() {
                        ShareBar {
                            key: "{p.seat}",
                            label: p.name.clone(),
                            part: *count,
                            total: wrong_scale,
                            value: format!("{count}×"),
                        }
                    }
                    if !stats.mistake_counts.is_empty() {
                        h4 { "Was schiefging" }
                        for (mistake, count) in mistakes_sorted.iter() {
                            ShareBar {
                                key: "{mistake.label()}",
                                label: mistake.label().to_string(),
                                part: *count,
                                total: stats.corrected_games,
                                value: format!("{count}×"),
                            }
                        }
                    }
                    h4 { "Zu viel oder zu wenig?" }
                    Kv { label: "Zu viel verlangt", value: count_share(stats.overcharged_games, stats.corrected_games) }
                    Kv { label: "Davon zurückverlangt", value: count_share(stats.reclaimed_games, stats.overcharged_games) }
                    Kv { label: "Zu wenig verlangt", value: count_share(stats.undercharged_games, stats.corrected_games) }
                    if stats.overcharge_cents > 0 {
                        Kv { label: "Zu viel verlangtes Geld", value: format_euro(stats.overcharge_cents) }
                        Kv { label: "Zurückverlangt (Geld)", value: format_euro(stats.reclaimed_cents) }
                        Kv { label: "Unterm Strich für die Gewinner", value: signed_euro(stats.overcharge_net_cents) }
                    }
                    if stats.reclaimed_games > 0 {
                        h4 { "Wer hat zurückverlangt?" }
                        for (p, count) in reclaimers.iter() {
                            ShareBar {
                                key: "{p.seat}",
                                label: p.name.clone(),
                                part: *count,
                                total: reclaim_scale,
                                value: format!("{count}×"),
                            }
                        }
                    }
                }
            }

            OffenceSection {
                offences: stats.offences,
                misdeals: stats.misdeals,
                penalty_cents: stats.penalty_cents,
                by_kind: stats.offences_by_kind.iter().map(|(k, c)| (*k, *c)).collect::<Vec<_>>(),
                players: stats.players.iter().map(|p| (p.name.clone(), p.offences, p.misdeals, p.penalty_paid, p.penalty_received)).collect::<Vec<_>>(),
            }
            StockSection {
                stock: stats.stock.clone(),
                players: stats.players.iter().map(|p| (p.name.clone(), p.stock.clone())).collect::<Vec<_>>(),
            }

            if !lauf_rows.is_empty() {
                section {
                    h3 { "Lauf und Gegenlauf" }
                    p { class: "hint",
                        "Lauf: Die Spielerpartei hatte die Laufenden. Gegenlauf: Die Gegner hatten sie. Gezählt wird ab der Mindestzahl Laufende des Tarifs, in der Regel beim Sauspiel ab 3, beim Wenz und Geier ab 2."
                    }
                    DonutWithLegend {
                        parts: lauf_parts,
                        center: (lauf_total.lauf + lauf_total.gegenlauf).to_string(),
                        sub: "Läufe".to_string(),
                        label: "Läufe und Gegenläufe aller Spielarten".to_string(),
                    }
                    p { class: "legend-note", "Blau Lauf, gelb Gegenlauf, grau ohne Lauf. Rechts steht: Lauf · Gegenlauf." }
                    for (kind, row) in lauf_rows.iter().copied() {
                        SegmentBar {
                            key: "{kind.label()}",
                            label: kind.label().to_string(),
                            segments: vec![
                                (row.lauf, LAUF_COLOR.to_string()),
                                (row.gegenlauf, GEGENLAUF_COLOR.to_string()),
                                (row.without(), "var(--line)".to_string()),
                            ],
                            max: lauf_scale,
                            value: format!("{} · {}", row.lauf, row.gegenlauf),
                        }
                    }
                    table { class: "stat-table",
                        thead { tr { th { "Spielart" } th { "Spiele" } th { "Lauf" } th { "Gegenl." } th { "Ø Lfd." } } }
                        tbody {
                            for (kind, row) in lauf_rows {
                                tr { key: "{kind.label()}",
                                    td { "{kind.label()}" }
                                    td { "{row.games}" }
                                    td { "{row.lauf}" }
                                    td { "{row.gegenlauf}" }
                                    td { "{average_laufende(&row)}" }
                                }
                            }
                        }
                    }
                    p { class: "hint", "Ø Lfd.: durchschnittliche Zahl Laufende in den Spielen mit Lauf oder Gegenlauf." }
                }
            }

            if !aces.is_empty() {
                section {
                    h3 { "Gerufene Sau" }
                    for (suit, r) in aces.iter().copied() {
                        StackedBar {
                            key: "{suit.label()}",
                            label: suit.label().to_string(),
                            won: r.wins,
                            lost: lost(r),
                            max: ace_scale,
                            value: share(r),
                        }
                    }
                    table { class: "stat-table",
                        thead { tr { th { "Sau" } th { "Sauspiele" } th { "Gewonnen" } } }
                        tbody {
                            for (suit, r) in aces {
                                tr { key: "{suit.label()}",
                                    td {
                                        SuitIcon { suit }
                                        "{suit.label()}"
                                    }
                                    td { "{r.games}" }
                                    td { "{share(r)}" }
                                }
                            }
                        }
                    }
                }
            }

            if !pairs.is_empty() {
                section {
                    h3 { "Wer mit wem" }
                    p { class: "hint", "Sauspiele und Hochzeiten, in denen beide zusammen gespielt haben. Grün gewonnen, orange verloren." }
                    for row in pairs {
                        StackedBar {
                            key: "{row.first}-{row.second}",
                            label: format!("{} & {}", row.first, row.second),
                            won: row.stats.wins,
                            lost: lost(row.stats),
                            max: pair_scale,
                            value: share(row.stats),
                        }
                    }
                }
            }

            section {
                h3 { "Die Spieler im Detail" }
                for p in ranked.iter().map(|p| (*p).clone()) {
                    PlayerDetail {
                        key: "{p.seat}",
                        results: results[p.seat].clone(),
                        open: focus == Some(p.seat),
                        peers: peers.clone(),
                        player: p,
                    }
                }
            }

            TariffSection { tariffs: meta.tariffs.clone(), tariff_changes: meta.tariff_changes, rule_sets: meta.rule_sets.clone(), rule_set_changes: meta.rule_set_changes, evenings: None }
            EveningPresence { rows: meta.presence.clone(), offset_min: t.utc_offset_min }
        }

        div { class: "actions",
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(back), "Zurück" }
        }
    }
}

/// Zahlen eines Spielers zu falsch verlangten Preisen: wie oft zu viel oder zu wenig verlangt, wie oft
/// zurückverlangt, und das Geld, das dadurch zu viel oder zu wenig floss. Nur, was vorkam.
#[component]
pub(super) fn DemandNumbers(
    over_demands: u32,
    under_demands: u32,
    reclaims: u32,
    overcharge_gain: Cents,
    reclaim_returned: Cents,
    undercharge_missed: Cents,
    overcharge_loss: Cents,
    reclaim_saved: Cents,
    undercharge_benefit: Cents,
) -> Element {
    let money = [
        ("Zu viel bekommen (nicht aufgefallen)", overcharge_gain),
        ("Zurückgeben müssen", reclaim_returned),
        ("Zu wenig bekommen", undercharge_missed),
        ("Zu viel gezahlt (nicht zurückverlangt)", overcharge_loss),
        ("Zurückverlangt (zurückgeholt)", reclaim_saved),
        ("Weniger gezahlt als nötig", undercharge_benefit),
    ];
    if over_demands + under_demands + reclaims == 0 && money.iter().all(|(_, c)| *c == 0) {
        return rsx! {};
    }
    rsx! {
        h4 { "Zu viel oder zu wenig verlangt" }
        if over_demands > 0 {
            Kv { label: "Zu viel verlangt", value: format!("{over_demands}×") }
        }
        if under_demands > 0 {
            Kv { label: "Zu wenig verlangt", value: format!("{under_demands}×") }
        }
        if reclaims > 0 {
            Kv { label: "Zurückverlangt", value: format!("{reclaims}×") }
        }
        for (label, cents) in money.iter().filter(|(_, c)| *c > 0) {
            Kv { key: "{label}", label: label.to_string(), value: format_euro(*cents) }
        }
    }
}

/// Vergehen und Neumischen: wie oft, welche Art, wer wie oft, und was an Strafen floss. Zeigt nichts,
/// wenn es keine gab.
#[component]
pub(super) fn OffenceSection(
    offences: u32,
    misdeals: u32,
    penalty_cents: Cents,
    by_kind: Vec<(OffenceKind, u32)>,
    /// Pro Spieler: Name, Vergehen, Neumischen, gezahlte und bekommene Strafen.
    players: Vec<(String, u32, u32, Cents, Cents)>,
) -> Element {
    let total = offences + misdeals;
    if total == 0 {
        return rsx! {};
    }
    let mut kinds = by_kind;
    kinds.sort_by_key(|(k, c)| (std::cmp::Reverse(*c), *k));
    let mut rows: Vec<&(String, u32, u32, Cents, Cents)> =
        players.iter().filter(|p| p.1 + p.2 > 0).collect();
    rows.sort_by_key(|p| std::cmp::Reverse(p.1 + p.2));
    let scale = rows.first().map_or(0, |p| p.1 + p.2);
    rsx! {
        section {
            h3 { "Vergehen und Neumischen" }
            p { class: "hint", "Nicht bedient, aus der Reihe gespielt und Ähnliches, dazu Fehler beim Geben (neu gemischt). Wer das Vergehen begangen hat, zahlt die Strafe laut Tarif." }
            Kv { label: "Vergehen", value: format!("{offences}×") }
            Kv { label: "Neu gemischt", value: format!("{misdeals}×") }
            if penalty_cents > 0 {
                Kv { label: "Strafen insgesamt", value: format_euro(penalty_cents) }
            }
            h4 { "Welche Art" }
            for (kind, count) in kinds.iter() {
                ShareBar {
                    key: "{kind.label()}",
                    label: kind.label().to_string(),
                    part: *count,
                    total: total,
                    value: format!("{count}×"),
                }
            }
            h4 { "Wer" }
            for (name, o, m, paid, received) in rows.iter().map(|p| (&p.0, p.1, p.2, p.3, p.4)) {
                ShareBar {
                    key: "{name}",
                    label: name.clone(),
                    part: o + m,
                    total: scale,
                    value: if paid > 0 {
                        format!("{}× · zahlt {}", o + m, format_euro(paid))
                    } else {
                        format!("{}×", o + m)
                    },
                }
                if received > 0 {
                    p { class: "muted small", "{name} bekam {format_euro(received)} an Strafen von anderen." }
                }
            }
        }
    }
}

/// Der Stock: höchster Stand, wie viel hineinfloss und ausgezahlt wurde, und was jeder Spieler eingezahlt
/// und bekommen hat. Zeigt nichts, wenn es keinen Stock gab.
#[component]
pub(super) fn StockSection(stock: StockStats, players: Vec<(String, PlayerStock)>) -> Element {
    if stock.is_empty() {
        return rsx! {};
    }
    let mut rows: Vec<&(String, PlayerStock)> =
        players.iter().filter(|p| !p.1.is_empty()).collect();
    rows.sort_by_key(|p| std::cmp::Reverse(p.1.paid()));
    let scale = rows.first().map_or(0, |p| p.1.paid().max(0) as u32);
    rsx! {
        section {
            h3 { "Stock" }
            p { class: "hint", "Wenn keiner spielt, legt jeder die Einlage in die Mitte. Wer danach ein Sauspiel gewinnt, bekommt den Stock, wer eines verliert, doppelt ihn." }
            Kv { label: "Höchster Stock", value: format_euro(stock.peak) }
            Kv { label: "Keiner spielt", value: format!("{}×", stock.no_game_rounds) }
            Kv { label: "Davon nur zusammengeworfen", value: format!("{}×", stock.reshuffles) }
            Kv { label: "Davon in den Stock eingezahlt", value: format!("{}×", stock.deposit_rounds) }
            Kv { label: "Eingelegt", value: format_euro(stock.deposited) }
            if stock.doublings > 0 {
                Kv { label: "Durch Doppeln dazu", value: format!("{} ({}×)", format_euro(stock.doubled), stock.doublings) }
            }
            Kv { label: "Insgesamt hineingeflossen", value: format_euro(stock.flowed_in()) }
            Kv { label: "Ausgezahlt", value: format!("{} ({}×)", format_euro(stock.paid_out), stock.payouts) }
            if stock.splits > 0 {
                Kv { label: "An alle aufgeteilt", value: format!("{} ({}×)", format_euro(stock.split_out), stock.splits) }
            }
            if stock.left > 0 {
                Kv { label: "Liegt noch im Stock", value: format_euro(stock.left) }
            }
            h4 { "Wer hat eingezahlt" }
            for (name, p) in rows.iter().map(|r| (&r.0, &r.1)) {
                ShareBar {
                    key: "{name}",
                    label: name.clone(),
                    part: p.paid().max(0) as u32,
                    total: scale,
                    value: format_euro(p.paid()),
                }
                p { class: "muted small",
                    "{name}: eingelegt {format_euro(p.deposited)}"
                    if p.doublings > 0 { ", gedoppelt {p.doublings}× ({format_euro(p.doubled)})" }
                    if p.wins > 0 { ", Stock gewonnen {p.wins}× ({format_euro(p.received)})" }
                    if p.split > 0 { ", beim Aufteilen bekommen {format_euro(p.split)}" }
                    ", unterm Strich "
                    span { class: amount_class(p.net()), "{signed_euro(p.net())}" }
                }
            }
        }
    }
}

/// Farben für die Doppler-Stufen (Okabe/Ito), von "ohne Doppler" bis "mehrfach".
const DOUBLING_COLORS: [&str; 7] = [
    "#0072B2", "#E69F00", "#009E73", "#CC79A7", "#56B4E9", "#D55E00", "#999999",
];

/// Wie viele Spiele einfach, verdoppelt, vervierfacht usw. waren: Tabelle mit Kuchendiagramm.
/// `rows` sind `(Zahl der Doppler, Zahl der Spiele)`, Stufen ohne Spiele dürfen fehlen.
#[component]
pub(super) fn DoublingTable(rows: Vec<(u32, u32)>, games: u32) -> Element {
    if games == 0 {
        return rsx! {};
    }
    let rows: Vec<(u32, u32)> = rows.into_iter().filter(|r| r.1 > 0).collect();
    let parts: Vec<(String, u32, String)> = rows
        .iter()
        .map(|&(times, count)| {
            (
                doubling_label(times),
                count,
                DOUBLING_COLORS[(times as usize).min(DOUBLING_COLORS.len() - 1)].to_string(),
            )
        })
        .collect();
    rsx! {
        DonutWithLegend {
            parts,
            center: games.to_string(),
            sub: if games == 1 { "Spiel".to_string() } else { "Spiele".to_string() },
            label: "Anteil der Spiele nach Zahl der Doppler".to_string(),
        }
        table { class: "stat-table",
            thead {
                tr {
                    th { "Preis" }
                    th { "Spiele" }
                    th { "Anteil" }
                }
            }
            tbody {
                for (times, count) in rows {
                    tr { key: "{times}",
                        td { "{doubling_label(times)}" }
                        td { "{count}" }
                        td { "{percent(count, games)}" }
                    }
                }
            }
        }
    }
}
