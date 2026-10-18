//! Ein Vergehen eintragen (nicht bedient, aus der Reihe gespielt, ...) oder festhalten, dass neu
//! gemischt wurde. Es kostet die Strafe aus dem Tarif, falls dieser Strafzahlungen vorsieht.

use dioxus::prelude::*;
use schafkopf_charts::{format_amount, format_euro};
use schafkopf_core::{parse_euro, OffenceKind, Table};

use super::common::{Chip, Notice};
use crate::{now_secs, storage, Screen};

#[component]
pub fn OffenceScreen(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;
    let mut kind = use_signal(|| None::<OffenceKind>);
    let mut offender = use_signal(|| None::<usize>);
    // Die Strafe als Text; leer: die Strafe aus dem Tarif.
    let mut amount = use_signal(String::new);
    // Wer die Strafe bekommt, wenn von Hand gewählt; sonst die Mitspieler der laufenden Runde.
    let mut payees = use_signal(|| None::<Vec<usize>>);

    let guard = table.read();
    let Some(t) = guard.as_ref() else {
        return rsx! {};
    };
    let present = t.present_seats();
    let names = t.players.clone();
    let dealer = t.next_dealer();
    let penalties = t.tariff.penalties_enabled;
    let default_amount = kind().map_or(0, |k| t.penalty_for(k));
    let recipients: Vec<usize> = match (offender(), payees()) {
        (Some(who), Some(list)) => list.into_iter().filter(|&s| s != who).collect(),
        (Some(who), None) => t.default_penalty_recipients(who),
        _ => Vec::new(),
    };
    let others = recipients.len() as i64;
    drop(guard);

    let typed = amount();
    let parsed = if typed.trim().is_empty() {
        Some(default_amount)
    } else {
        parse_euro(&typed)
    };
    let cost = if penalties { parsed } else { Some(0) };

    let save_recipients = recipients.clone();
    let save = move |_| {
        let (Some(k), Some(who)) = (kind(), offender()) else {
            return;
        };
        let text = amount();
        let custom = if text.trim().is_empty() {
            None
        } else {
            match parse_euro(&text) {
                Some(v) => Some(v),
                None => {
                    return notice.set(Some(Notice::error(
                        "Strafe: Betrag nicht lesbar, zum Beispiel 0,50",
                    )))
                }
            }
        };
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        let added = t
            .add_offence(
                k,
                who,
                Some(save_recipients.clone()),
                custom,
                Some(now_secs() as i64),
            )
            .map(|o| (o.offender, o.amount));
        let message = match added {
            Ok((offender, paid)) => {
                let name = t.players[offender].clone();
                if paid > 0 {
                    format!(
                        "{}: {name}, Strafe {} an jeden Mitspieler",
                        k.label(),
                        format_euro(paid)
                    )
                } else {
                    format!("{}: {name}, ohne Strafe", k.label())
                }
            }
            Err(e) => {
                drop(guard);
                return notice.set(Some(Notice::error(e.to_string())));
            }
        };
        let saved = storage::save(t, None);
        drop(guard);
        notice.set(Some(match saved {
            Ok(_) => Notice::ok(message),
            Err(e) => Notice::error(format!("{message}, aber Speichern fehlgeschlagen: {e}")),
        }));
        screen.set(Screen::Table);
    };

    rsx! {
        section {
            h2 { "Vergehen oder Neumischen" }
            p { class: "hint", "Was ist passiert?" }
            div { class: "chips",
                for k in OffenceKind::ALL {
                    Chip {
                        key: "{k.label()}",
                        label: k.label().to_string(),
                        on: kind() == Some(k),
                        onclick: move |_| {
                            kind.set(Some(k));
                            amount.set(String::new());
                            // Beim Neumischen ist der Geber der Täter.
                            if k == OffenceKind::Misdeal {
                                offender.set(Some(dealer));
                            }
                        },
                    }
                }
            }
            if kind() == Some(OffenceKind::Misdeal) {
                p { class: "hint", "Der Geber bleibt derselbe und mischt neu, der Geber rückt also nicht weiter." }
            }
            p { class: "hint", if kind() == Some(OffenceKind::Misdeal) { "Wer hat falsch gegeben?" } else { "Wer hat es getan?" } }
            div { class: "chips",
                for seat in present.clone() {
                    Chip {
                        key: "{seat}",
                        label: names[seat].clone(),
                        on: offender() == Some(seat),
                        onclick: move |_| {
                            offender.set(Some(seat));
                            payees.set(None);
                        },
                    }
                }
            }
            if offender().is_some() {
                p { class: "hint", "Wer bekommt die Strafe? Standard sind die Mitspieler der laufenden Runde." }
                div { class: "chips",
                    for seat in present.clone().into_iter().filter(|&s| Some(s) != offender()) {
                        Chip {
                            key: "{seat}",
                            label: names[seat].clone(),
                            on: recipients.contains(&seat),
                            onclick: {
                                let current = recipients.clone();
                                move |_| {
                                    let mut list = current.clone();
                                    if let Some(i) = list.iter().position(|&s| s == seat) {
                                        list.remove(i);
                                    } else {
                                        list.push(seat);
                                    }
                                    payees.set(Some(list));
                                }
                            },
                        }
                    }
                }
            }
            if penalties {
                label { class: "field",
                    span { "Strafe je Mitspieler (€)" }
                    input {
                        r#type: "text",
                        inputmode: "decimal",
                        autocomplete: "off",
                        placeholder: "leer lassen = {format_amount(default_amount)}",
                        value: "{typed}",
                        oninput: move |e| amount.set(e.value()),
                    }
                }
                match cost {
                    Some(c) if c > 0 && offender().is_some() => rsx! {
                        p { class: "hint", "{names[offender().unwrap_or(0)]} zahlt {format_euro(c)} an jeden der {others} Empfänger, zusammen {format_euro(c * others)}." }
                    },
                    Some(_) => rsx! { p { class: "hint", "Keine Strafe, das Vergehen wird nur gezählt." } },
                    None => rsx! { p { class: "hint", role: "status", "Der Betrag ist nicht lesbar, zum Beispiel 0,50." } },
                }
            } else {
                p { class: "hint", "Der Tarif sieht keine Strafzahlungen vor. Das Vergehen wird nur gezählt." }
            }
            div { class: "actions",
                button {
                    class: "primary big",
                    r#type: "button",
                    disabled: kind().is_none() || offender().is_none() || recipients.is_empty() || cost.is_none(),
                    onclick: save,
                    "Eintragen"
                }
                button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "Abbrechen" }
            }
        }
    }
}
