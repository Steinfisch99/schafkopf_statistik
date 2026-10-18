//! Statistik über mehrere Abende: Standardmäßig zählt jeder Abend, einzelne lassen sich abwählen.
//! Testabende sind nie dabei.

use std::collections::BTreeSet;

use dioxus::prelude::*;
use schafkopf_charts::{format_euro, player_color};
use schafkopf_core::{overall_stats, OverallPlayer, OverallStats, PriceMistake, Table};

use super::common::{Chip, Notice};
use super::compare::{overall_peer, Metric, PeerRow, Tap, TapKv};
use super::meta_sections::{OverallPresence, TariffSection};
use super::stat_widgets::{
    DivergingRow, DoublerSection, ShareBar, StackedBar, ToutSection, WinDonut,
};
use super::stats_screen::{amount_class, count_share, lost, DemandNumbers, Kv, OffenceSection};
use crate::stats_text::{
    doubling_label, kind_rows, max_abs, percent, share, streak, DoublerRow, ToutRow,
};
use crate::summary::{evening_date, evening_times, games_text, signed_euro};
use crate::{storage, Screen};

/// Der laufende Abend hat keinen Zeitstempel im Archiv, er zählt unter dieser Nummer.
const CURRENT: u64 = 0;

#[component]
fn OverallPlayerCard(player: OverallPlayer, games_total: u32, peers: Vec<PeerRow>) -> Element {
    let p = player;
    let compare = use_signal(|| None::<Metric>);
    let me = p.name.clone();
    let doubling_scale = p.by_doublings.values().copied().max().unwrap_or(0);
    let total = p.overall();
    let kinds = kind_rows(&p.by_kind);
    let average_game = if p.games_played > 0 {
        signed_euro(p.balance / i64::from(p.games_played))
    } else {
        "–".to_owned()
    };
    let average_evening = if p.evenings > 0 {
        signed_euro(p.balance / i64::from(p.evenings))
    } else {
        "–".to_owned()
    };
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
    let evening_scale = max_abs(p.evening_balances.iter().copied());
    let _ = games_total;

    rsx! {
        details { class: "player-detail",
            summary {
                span { class: "name", "{p.name}" }
                span { class: amount_class(p.balance), "{format_euro(p.balance)}" }
            }
            WinDonut {
                wins: total.wins,
                games: total.games,
                win_name: "Siege".to_string(),
                lose_name: "Niederlagen".to_string(),
                label: format!("Siege und Niederlagen von {}", p.name),
            }

            h4 { "Bilanz" }
            p { class: "legend-note", "Tippe auf eine Zahl: Dann siehst du die Rangliste aller Spieler dazu." }
            TapKv { label: "Bilanz", value: signed_euro(p.balance), metric: Metric::Balance, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Abende", value: p.evenings.to_string(), metric: Metric::Evenings, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Abende im Plus", value: format!("{} von {}", p.evenings_won(), p.evenings), metric: Metric::EveningsWon, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Ø pro Abend", value: average_evening, metric: Metric::AveragePerEvening, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Ø pro Spiel", value: average_game, metric: Metric::AveragePerGame, me: me.clone(), peers: peers.clone(), open: compare }
            if let Some(best) = p.best_evening() {
                TapKv { label: "Bester Abend", value: signed_euro(best), metric: Metric::BestEvening, me: me.clone(), peers: peers.clone(), open: compare }
            }
            if let Some(worst) = p.worst_evening() {
                TapKv { label: "Schlechtester Abend", value: signed_euro(worst), metric: Metric::WorstEvening, me: me.clone(), peers: peers.clone(), open: compare }
            }
            TapKv { label: "Bestes Spiel", value: signed_euro(p.best_game), metric: Metric::BestGame, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Schlechtestes Spiel", value: signed_euro(p.worst_game), metric: Metric::WorstGame, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Gespielt", value: p.games_played.to_string(), metric: Metric::Played, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Ausgesetzt", value: p.games_sat_out.to_string(), metric: Metric::SatOut, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Gegeben", value: p.games_dealt.to_string(), metric: Metric::Dealt, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Längste Siegesserie", value: streak(p.longest_win_streak as i32), metric: Metric::LongestWinStreak, me: me.clone(), peers: peers.clone(), open: compare }
            TapKv { label: "Längste Verlustserie", value: streak(-(p.longest_loss_streak as i32)), metric: Metric::LongestLossStreak, me: me.clone(), peers: peers.clone(), open: compare }

            if p.evening_balances.len() > 1 {
                h4 { "Ergebnis je Abend" }
                p { class: "legend-note", "Älteste Abende zuerst. Gewinn nach rechts, Verlust nach links." }
                for (i, b) in p.evening_balances.iter().enumerate() {
                    DivergingRow {
                        key: "{i}",
                        label: format!("Abend {}", i + 1),
                        seat: None,
                        balance: *b,
                        max_abs: evening_scale,
                    }
                }
            }

            h4 { "Siege nach Rolle" }
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

            h4 { "Doppler" }
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
            Kv { label: "In gedoppelten Spielen (mindestens)", value: count_share(p.doubled_games, p.games_played) }
            Kv { label: "In vervierfachten Spielen (mindestens)", value: count_share(p.quadrupled_games, p.games_played) }
            p { class: "legend-note", "Seine Spiele genau nach der Zahl der Doppler im Preis (Klopfen, Kontra, Re, Tout, Bock)." }
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

            if !kinds.is_empty() {
                h4 { "Nach Spielart" }
                table { class: "stat-table",
                    thead { tr { th { "Spielart" } th { "Spiele" } th { "Siege" } th { "Gewinn" } } }
                    tbody {
                        for (kind, k) in kinds {
                            tr { key: "{kind.label()}",
                                td { "{kind.label()}" }
                                td { "{k.games}" }
                                td { "{percent(k.wins, k.games)}" }
                                td { class: amount_class(k.balance), "{format_euro(k.balance)}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Die Zahlen aller gewählten Abende zusammen, ohne die Wahl der Abende.
#[component]
fn OverallNumbers(stats: OverallStats) -> Element {
    let games = stats.games;
    let mut ranked: Vec<OverallPlayer> = stats.players.clone();
    ranked.sort_by_key(|p| (std::cmp::Reverse(p.balance), p.name.clone()));
    let money_scale = max_abs(ranked.iter().map(|p| p.balance));
    // Alle Spieler der gewählten Abende für die Ranglisten beim Antippen einer Zahl.
    let peers: Vec<PeerRow> = ranked.iter().map(overall_peer).collect();
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
    let kinds = kind_rows(&stats.by_kind);
    let kind_scale = kinds.iter().map(|(_, k)| k.games).max().unwrap_or(0);
    let top_doublings = stats.by_doublings.keys().next_back().copied().unwrap_or(0);
    let mut mistakes: Vec<(PriceMistake, u32)> =
        stats.mistake_counts.iter().map(|(m, c)| (*m, *c)).collect();
    mistakes.sort_by_key(|(m, c)| (std::cmp::Reverse(*c), *m));
    let mut shares = vec![
        ("Mit Klopfen", stats.knocked_games),
        ("Mit Kontra", stats.kontra_games),
        ("Mit Re", stats.re_games),
        ("Mit Schneider", stats.schneider_games),
        ("Mit Schwarz", stats.schwarz_games),
        ("Preis korrigiert", stats.corrected_games),
    ];
    if stats.bock_games > 0 || stats.bock_started > 0 {
        shares.push(("Bockspiele", stats.bock_games));
        shares.push(("Bockrunden ausgelöst", stats.bock_started));
    }

    rsx! {
        section {
            h3 { "Rangliste" }
            p { class: "legend-note", "Gesamtgewinn über alle gewählten Abende: Gewinn nach rechts, Verlust nach links." }
            for (i, p) in ranked.iter().enumerate() {
                DivergingRow {
                    key: "{p.name}",
                    label: p.name.clone(),
                    seat: Some(i),
                    balance: p.balance,
                    max_abs: money_scale,
                }
            }
            table { class: "stat-table ranking",
                thead { tr { th { "" } th { "Spieler" } th { "Stand" } th { "Abende" } th { "Spiele" } th { "Siege" } } }
                tbody {
                    for (place, p) in ranked.iter().enumerate() {
                        tr { key: "{p.name}",
                            td { "{place + 1}." }
                            td { class: "name-cell",
                                span { class: "dot", style: "background:{player_color(place)}" }
                                "{p.name}"
                            }
                            td { class: amount_class(p.balance), "{format_euro(p.balance)}" }
                            td { "{p.evenings}" }
                            td { "{p.games_played}" }
                            td { "{percent(p.overall().wins, p.games_played)}" }
                        }
                    }
                }
            }
        }

        DoublerSection { rows: doubler_rows, intro: "Klopfen, Kontra und Re zusammen über alle gewählten Abende, gezählt in den Spielen, bei denen man mitgespielt hat." }
        ToutSection { rows: tout_rows }
        TariffSection { tariffs: stats.tariffs.clone(), tariff_changes: stats.tariff_changes, rule_sets: stats.rule_sets.clone(), rule_set_changes: stats.rule_set_changes, evenings: Some(stats.evenings) }
        OverallPresence { players: stats.players.clone(), evening_starts: stats.evening_starts.clone(), evening_lengths: stats.evening_lengths.clone() }

        section {
            h3 { "Alle Abende zusammen" }
            Kv { label: "Abende", value: stats.evenings.to_string() }
            Kv { label: "Spiele", value: games.to_string() }
            Kv { label: "Bewegtes Geld", value: format_euro(stats.turnover) }
            if stats.bock_games > 0 || stats.bock_started > 0 {
                Kv { label: "Bockrunden entstanden", value: format!("{} an {} Abenden", stats.bock_started, stats.evenings) }
            }
            Kv { label: "Spielerpartei gewann", value: share(stats.declarer_side) }
            WinDonut {
                wins: stats.declarer_side.wins,
                games: stats.declarer_side.games,
                win_name: "Spielerpartei gewann".to_string(),
                lose_name: "Spielerpartei verlor".to_string(),
                label: "Wie oft die Spielerpartei gewann".to_string(),
            }

            h4 { "Wie oft wurde gedoppelt?" }
            p { class: "legend-note", "Anteil der Spiele nach der Zahl der Verdopplungen im Preis (Klopfen, Kontra, Re, Tout, Bock)." }
            for times in 0..=top_doublings {
                ShareBar {
                    key: "{times}",
                    label: doubling_label(times),
                    part: stats.by_doublings.get(&times).copied().unwrap_or(0),
                    total: games,
                    value: count_share(stats.by_doublings.get(&times).copied().unwrap_or(0), games),
                }
            }
            Kv { label: "Mindestens gedoppelt", value: count_share(stats.doubled_at_least(1), games) }
            Kv { label: "Mindestens vervierfacht", value: count_share(stats.doubled_at_least(2), games) }

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

        section {
            h3 { "Spielarten" }
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
        }

        if stats.corrected_games > 0 {
            section {
                h3 { "Falsch verlangt" }
                p { class: "hint", "Bei {stats.corrected_games} von {games} Spielen wurde der Preis korrigiert." }
                for (mistake, count) in mistakes.iter() {
                    ShareBar {
                        key: "{mistake.label()}",
                        label: mistake.label().to_string(),
                        part: *count,
                        total: stats.corrected_games,
                        value: format!("{count}×"),
                    }
                }
                for p in ranked.iter().filter(|p| p.wrong_demands > 0) {
                    ShareBar {
                        key: "{p.name}",
                        label: p.name.clone(),
                        part: p.wrong_demands,
                        total: stats.corrected_games,
                        value: format!("{}×", p.wrong_demands),
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
                    for p in ranked.iter().filter(|p| p.reclaims > 0) {
                        ShareBar {
                            key: "{p.name}",
                            label: p.name.clone(),
                            part: p.reclaims,
                            total: stats.reclaimed_games,
                            value: format!("{}×", p.reclaims),
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
            players: ranked.iter().map(|p| (p.name.clone(), p.offences, p.misdeals, p.penalty_paid, p.penalty_received)).collect::<Vec<_>>(),
        }

        section {
            h3 { "Spieler" }
            for p in ranked.iter() {
                OverallPlayerCard { key: "{p.name}", player: p.clone(), games_total: games, peers: peers.clone() }
            }
        }
    }
}

#[component]
pub fn OverallScreen(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
) -> Element {
    let mut screen = screen;
    let _ = notice;
    // Beim Öffnen einmal gelesen: Die früheren Abende ändern sich hier nicht.
    let past = use_signal(storage::list_past);
    let current: Option<Table> = table
        .read()
        .as_ref()
        .filter(|t| !t.debug && !t.games().is_empty())
        .cloned();

    // Alle Abende ohne Testabende, der älteste zuerst, der laufende zuletzt.
    let mut evenings: Vec<(u64, Table)> = past
        .read()
        .iter()
        .rev()
        .filter(|p| !p.table.debug)
        .filter(|p| !p.table.games().is_empty())
        .map(|p| (p.stamp, p.table.clone()))
        .collect();
    if let Some(t) = current {
        evenings.push((CURRENT, t));
    }
    let test_evenings = past.read().iter().filter(|p| p.table.debug).count();

    // Standardmäßig zählt jeder Abend. Wer abwählt, merkt sich die abgewählten.
    let mut off = use_signal(BTreeSet::<u64>::new);
    let chosen: Vec<&Table> = evenings
        .iter()
        .filter(|(key, _)| !off.read().contains(key))
        .map(|(_, t)| t)
        .collect();
    let stats = overall_stats(&chosen);
    let keys: Vec<u64> = evenings.iter().map(|(k, _)| *k).collect();

    rsx! {
        h2 { "Gesamtstatistik" }
        button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::PastList), "← Zurück" }

        if evenings.is_empty() {
            p { class: "hint empty",
                "Noch keine Abende mit Spielen. Sobald ein Abend beendet ist (oder läuft), zählt er hier."
            }
        } else {
            section {
                h3 { "Welche Abende?" }
                p { class: "hint",
                    "Standardmäßig zählt jeder Abend. Tippe einen an, um ihn herauszunehmen oder wieder dazuzunehmen."
                }
                div { class: "actions",
                    button {
                        class: "secondary",
                        r#type: "button",
                        onclick: {
                            let keys = keys.clone();
                            move |_| off.set(keys.iter().copied().collect())
                        },
                        "Keinen"
                    }
                    button { class: "secondary", r#type: "button", onclick: move |_| off.set(BTreeSet::new()), "Alle" }
                }
                div { class: "evening-pick",
                    for (key, t) in evenings.iter().rev() {
                        Chip {
                            key: "{key}",
                            label: format!(
                                "{}{} · {}{}",
                                if *key == CURRENT { "Laufender Abend, " } else { "" },
                                evening_date(t, *key),
                                games_text(t.games().len()),
                                evening_times(t).map(|x| format!(" · {x}")).unwrap_or_default(),
                            ),
                            on: !off.read().contains(key),
                            onclick: {
                                let key = *key;
                                move |_| {
                                    let mut set = off.write();
                                    if !set.remove(&key) {
                                        set.insert(key);
                                    }
                                }
                            },
                        }
                    }
                }
                p { class: "muted small",
                    "{chosen.len()} von {evenings.len()} Abenden gewählt."
                    if test_evenings > 0 {
                        " {test_evenings} Testabende zählen nie."
                    }
                }
            }

            if chosen.is_empty() {
                p { class: "hint empty", "Kein Abend gewählt." }
            } else {
                OverallNumbers { stats }
            }
        }
    }
}
