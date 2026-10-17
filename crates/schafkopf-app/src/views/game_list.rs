//! Die Liste der Spiele eines Abends, das neueste zuerst.

use dioxus::prelude::*;
use schafkopf_core::Table;

use crate::summary::{detail, signed_euro, title};
use crate::Screen;

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
            ol { reversed: true, start: "{games.len()}",
                for (index, game) in games.iter().enumerate().rev() {
                    li { key: "{index}",
                        div { class: "game-line",
                            b { "{title(game)}" }
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
