//! Wer am Tisch sitzt: Spieler, die später kommen, dazusetzen, früher Gehende austragen und
//! Zurückkehrende wieder an ihren Platz holen. Die Spiele, der Stand und die Statistik der
//! Gegangenen bleiben erhalten.

use dioxus::prelude::*;
use schafkopf_core::{Library, Table};

use super::common::{persist_library, ConfirmButton, Notice, TextField};
use crate::summary::event_text;
use crate::{now_secs, storage, Screen};

/// Meldung nach einer Änderung am Tisch, mit dem Ergebnis des Speicherns.
fn saved_notice(mut notice: Signal<Option<Notice>>, text: String, saved: std::io::Result<()>) {
    notice.set(Some(match saved {
        Ok(()) => Notice::ok(text),
        Err(e) => Notice::error(format!(
            "{text}, aber die Datei konnte nicht geschrieben werden: {e}"
        )),
    }));
}

#[component]
pub fn SeatsScreen(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
    library: Signal<Library>,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;
    let mut library = library;
    // Der Name des Neuen: aus der Liste gewählt oder getippt.
    let mut chosen = use_signal(String::new);
    let mut typed = use_signal(String::new);
    // Hinter welchem Sitzplatz der Neue sitzt; leer: am Ende der Runde.
    let mut after = use_signal(String::new);

    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    let present: Vec<usize> = t.present_seats();
    let away: Vec<usize> = t.ring().into_iter().filter(|&s| !t.is_present(s)).collect();
    let names: Vec<String> = t.players.clone();
    let can_leave = t.player_count() > 2;
    let can_add = t.player_count() < 10;
    let dealer = t.next_dealer();
    // Aus der Bibliothek kommen nur Namen zur Wahl, die an diesem Abend noch nicht dabei sind.
    let suggestions: Vec<String> = library
        .read()
        .players_sorted()
        .into_iter()
        .filter(|p| !names.contains(p))
        .collect();
    drop(guard);

    let mut change = move |change: &mut dyn FnMut(&mut Table) -> Result<String, String>| {
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        let text = match change(t) {
            Ok(text) => text,
            Err(e) => {
                drop(guard);
                return notice.set(Some(Notice::error(e)));
            }
        };
        let saved = storage::save(t, None);
        drop(guard);
        saved_notice(notice, text, saved.map(|_| ()));
    };

    // Zum Umsetzen: Hinter wem sitzt jemand, wenn er eins nach oben (zuerst) oder unten rückt.
    let rows: Vec<(usize, Option<usize>, Option<usize>)> = present
        .iter()
        .enumerate()
        .map(|(i, &seat)| {
            let n = present.len();
            let up = (i > 0).then(|| present[(i + n - 2) % n]);
            let down = (i + 1 < n).then(|| present[i + 1]);
            (seat, up, down)
        })
        .collect();
    let mut reseat = move |seat: usize, after: usize| {
        change(&mut |t: &mut Table| {
            t.move_player(seat, after, Some(now_secs() as i64))
                .map_err(|e| e.to_string())?;
            Ok(format!(
                "{} sitzt jetzt hinter {}",
                t.players[seat], t.players[after]
            ))
        });
    };

    let add = move |_| {
        let name = if typed.read().trim().is_empty() {
            chosen.read().clone()
        } else {
            typed.read().trim().to_owned()
        };
        let behind = after.read().parse::<usize>().ok();
        let mut added = false;
        change(&mut |t: &mut Table| {
            let seat = t
                .add_player(&name, behind, Some(now_secs() as i64))
                .map_err(|e| e.to_string())?;
            added = true;
            Ok(format!("{} sitzt jetzt mit am Tisch", t.players[seat]))
        });
        if added {
            // Ein neu getippter Name steht beim nächsten Abend zur Auswahl.
            if library.write().ensure_players(&[name]) > 0 {
                if let Err(e) = persist_library(&library.read()) {
                    notice.set(Some(Notice::error(e)));
                }
            }
            chosen.set(String::new());
            typed.set(String::new());
            after.set(String::new());
        }
    };

    rsx! {
        h2 { "Am Tisch" }
        p { class: "hint",
            "Wer früher geht, später kommt oder sich umsetzt: Hier änderst du, wer wo sitzt. Die Spiele, der Stand und die Statistik der Gegangenen bleiben erhalten, und die Uhrzeiten werden im Protokoll des Abends festgehalten. Die Änderung gilt ab dem nächsten Spiel."
        }
        p { class: "muted", "Nächster Geber: " b { "{names[dealer]}" } }

        section {
            h3 { "Am Tisch ({present.len()})" }
            ul { class: "seat-list",
                for (row, &seat) in present.iter().enumerate() {
                    li { key: "{seat}",
                        span { class: "name", "{names[seat]}" }
                        if present.len() > 2 {
                            if let Some(up) = rows[row].1 {
                                button {
                                    class: "secondary reseat",
                                    r#type: "button",
                                    "aria-label": "{names[seat]} eins nach oben setzen",
                                    onclick: move |_| reseat(seat, up),
                                    "▲"
                                }
                            }
                            if let Some(down) = rows[row].2 {
                                button {
                                    class: "secondary reseat",
                                    r#type: "button",
                                    "aria-label": "{names[seat]} eins nach unten setzen",
                                    onclick: move |_| reseat(seat, down),
                                    "▼"
                                }
                            }
                        }
                        if can_leave {
                            ConfirmButton {
                                label: "Geht",
                                confirm_label: "Ja, geht",
                                onconfirm: move |_| {
                                    change(&mut |t: &mut Table| {
                                        t.leave(seat, Some(now_secs() as i64)).map_err(|e| e.to_string())?;
                                        Ok(format!("{} sitzt nicht mehr am Tisch", t.players[seat]))
                                    });
                                },
                            }
                        }
                    }
                }
            }
            if !can_leave {
                p { class: "hint", "Es müssen mindestens zwei Spieler am Tisch bleiben." }
            }
        }

        if !away.is_empty() {
            section {
                h3 { "Nicht am Tisch ({away.len()})" }
                ul { class: "seat-list",
                    for &seat in away.iter() {
                        li { key: "{seat}",
                            span { class: "name", "{names[seat]}" }
                            button {
                                class: "secondary",
                                r#type: "button",
                                disabled: !can_add,
                                onclick: move |_| {
                                    change(&mut |t: &mut Table| {
                                        t.rejoin(seat, Some(now_secs() as i64)).map_err(|e| e.to_string())?;
                                        Ok(format!("{} sitzt wieder am Tisch", t.players[seat]))
                                    });
                                },
                                "Kommt zurück"
                            }
                        }
                    }
                }
            }
        }

        EventLog { table }

        section {
            h3 { "Neuer Spieler" }
            if !can_add {
                p { class: "hint", "Am Tisch sitzen schon zehn Spieler." }
            } else {
                if !suggestions.is_empty() {
                    label { class: "field",
                        span { "Aus der Spielerliste" }
                        select {
                            onchange: move |e| {
                                chosen.set(e.value());
                                typed.set(String::new());
                            },
                            option { value: "", selected: chosen().is_empty(), "– wählen –" }
                            for name in suggestions.iter() {
                                option { key: "{name}", value: "{name}", selected: chosen() == *name, "{name}" }
                            }
                        }
                    }
                }
                TextField {
                    label: "Oder ein neuer Name".to_string(),
                    value: typed(),
                    hint: "zum Beispiel Eva".to_string(),
                    oninput: move |v: String| {
                        if !v.trim().is_empty() {
                            chosen.set(String::new());
                        }
                        typed.set(v);
                    },
                }
                label { class: "field",
                    span { "Sitzt hinter" }
                    select {
                        onchange: move |e| after.set(e.value()),
                        option { value: "", selected: after().is_empty(), "am Ende der Runde" }
                        for &seat in present.iter() {
                            option { key: "{seat}", value: "{seat}", selected: after() == seat.to_string(), "{names[seat]}" }
                        }
                    }
                }
                p { class: "hint", "Gegeben wird in der Reihenfolge, in der die Spieler sitzen." }
                div { class: "actions",
                    button {
                        class: "primary",
                        r#type: "button",
                        disabled: chosen().is_empty() && typed().trim().is_empty(),
                        onclick: add,
                        "Dazusetzen"
                    }
                }
            }
        }

        div { class: "actions",
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "Zurück" }
        }
    }
}

/// Wer an diesem Abend kam, ging oder sich umsetzte, mit Uhrzeit. Ohne Einträge zeigt sie nichts.
#[component]
pub fn EventLog(table: Signal<Option<Table>>) -> Element {
    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    if t.events.is_empty() {
        return rsx! {};
    }
    let lines: Vec<String> = t.events.iter().map(|e| event_text(t, e)).collect();
    rsx! {
        details {
            summary { "Wer kam und ging ({lines.len()})" }
            ul { class: "event-log",
                for (i, line) in lines.iter().enumerate() {
                    li { key: "{i}", "{line}" }
                }
            }
        }
    }
}
