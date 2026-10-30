//! Ein Spiel eintragen oder ein früheres bearbeiten: Spielart wählen, dann in der Übersicht der
//! Mitspielenden alles ankreuzen.

use dioxus::prelude::*;
use schafkopf_charts::format_euro;
use schafkopf_core::{BockStacking, GameKind, PriceMistake, Suit, Table, MAX_BOCK};

use super::common::{Chip, ConfirmButton, Notice};
use crate::draft::GameDraft;
use crate::summary::{check_rows, signed_euro};
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
    // Wer zur Wahl steht: wer am Tisch sitzt, in Sitzreihenfolge. Beim Bearbeiten kommen die dazu,
    // die damals dabei waren und inzwischen gegangen sind.
    let seats: Vec<usize> = table
        .read()
        .as_ref()
        .map(|t| {
            t.ring()
                .into_iter()
                .filter(|&s| t.is_present(s) || d.dealer == s || d.participants.contains(&s))
                .collect()
        })
        .unwrap_or_default();
    let kind = d.kind;
    let preview = table.read().as_ref().map(|t| match editing {
        Some(index) => d.preview_edit(t, index),
        None => d.preview(t),
    });
    let problem = d.problem();
    // Die Fehlerarten gibt es nur, wenn der Preis wirklich korrigiert wird.
    let price_corrected = matches!(&preview, Some(Ok(g)) if g.demanded_wrong());
    // Wer zu viel verlangt, hat nichts vergessen, sondern dazugedichtet: Die Auswahl passt zur Richtung.
    // Schon gewählte Fehlerarten bleiben sichtbar, auch wenn sie nicht mehr dazu passen.
    let mistake_choices: Vec<PriceMistake> = {
        let too_high = matches!(&preview, Some(Ok(g)) if g.overcharged());
        let mut choices = PriceMistake::for_price(too_high);
        choices.extend(
            d.mistakes
                .iter()
                .filter(|m| !choices.contains(m))
                .copied()
                .collect::<Vec<_>>(),
        );
        choices
    };
    // Zu viel verlangt und der Tarif erlaubt das Zurückverlangen: Die Verlierer können es zurückholen.
    let price_too_high = matches!(
        &preview,
        Some(Ok(g)) if g.overcharged() && g.tariff.as_ref().is_some_and(|t| t.reclaim_enabled)
    );
    let stacked = table
        .read()
        .as_ref()
        .is_some_and(|t| t.bock_stacking == BockStacking::Stacked);
    // Wie viele Bockspiele vor diesem Spiel noch offen sind.
    let bock_pending = table.read().as_ref().map_or(0, |t| match editing {
        Some(index) => t.bock_pending_before(index),
        None => t.bock_pending(),
    });
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
    let bock_hint = match bock_pending {
        0 => "Es ist kein Bock offen. Schwarz löst eine Bockrunde von selbst aus, du kannst sie hier ein- oder ausschalten.".to_string(),
        1 => "Es ist noch 1 Bockspiel offen. Ist dieses Spiel eines, bleibt der Knopf an.".to_string(),
        n => format!("Es sind noch {n} Bockspiele offen. Ist dieses Spiel eines, bleibt der Knopf an."),
    };
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
                    for &i in seats.iter() {
                        option { key: "{i}", value: "{i}", selected: d.dealer == i, "{players[i]}" }
                    }
                }
            }
            if seats.len() > 4 {
                details {
                    summary { "Wer spielt mit? ({d.participants.len()} von {seats.len()})" }
                    div { class: "chips",
                        for &i in seats.iter() {
                            Chip {
                                key: "{i}",
                                label: players[i].clone(),
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
                            icon: Some(suit),
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
                            icon: Some(suit),
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
                        if kind != GameKind::Ramsch {
                            Chip {
                                label: "Re".to_string(),
                                on: d.re_by == Some(seat),
                                disabled: d.kontra_by.is_none() || !d.is_on_declarer_team(seat),
                                onclick: move |_| draft.write().toggle_re(seat),
                            }
                        }
                    }
                }
            }
            div { class: "chips",
                if kind.allows_tout() {
                    Chip { label: "Tout".to_string(), on: d.tout, onclick: move |_| { let on = !draft.read().tout; draft.write().set_tout(on); } }
                }
                if kind == GameKind::Farbsolo {
                    Chip {
                        label: "Sie".to_string(),
                        on: d.sie,
                        disabled: !d.tout,
                        onclick: move |_| { let on = !draft.read().sie; draft.write().set_sie(on); },
                    }
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
                if d.sie {
                    p { class: "hint", "Beim Sie hat der Spieler immer alle 8 Laufende, Schneider und Schwarz zählen nicht zum Preis. Der Preis ist sonst wie beim Tout." }
                } else if d.tout {
                    p { class: "hint", "Beim Tout zählen Schneider und Schwarz nicht zum Preis, der Spieler muss ohnehin alle Stiche machen. Sie werden trotzdem eingetragen." }
                }
                if !d.sie {
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
                        disabled: d.laufende >= kind.max_laufende(d.gegenlauf),
                        onclick: move |_| { let n = draft.read().laufende; draft.write().set_laufende(n + 1); },
                        "+"
                    }
                }
                div { class: "chips",
                    Chip {
                        label: "Gegenlauf (die Gegner hatten sie)".to_string(),
                        on: d.gegenlauf,
                        disabled: d.laufende == 0,
                        onclick: move |_| draft.write().toggle_gegenlauf(),
                    }
                }
                }
            }
        }

        if d.bock_enabled || d.bock || d.starts_bock {
            section {
                h3 { "Bock" }
                div { class: "chips",
                    Chip {
                        label: "Bockspiel (doppelter Preis)".to_string(),
                        on: d.bock,
                        onclick: move |_| draft.write().toggle_bock(),
                    }
                    Chip {
                        label: "Bockrunde entstanden".to_string(),
                        on: d.starts_bock,
                        onclick: move |_| draft.write().toggle_starts_bock(),
                    }
                }
                if d.bock && (stacked || d.bock_extra > 0) {
                    p { class: "hint", "Mehrere Bockrunden laufen zugleich: Wie oft wird der Preis gedoppelt?" }
                    div { class: "chips",
                        for n in 0..MAX_BOCK {
                            Chip {
                                key: "{n}",
                                label: format!("mal {}", 1u32 << (n + 1)),
                                on: d.bock_extra == n,
                                onclick: move |_| draft.write().set_bock_extra(n),
                            }
                        }
                    }
                }
                if d.starts_bock {
                    p { class: "hint", "Wie viele Bockrunden sind durch dieses Spiel entstanden?" }
                    div { class: "chips",
                        for n in 0..MAX_BOCK {
                            Chip {
                                key: "{n}",
                                label: format!("{}", n + 1),
                                on: d.extra_bock_rounds == n,
                                onclick: move |_| draft.write().set_extra_bock_rounds(n),
                            }
                        }
                    }
                }
                p { class: "hint", "{bock_hint}" }
            }
        }

        section {
            h3 { "Preis" }
            if let Some(Ok(game)) = &preview {
                p { class: "price", "Berechnet: " b { "{format_euro(game.calculated_value)}" } }
            }
            if let Some(Ok(game)) = &preview {
                if kind == GameKind::Ramsch
                    && !d.knockers.is_empty()
                    && game.tariff.as_ref().is_some_and(|t| !t.ramsch_knock_doubles)
                {
                    p { class: "hint", "Das Klopfen ist erfasst, ändert beim Ramsch laut Tarif den Preis aber nicht." }
                }
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
            if price_corrected {
                p { class: "hint", "Der verlangte Preis weicht vom berechneten ab. Wer hat falsch verlangt? Es verlangen nur die Gewinner, beim Sauspiel beide." }
                div { class: "chips",
                    for seat in d.winners() {
                        Chip {
                            key: "{seat}",
                            label: players[seat].clone(),
                            on: d.demanded_by().contains(&seat),
                            onclick: move |_| draft.write().toggle_demander(seat),
                        }
                    }
                }
                p { class: "hint", "Was ging beim Verlangen schief? (Mehrfachauswahl)" }
                div { class: "chips",
                    for mistake in mistake_choices.clone() {
                        Chip {
                            key: "{mistake.label()}",
                            label: mistake.label().to_string(),
                            on: d.mistakes.contains(&mistake),
                            onclick: move |_| draft.write().toggle_mistake(mistake),
                        }
                    }
                }
            }
            if price_too_high {
                p { class: "hint", "Es wurde zu viel verlangt. Ist es den Verlierern aufgefallen?" }
                div { class: "chips",
                    Chip {
                        label: "Nicht aufgefallen (verlangter Preis gilt)".to_string(),
                        on: !d.reclaimed,
                        onclick: move |_| draft.write().set_reclaimed(false),
                    }
                    Chip {
                        label: "Zurückverlangt (das Doppelte des Mehrbetrags)".to_string(),
                        on: d.reclaimed,
                        onclick: move |_| draft.write().set_reclaimed(true),
                    }
                }
                if d.reclaimed {
                    if let Some(Ok(game)) = &preview {
                        {
                            let extra = game.input.price_override.unwrap_or(0) - game.calculated_value;
                            rsx! {
                                p { class: "hint",
                                    "Zu viel verlangt: {format_euro(extra)} je Gegner. Die Verlierer verlangen das Doppelte zurück ({format_euro(2 * extra)}), der Preis sinkt damit auf {format_euro(game.value)}."
                                }
                            }
                        }
                    }
                }
                if d.reclaimed {
                    p { class: "hint", "Wer hat zurückverlangt?" }
                    div { class: "chips",
                        for seat in d.losers() {
                            Chip {
                                key: "{seat}",
                                label: players[seat].clone(),
                                on: d.reclaimed_by().contains(&seat),
                                onclick: move |_| draft.write().toggle_reclaimer(seat),
                            }
                        }
                    }
                }
            }
            if let Some(Ok(game)) = &preview {
                if let Some(t) = table.read().as_ref() {
                    dl { class: "check",
                        for (label, value) in check_rows(t, game) {
                            div { key: "{label}",
                                dt { "{label}" }
                                dd { "{value}" }
                            }
                        }
                    }
                }
                ul { class: "payouts",
                    for seat in d.participants.clone() {
                        li {
                            key: "{seat}",
                            class: if seat == game.input.declarer { "role-declarer" } else if game.input.partner == Some(seat) { "role-partner" } else { "" },
                            span {
                                "{players[seat]}"
                                if seat == game.input.declarer {
                                    b { class: "role", if kind == GameKind::Ramsch { "Verlierer" } else { "Spieler" } }
                                } else if game.input.partner == Some(seat) {
                                    b { class: "role", "Mitspieler" }
                                }
                            }
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
