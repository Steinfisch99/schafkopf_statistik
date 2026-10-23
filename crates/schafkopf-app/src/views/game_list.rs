//! Die Liste der Spiele eines Abends, das neueste zuerst.

use dioxus::prelude::*;
use schafkopf_core::{GameKind, Table};

use super::common::{ConfirmButton, Notice};
use super::suit_icon::SuitIcon;
use crate::summary::{detail, offence_text, signed_euro, stock_text, title};
use crate::{storage, Screen};

/// Die Spiele eines Abends zum Aufklappen. Mit `editable` hat jedes Spiel einen Link zum Bearbeiten,
/// bei früheren Abenden fehlt er.
#[component]
pub fn GameList(table: Signal<Option<Table>>, screen: Signal<Screen>, editable: bool) -> Element {
    let mut screen = screen;
    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    let games = t.games();
    if games.is_empty() {
        return rsx! {};
    }

    rsx! {
        details { class: "history", open: games.len() <= 5,
            summary { "Spiele ({games.len()})" }
            p { class: "kind-legend",
                span { class: "kind-team", "Sauspiel, Hochzeit" }
                span { class: "kind-solo", "Solo, Wenz, Geier, Bettel" }
                span { class: "kind-ramsch", "Ramsch" }
            }
            ol { reversed: true, start: "{games.len()}",
                for (index, game) in games.iter().enumerate().rev() {
                    li { key: "{index}", class: kind_class(game.input.kind),
                        div { class: "game-line",
                            b {
                                if let Some(suit) = game.input.suit {
                                    SuitIcon { suit }
                                }
                                "{title(game)}"
                            }
                            span { class: if game.payouts[game.input.declarer] < 0 { "amount neg" } else { "amount" },
                                "{signed_euro(game.payouts[game.input.declarer])}"
                            }
                        }
                        div { class: "muted", "{detail(t, game)}" }
                        if editable {
                            button {
                                class: "link edit",
                                r#type: "button",
                                onclick: move |_| screen.set(Screen::EditGame(index)),
                                "Bearbeiten"
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Die Farbklasse eines Eintrags: Spiele zu zweit, Alleinspiele und Ramsch sehen verschieden aus.
fn kind_class(kind: GameKind) -> &'static str {
    if kind.is_team_game() {
        "kind-team"
    } else if kind.is_solo() {
        "kind-solo"
    } else {
        "kind-ramsch"
    }
}

/// Die Vergehen und das Neumischen eines Abends. Mit `editable` lässt sich ein Eintrag zurücknehmen.
#[component]
pub fn OffenceList(
    table: Signal<Option<Table>>,
    notice: Signal<Option<Notice>>,
    editable: bool,
) -> Element {
    let mut table = table;
    let mut notice = notice;
    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    if t.offences.is_empty() {
        return rsx! {};
    }
    let lines: Vec<String> = t.offences.iter().map(|o| offence_text(t, o)).collect();
    drop(guard);

    let mut remove = move |index: usize| {
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        if t.remove_offence(index).is_none() {
            return;
        }
        let saved = storage::save(t, None);
        drop(guard);
        notice.set(Some(match saved {
            Ok(_) => Notice::ok("Eintrag zurückgenommen"),
            Err(e) => Notice::error(format!("Speichern fehlgeschlagen: {e}")),
        }));
    };

    rsx! {
        details { class: "history",
            summary { "Vergehen und Neumischen ({lines.len()})" }
            ol {
                for (index, line) in lines.iter().enumerate() {
                    li { key: "{index}", class: "offence",
                        "{line}"
                        if editable {
                            ConfirmButton {
                                label: "Zurücknehmen",
                                confirm_label: "Ja, zurücknehmen",
                                onconfirm: move |_| remove(index),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Die Bewegungen am Stock: Einlagen ("Keiner spielt"), Auszahlungen und Verdopplungen. Mit `editable`
/// lässt sich eine Einlage zurücknehmen, die anderen Einträge folgen den Spielen.
#[component]
pub fn StockList(
    table: Signal<Option<Table>>,
    notice: Signal<Option<Notice>>,
    editable: bool,
) -> Element {
    let mut table = table;
    let mut notice = notice;
    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    if t.stock_moves.is_empty() {
        return rsx! {};
    }
    let lines: Vec<(String, bool)> = t
        .stock_moves
        .iter()
        .map(|m| (stock_text(t, m), m.kind.is_manual()))
        .collect();
    drop(guard);

    let mut remove = move |index: usize| {
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        if t.remove_stock_move(index).is_none() {
            return;
        }
        let saved = storage::save(t, None);
        drop(guard);
        notice.set(Some(match saved {
            Ok(_) => Notice::ok("Eintrag zurückgenommen"),
            Err(e) => Notice::error(format!("Speichern fehlgeschlagen: {e}")),
        }));
    };

    rsx! {
        details { class: "history",
            summary { "Stock ({lines.len()})" }
            ol {
                for (index, (line, removable)) in lines.iter().enumerate() {
                    li { key: "{index}", class: "offence",
                        "{line}"
                        if editable && *removable {
                            ConfirmButton {
                                label: "Zurücknehmen",
                                confirm_label: "Ja, zurücknehmen",
                                onconfirm: move |_| remove(index),
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_games_solos_and_ramsch_get_different_classes() {
        assert_eq!(kind_class(GameKind::Sauspiel), "kind-team");
        assert_eq!(kind_class(GameKind::Hochzeit), "kind-team");
        for solo in [
            GameKind::Wenz,
            GameKind::Geier,
            GameKind::Farbsolo,
            GameKind::Farbwenz,
            GameKind::Farbgeier,
            GameKind::Bettel,
        ] {
            assert_eq!(kind_class(solo), "kind-solo", "{solo:?}");
        }
        assert_eq!(kind_class(GameKind::Ramsch), "kind-ramsch");
        // Jede Spielart hat eine Klasse, und es gibt drei verschiedene.
        let classes: std::collections::BTreeSet<_> =
            GameKind::ALL.into_iter().map(kind_class).collect();
        assert_eq!(classes.len(), 3);
    }
}
