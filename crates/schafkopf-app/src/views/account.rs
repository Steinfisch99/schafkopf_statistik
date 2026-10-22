//! Geldstand aller eingetragenen Spieler über alle Abende, mit Startwert und Nachträgen.

use dioxus::prelude::*;
use schafkopf_charts::format_euro;
use schafkopf_core::{parse_euro, Cents, Library, Table};

use super::common::{persist_library, Chip, ConfirmButton, Notice, TextField};
use crate::summary::{german_date, signed_euro};
use crate::{now_secs, storage};

/// Liest einen Betrag, der auch negativ sein darf: `-5,50`, `+2` oder `3`.
pub fn parse_signed_euro(text: &str) -> Option<Cents> {
    let text = text.trim();
    let (negative, rest) = match text.chars().next() {
        Some('-' | '−') => (true, &text[text.chars().next()?.len_utf8()..]),
        Some('+') => (false, &text[1..]),
        _ => (false, text),
    };
    let cents = parse_euro(rest)?;
    Some(if negative { -cents } else { cents })
}

/// Ändert die Bibliothek und speichert sie. Misslingt es, bleibt die Bibliothek, wie sie war.
fn change(
    mut library: Signal<Library>,
    mut notice: Signal<Option<Notice>>,
    change: impl FnOnce(&mut Library) -> Result<String, String>,
) -> bool {
    let before = library.read().clone();
    let result = {
        let mut lib = library.write();
        change(&mut lib)
    }
    .and_then(|text| persist_library(&library.read()).map(|()| text));
    match result {
        Ok(text) => {
            notice.set(Some(Notice::ok(text)));
            true
        }
        Err(e) => {
            library.set(before);
            notice.set(Some(Notice::error(e)));
            false
        }
    }
}

/// Der Geldstand jedes Spielers aus der Liste der gespeicherten Spieler: Startwert, was die Abende
/// (auch der laufende) gebracht haben und Nachträge. Startwert und Nachträge lassen sich hier eintragen.
#[component]
pub fn AccountSection(
    table: Signal<Option<Table>>,
    library: Signal<Library>,
    notice: Signal<Option<Notice>>,
) -> Element {
    // Die früheren Abende liegen auf der Platte. Sie ändern sich nicht, solange dieser Bildschirm offen ist.
    let past = use_hook(storage::list_past);
    let mut player = use_signal(String::new);
    let mut start_text = use_signal(String::new);
    let mut amount_text = use_signal(String::new);
    let mut note_text = use_signal(String::new);
    let mut deduct = use_signal(|| false);

    let rows = {
        let mut tables: Vec<Table> = past.iter().map(|p| p.table.clone()).collect();
        if let Some(t) = table.read().as_ref() {
            tables.push(t.clone());
        }
        library.read().account(&tables)
    };
    if rows.is_empty() {
        return rsx! {};
    }
    let players: Vec<String> = rows.iter().map(|r| r.name.clone()).collect();
    let selected = {
        let chosen = player();
        if players.contains(&chosen) {
            chosen
        } else {
            players[0].clone()
        }
    };
    let adjustments: Vec<(usize, String)> = library
        .read()
        .adjustments()
        .iter()
        .enumerate()
        .rev()
        .map(|(i, a)| {
            let when =
                a.at.map(|t| format!("{} · ", german_date(t.max(0) as u64)))
                    .unwrap_or_default();
            let note = a
                .note
                .as_deref()
                .map(|n| format!(" · {n}"))
                .unwrap_or_default();
            (
                i,
                format!("{when}{}: {}{note}", a.player, signed_euro(a.amount)),
            )
        })
        .collect();

    let set_start = {
        let selected = selected.clone();
        move |_| {
            let text = start_text();
            let Some(amount) = parse_signed_euro(&text) else {
                return notice.set(Some(Notice::error(
                    "Startwert: Betrag nicht lesbar, zum Beispiel 25 oder -12,50",
                )));
            };
            let name = selected.clone();
            if change(library, notice, |lib| {
                lib.set_start_balance(&name, amount)
                    .map(|()| format!("Startwert von {name}: {}", format_euro(amount)))
                    .map_err(|e| e.to_string())
            }) {
                start_text.set(String::new());
            }
        }
    };
    let add = {
        let selected = selected.clone();
        move |_| {
            let Some(mut amount) = parse_euro(&amount_text()) else {
                return notice.set(Some(Notice::error(
                    "Betrag nicht lesbar, zum Beispiel 5 oder 12,50",
                )));
            };
            if deduct() {
                amount = -amount;
            }
            let name = selected.clone();
            let note = note_text();
            let at = Some(now_secs() as i64);
            if change(library, notice, |lib| {
                lib.add_adjustment(&name, amount, at, Some(note))
                    .map(|_| format!("{name}: {} nachgetragen", signed_euro(amount)))
                    .map_err(|e| e.to_string())
            }) {
                amount_text.set(String::new());
                note_text.set(String::new());
            }
        }
    };
    let remove = move |index: usize| {
        change(library, notice, |lib| match lib.remove_adjustment(index) {
            Some(a) => Ok(format!("Nachtrag für {} zurückgenommen", a.player)),
            None => Err("Diesen Nachtrag gibt es nicht mehr".to_owned()),
        });
    };

    rsx! {
        section {
            h3 { "Geldstand" }
            p { class: "hint", "Alle Abende zusammen (ohne Testabende), mit Startwert und Nachträgen." }
            ul { class: "balances",
                for row in rows.iter() {
                    li { key: "{row.name}",
                        div { class: "game-line",
                            b { "{row.name}" }
                            span { class: if row.total() < 0 { "amount neg" } else { "amount" }, "{format_euro(row.total())}" }
                        }
                        if row.start != 0 || row.adjustments != 0 {
                            div { class: "muted small",
                                "Abende {signed_euro(row.played)} · Start {format_euro(row.start)} · Nachträge {signed_euro(row.adjustments)}"
                            }
                        }
                    }
                }
            }
            details {
                summary { "Startwert und Nachträge" }
                label { class: "field",
                    span { "Spieler" }
                    select {
                        aria_label: "Spieler für Startwert und Nachtrag",
                        onchange: move |e| player.set(e.value()),
                        for name in players.iter().cloned() {
                            option { key: "{name}", value: "{name}", selected: name == selected, "{name}" }
                        }
                    }
                }
                p { class: "hint", "Startwert: Was {selected} schon hatte, bevor die App mitgeschrieben hat. Der neue Wert ersetzt den alten, 0 löscht ihn." }
                TextField { label: "Startwert (€)", value: start_text(), mode: "text", oninput: move |v| start_text.set(v) }
                div { class: "actions",
                    button { class: "secondary", r#type: "button", onclick: set_start, "Startwert setzen" }
                }
                p { class: "hint", "Nachtrag: Gewinn oder Verlust, der sonst nirgends steht, zum Beispiel von einem Abend woanders." }
                div { class: "chips",
                    Chip { label: "Gutschreiben".to_string(), on: !deduct(), onclick: move |_| deduct.set(false) }
                    Chip { label: "Abziehen".to_string(), on: deduct(), onclick: move |_| deduct.set(true) }
                }
                TextField { label: "Betrag (€)", value: amount_text(), mode: "decimal", oninput: move |v| amount_text.set(v) }
                TextField { label: "Notiz (zum Beispiel wo gespielt)", value: note_text(), oninput: move |v| note_text.set(v) }
                div { class: "actions",
                    button { class: "secondary", r#type: "button", onclick: add, "Nachtragen" }
                }
                if !adjustments.is_empty() {
                    ol { class: "adjustments",
                        for (index, line) in adjustments.iter() {
                            li { key: "{index}", class: "offence",
                                "{line}"
                                ConfirmButton {
                                    label: "Zurücknehmen",
                                    confirm_label: "Ja, zurücknehmen",
                                    onconfirm: {
                                        let index = *index;
                                        move |_| remove(index)
                                    },
                                }
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
    fn signed_amounts_are_read_with_either_minus_sign() {
        assert_eq!(parse_signed_euro("12,50"), Some(1250));
        assert_eq!(parse_signed_euro("-12,50"), Some(-1250));
        assert_eq!(parse_signed_euro("- 3"), Some(-300));
        assert_eq!(parse_signed_euro("−3"), Some(-300));
        assert_eq!(parse_signed_euro("+2"), Some(200));
        assert_eq!(parse_signed_euro(""), None);
        assert_eq!(parse_signed_euro("-"), None);
        assert_eq!(parse_signed_euro("--1"), None);
    }
}
