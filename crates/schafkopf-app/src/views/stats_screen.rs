//! Statistik: Rangliste, Überblick über den Abend, Spielarten, Paare und jeder Spieler im Detail.

use dioxus::prelude::*;
use schafkopf_charts::{format_euro, player_color};
use schafkopf_core::{table_stats, Cents, PlayerStats, Suit, Table};

use crate::stats_text::{
    average_per_game, declarer_rate, kind_rows, overall, pair_rows, percent, position_label,
    ranking, share, streak,
};
use crate::summary::{detail, signed_euro, title};
use crate::Screen;

/// "3 (21 %)": eine Anzahl und ihr Anteil an allen Spielen.
fn count_share(count: u32, total: u32) -> String {
    format!("{count} ({})", percent(count, total))
}

fn amount_class(cents: Cents) -> &'static str {
    if cents < 0 {
        "amount neg"
    } else {
        "amount"
    }
}

#[component]
fn Kv(label: String, value: String) -> Element {
    rsx! {
        div { class: "kv",
            span { "{label}" }
            b { "{value}" }
        }
    }
}

#[component]
fn PlayerDetail(player: PlayerStats) -> Element {
    let p = player;
    let kinds = kind_rows(&p.by_kind);
    let positions: Vec<(usize, _)> = p
        .by_position
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, r)| r.games > 0)
        .collect();
    let average = average_per_game(&p).map_or("–".to_string(), signed_euro);

    rsx! {
        details { class: "player-detail",
            summary {
                span { class: "dot", style: "background:{player_color(p.seat)}" }
                span { class: "name", "{p.name}" }
                span { class: amount_class(p.balance), "{format_euro(p.balance)}" }
            }
            h4 { "Bilanz" }
            Kv { label: "Ø pro Spiel", value: average }
            Kv { label: "Bestes Spiel", value: signed_euro(p.best_game) }
            Kv { label: "Schlechtestes Spiel", value: signed_euro(p.worst_game) }
            Kv { label: "Gespielt", value: p.games_played.to_string() }
            Kv { label: "Ausgesetzt", value: p.games_sat_out.to_string() }
            Kv { label: "Gegeben", value: p.games_dealt.to_string() }

            h4 { "Siege nach Rolle" }
            Kv { label: "Als Spieler", value: share(p.as_declarer) }
            Kv { label: "Als Mitspieler", value: share(p.as_partner) }
            Kv { label: "Als Gegenspieler", value: share(p.as_defender) }

            h4 { "Ansagen" }
            Kv { label: "Geklopft, danach gewonnen", value: share(p.knocks) }
            Kv { label: "Kontra gegeben, gewonnen", value: share(p.kontra_given) }
            Kv { label: "Re gegeben, gewonnen", value: share(p.re_given) }
            Kv { label: "Siege mit Schneider", value: p.schneider_wins.to_string() }
            Kv { label: "Siege mit Schwarz", value: p.schwarz_wins.to_string() }

            h4 { "Serien" }
            Kv { label: "Längste Siegesserie", value: p.longest_win_streak.to_string() }
            Kv { label: "Längste Verlustserie", value: p.longest_loss_streak.to_string() }
            Kv { label: "Aktuell", value: streak(p.current_streak) }

            if !kinds.is_empty() {
                h4 { "Nach Spielart" }
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
pub fn StatsScreen(table: Signal<Option<Table>>, screen: Signal<Screen>) -> Element {
    let mut screen = screen;
    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    let stats = table_stats(t);
    let games = stats.games;

    let ranked = ranking(&stats);
    let kinds = kind_rows(&stats.by_kind);
    let aces: Vec<(Suit, _)> = [Suit::Eichel, Suit::Gras, Suit::Schellen]
        .into_iter()
        .filter_map(|s| {
            stats
                .by_called_ace
                .get(&s)
                .filter(|r| r.games > 0)
                .map(|r| (s, *r))
        })
        .collect();
    let pairs = pair_rows(&stats);
    let priciest = stats.most_expensive.and_then(|(index, value)| {
        t.games().get(index).map(|g| {
            (
                format!("{}, {}", title(g), format_euro(value)),
                detail(t, g),
            )
        })
    });

    rsx! {
        h2 { "Statistik" }
        button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "← Zurück" }

        if games == 0 {
            p { class: "hint empty", "Noch keine Spiele. Sobald ein Spiel eingetragen ist, erscheinen hier die Zahlen." }
        } else {
            section {
                h3 { "Rangliste" }
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

            section {
                h3 { "Dieser Abend" }
                Kv { label: "Spiele", value: games.to_string() }
                Kv { label: "Bewegtes Geld", value: format_euro(stats.turnover) }
                if let Some((headline, description)) = priciest {
                    Kv { label: "Teuerstes Spiel", value: headline }
                    p { class: "muted small", "{description}" }
                }
                Kv { label: "Spielerpartei gewann", value: share(declarer_rate(&stats)) }
                Kv { label: "Mit Klopfen", value: count_share(stats.knocked_games, games) }
                Kv { label: "Mit Kontra", value: count_share(stats.kontra_games, games) }
                Kv { label: "Mit Re", value: count_share(stats.re_games, games) }
                Kv { label: "Mit Schneider", value: count_share(stats.schneider_games, games) }
                Kv { label: "Mit Schwarz", value: count_share(stats.schwarz_games, games) }
                Kv { label: "Preis von Hand korrigiert", value: count_share(stats.corrected_games, games) }
            }

            section {
                h3 { "Spielarten" }
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

            if !aces.is_empty() {
                section {
                    h3 { "Gerufene Sau" }
                    table { class: "stat-table",
                        thead { tr { th { "Sau" } th { "Sauspiele" } th { "Gewonnen" } } }
                        tbody {
                            for (suit, r) in aces {
                                tr { key: "{suit.label()}",
                                    td { "{suit.label()}" }
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
                    p { class: "hint", "Sauspiele und Hochzeiten, in denen beide zusammen gespielt haben." }
                    for row in pairs {
                        div { class: "kv", key: "{row.first}-{row.second}",
                            span { "{row.first} & {row.second}" }
                            b { "{share(row.stats)}" }
                        }
                    }
                }
            }

            section {
                h3 { "Die Spieler im Detail" }
                for p in ranked.iter().map(|p| (*p).clone()) {
                    PlayerDetail { key: "{p.seat}", player: p }
                }
            }
        }

        div { class: "actions",
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "Zurück" }
        }
    }
}
