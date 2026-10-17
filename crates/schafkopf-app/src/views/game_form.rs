//! Ein Spiel eintragen oder ein früheres bearbeiten: Spielart wählen, dann in der Übersicht der
//! Mitspielenden alles ankreuzen.

use dioxus::prelude::*;
use schafkopf_charts::format_euro;
use schafkopf_core::{GameKind, Suit, Table};

use super::common::{Chip, ConfirmButton, Notice};
use crate::draft::GameDraft;
use crate::summary::signed_euro;
use crate::{now_secs, storage, Screen};

const CALLABLE_SUITS: [Suit; 3] = [Suit::Eichel, Suit::Gras, Suit::Schellen];

/// `editing` ist die Position des Spiels in der Liste, wenn ein bestehendes bearbeitet wird.
#[component]
pub fn GameForm(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
    editing: Option<usize>,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;
    let mut draft = use_signal(|| {
        table
            .read()
            .as_ref()
            .and_then(|t| match editing {
                Some(index) => GameDraft::from_game(t, index),
                None => Some(GameDraft::new(t)),
            })
            .expect(
                "Das Formular gibt es nur für einen vorhandenen Tisch und ein vorhandenes Spiel",
            )
    });

    let players: Vec<String> = table
        .read()
        .as_ref()
        .map(|t| t.players.clone())
        .unwrap_or_default();
    let d = draft();
    let kind = d.kind;
    let preview = table.read().as_ref().map(|t| match editing {
        Some(index) => d.preview_edit(t, index),
        None => d.preview(t),
    });
    let problem = d.problem();
    // Nur die Spielarten des Regelwerks, dazu die des bearbeiteten Spiels, falls sie danach
    // deaktiviert wurde.
    let shown_kinds: Vec<GameKind> = table
        .read()
        .as_ref()
        .map(|t| {
            GameKind::ALL
                .into_iter()
                .filter(|&k| t.is_enabled(k) || k == kind)
                .collect()
        })
        .unwrap_or_else(|| GameKind::ALL.to_vec());
    let declarer_label = match (kind, d.won) {
        (GameKind::Ramsch, true) => "Durchmarsch",
        (GameKind::Ramsch, false) => "Verlierer",
        _ => "Spieler",
    };

    let save = move |_| {
        let input = match draft.read().to_input(Some(now_secs() as i64)) {
            Ok(input) => input,
            Err(e) => return notice.set(Some(Notice::error(e))),
        };
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        let (index, saved, verb) = match editing {
            Some(index) => {
                if let Err(e) = t.replace_game(index, input) {
                    return notice.set(Some(Notice::error(e.to_string())));
                }
                // Beim Ändern kann sich jede Zeile ändern, deshalb wird die ganze CSV neu geschrieben.
                (index, storage::save(t, None), "geändert")
            }
            None => {
                if let Err(e) = t.add_game(input) {
                    return notice.set(Some(Notice::error(e.to_string())));
                }
                let index = t.games().len() - 1;
                (index, storage::save(t, Some(index)), "gespeichert")
            }
        };
        drop(guard);
        match saved {
            Ok(_) => notice.set(Some(Notice::ok(format!("Spiel {} {verb}", index + 1)))),
            Err(e) => notice.set(Some(Notice::error(format!(
                "Spiel {} ist {verb}, aber die Datei konnte nicht geschrieben werden: {e}",
                index + 1
            )))),
        }
        screen.set(Screen::Table);
    };

    let delete = move |_| {
        let Some(index) = editing else { return };
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        if t.remove_game(index).is_none() {
            return;
        }
        let saved = storage::save(t, None);
        drop(guard);
        match saved {
            Ok(_) => notice.set(Some(Notice::ok(format!("Spiel {} gelöscht", index + 1)))),
            Err(e) => notice.set(Some(Notice::error(format!(
                "Gelöscht, aber die Datei konnte nicht geschrieben werden: {e}"
            )))),
        }
        screen.set(Screen::Table);
    };

    rsx! {
        if let Some(index) = editing {
            h2 { "Spiel {index + 1} bearbeiten" }
            p { class: "hint", "Der Preis wird mit dem Tarif gerechnet, der damals galt." }
        } else {
            h2 { "Spiel {table.read().as_ref().map_or(0, |t| t.games().len()) + 1} eintragen" }
        }

        section {
            label { class: "field",
                span { "Geber" }
                select {
                    onchange: move |e| {
                        if let Ok(seat) = e.value().parse::<usize>() {
                            if let Some(t) = table.read().as_ref() {
                                draft.write().set_dealer(t, seat);
                            }
                        }
                    },
                    for (i, name) in players.iter().enumerate() {
                        option { key: "{i}", value: "{i}", selected: d.dealer == i, "{name}" }
                    }
                }
            }
            if players.len() > 4 {
                details {
                    summary { "Wer spielt mit? ({d.participants.len()} von {players.len()})" }
                    div { class: "chips",
                        for (i, name) in players.iter().enumerate() {
                            Chip {
                                key: "{i}",
                                label: name.clone(),
                                on: d.participants.contains(&i),
                                disabled: !d.participants.contains(&i) && d.participants.len() >= 4,
                                onclick: move |_| {
                                    let on = !draft.read().participants.contains(&i);
                                    if let Some(t) = table.read().as_ref() {
                                        draft.write().set_participant(t, i, on);
                                    }
                                },
                            }
                        }
                    }
                }
            }
        }

        section {
            label { class: "field",
                span { "Spielart" }
                select {
                    onchange: move |e| {
                        if let Ok(i) = e.value().parse::<usize>() {
                            if let Some(kind) = GameKind::ALL.get(i) {
                                draft.write().set_kind(*kind);
                            }
                        }
                    },
                    for (i, k) in GameKind::ALL.iter().enumerate().filter(|(_, k)| shown_kinds.contains(k)) {
                        option { key: "{i}", value: "{i}", selected: *k == kind, "{k.label()}" }
                    }
                }
            }
            if kind == GameKind::Sauspiel {
                p { class: "hint", "Gerufene Sau" }
                div { class: "chips",
                    for suit in CALLABLE_SUITS {
                        Chip {
                            key: "{suit.label()}",
                            label: suit.label().to_string(),
                            on: d.suit == Some(suit),
                            onclick: move |_| draft.write().suit = Some(suit),
                        }
                    }
                }
            }
            if kind.needs_trump_suit() {
                p { class: "hint", "Trumpffarbe" }
                div { class: "chips",
                    for suit in Suit::ALL {
                        Chip {
                            key: "{suit.label()}",
                            label: suit.label().to_string(),
                            on: d.suit == Some(suit),
                            onclick: move |_| draft.write().suit = Some(suit),
                        }
                    }
                }
            }
        }

        section {
            h3 { "Mitspielende" }
            for seat in d.participants.clone() {
                div { class: "card", key: "{seat}",
                    div { class: "card-name", "{players[seat]}" }
                    div { class: "chips",
                        Chip {
                            label: declarer_label.to_string(),
                            on: d.declarer == Some(seat),
                            onclick: move |_| draft.write().toggle_declarer(seat),
                        }
                        if kind.is_team_game() {
                            Chip {
                                label: "Mitspieler".to_string(),
                                on: d.partner == Some(seat),
                                onclick: move |_| draft.write().toggle_partner(seat),
                            }
                        }
                        Chip {
                            label: "Geklopft".to_string(),
                            on: d.knockers.contains(&seat),
                            onclick: move |_| draft.write().toggle_knock(seat),
                        }
                        if kind != GameKind::Ramsch {
                            Chip {
                                label: "Kontra".to_string(),
                                on: d.kontra_by == Some(seat),
                                disabled: d.is_on_declarer_team(seat),
                                onclick: move |_| draft.write().toggle_kontra(seat),
                            }
                        }
                    }
                }
            }
            div { class: "chips",
                if kind != GameKind::Ramsch {
                    Chip {
                        label: "Re".to_string(),
                        on: d.re,
                        disabled: d.kontra_by.is_none(),
                        onclick: move |_| draft.write().toggle_re(),
                    }
                }
                if kind.allows_tout() {
                    Chip { label: "Tout".to_string(), on: d.tout, onclick: move |_| { let on = !draft.read().tout; draft.write().tout = on; } }
                }
            }
        }

        section {
            h3 { "Ergebnis" }
            if kind == GameKind::Ramsch {
                div { class: "chips",
                    Chip {
                        label: "Durchmarsch".to_string(),
                        on: d.won,
                        onclick: move |_| draft.write().toggle_durchmarsch(),
                    }
                }
                p { class: "hint",
                    if d.won {
                        "Durchmarsch: Wähle oben den Spieler, der alle Stiche gemacht hat."
                    } else {
                        "Wähle oben den Verlierer, den Spieler mit den meisten Augen."
                    }
                }
            } else {
                div { class: "chips",
                    Chip { label: "Gewonnen".to_string(), on: d.won, onclick: move |_| draft.write().won = true }
                    Chip { label: "Verloren".to_string(), on: !d.won, onclick: move |_| draft.write().won = false }
                }
            }
            if kind.has_extras() {
                div { class: "chips",
                    Chip { label: "Schneider".to_string(), on: d.schneider, onclick: move |_| { let on = !draft.read().schneider; draft.write().set_schneider(on); } }
                    Chip { label: "Schwarz".to_string(), on: d.schwarz, onclick: move |_| { let on = !draft.read().schwarz; draft.write().set_schwarz(on); } }
                }
                div { class: "stepper",
                    span { "Laufende" }
                    button {
                        class: "secondary icon",
                        r#type: "button",
                        "aria-label": "Eine Laufende weniger",
                        disabled: d.laufende == 0,
                        onclick: move |_| { let n = draft.read().laufende; draft.write().set_laufende(n.saturating_sub(1)); },
                        "−"
                    }
                    span { class: "count", "{d.laufende}" }
                    button {
                        class: "secondary icon",
                        r#type: "button",
                        "aria-label": "Eine Laufende mehr",
                        disabled: d.laufende >= 8,
                        onclick: move |_| { let n = draft.read().laufende; draft.write().set_laufende(n + 1); },
                        "+"
                    }
                }
            }
        }

        section {
            h3 { "Preis" }
            if let Some(Ok(game)) = &preview {
                p { class: "price", "Berechnet: " b { "{format_euro(game.calculated_value)}" } }
            }
            label { class: "field",
                span { "Verlangter Preis, falls abweichend (€)" }
                input {
                    r#type: "text",
                    inputmode: "decimal",
                    autocomplete: "off",
                    placeholder: "leer lassen = berechneter Preis",
                    value: "{d.price_text}",
                    oninput: move |e| draft.write().price_text = e.value(),
                }
            }
            p { class: "hint", "Der Preis gilt je Gegner. Beim Solo zahlt oder bekommt der Spieler ihn dreifach." }
            if let Some(Ok(game)) = &preview {
                ul { class: "payouts",
                    for seat in d.participants.clone() {
                        li { key: "{seat}",
                            span { "{players[seat]}" }
                            span { class: if game.payouts[seat] < 0 { "amount neg" } else { "amount" }, "{signed_euro(game.payouts[seat])}" }
                        }
                    }
                }
            }
        }

        if let Some(p) = &problem {
            p { class: "hint", role: "status", "{p}" }
        }
        div { class: "actions",
            button {
                class: "primary big",
                r#type: "button",
                disabled: problem.is_some(),
                onclick: save,
                if editing.is_some() { "Änderungen speichern" } else { "Spiel speichern" }
            }
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "Abbrechen" }
            if editing.is_some() {
                ConfirmButton { label: "Spiel löschen", confirm_label: "Ja, löschen", onconfirm: delete }
            }
        }
    }
}
