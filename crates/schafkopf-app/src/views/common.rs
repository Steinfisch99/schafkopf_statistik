//! Kleine Bausteine, die mehrere Bildschirme brauchen.

use dioxus::prelude::*;

use super::suit_icon::SuitIcon;
use crate::draft::TariffDraft;
use schafkopf_core::Suit;

/// Rückmeldung an die Nutzerin oder den Nutzer, oben auf jedem Bildschirm.
#[derive(Clone, PartialEq)]
pub struct Notice {
    pub text: String,
    pub error: bool,
}

impl Notice {
    pub fn ok(text: impl Into<String>) -> Self {
        Notice {
            text: text.into(),
            error: false,
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Notice {
            text: text.into(),
            error: true,
        }
    }
}

#[component]
pub fn NoticeBar(notice: Signal<Option<Notice>>) -> Element {
    let mut notice = notice;
    match notice() {
        Some(n) => rsx! {
            div { class: if n.error { "notice error" } else { "notice" }, role: "status",
                span { "{n.text}" }
                button { class: "link", onclick: move |_| notice.set(None), "OK" }
            }
        },
        None => rsx! {},
    }
}

/// Umschaltknopf: an oder aus.
#[component]
pub fn Chip(
    label: String,
    on: bool,
    onclick: EventHandler<()>,
    #[props(default = false)] disabled: bool,
    /// Ein Symbol vor dem Text, zum Beispiel bei der Wahl einer Farbe.
    #[props(default = None)]
    icon: Option<Suit>,
) -> Element {
    rsx! {
        button {
            class: if on { "chip on" } else { "chip" },
            r#type: "button",
            disabled: disabled,
            "aria-pressed": "{on}",
            onclick: move |_| onclick.call(()),
            if let Some(suit) = icon {
                SuitIcon { suit }
            }
            "{label}"
        }
    }
}

/// Knopf, der erst nach einer zweiten Bestätigung etwas auslöst.
#[component]
pub fn ConfirmButton(label: String, confirm_label: String, onconfirm: EventHandler<()>) -> Element {
    let mut armed = use_signal(|| false);
    // Beide Zustände füllen dieselbe Zeile, damit beim Antippen nichts verrutscht.
    rsx! {
        div { class: "confirm",
            if armed() {
                button {
                    class: "danger",
                    r#type: "button",
                    onclick: move |_| {
                        armed.set(false);
                        onconfirm.call(());
                    },
                    "{confirm_label}"
                }
                button { class: "secondary", r#type: "button", onclick: move |_| armed.set(false), "Abbrechen" }
            } else {
                button { class: "secondary", r#type: "button", onclick: move |_| armed.set(true), "{label}" }
            }
        }
    }
}

#[component]
pub fn TextField(
    label: String,
    value: String,
    oninput: EventHandler<String>,
    #[props(default = String::new())] hint: String,
    #[props(default = "text".to_string())] mode: String,
) -> Element {
    rsx! {
        label { class: "field",
            span { "{label}" }
            input {
                r#type: "text",
                inputmode: "{mode}",
                autocomplete: "off",
                value: "{value}",
                placeholder: "{hint}",
                oninput: move |e| oninput.call(e.value()),
            }
        }
    }
}

/// Eingabefelder für den Tarif, gemeinsam für Einrichtung und spätere Änderung.
#[component]
pub fn TariffFields(draft: Signal<TariffDraft>) -> Element {
    let mut draft = draft;
    let d = draft();
    rsx! {
        div { class: "grid2",
            TextField { label: "Sauspiel (€)", value: d.sauspiel, mode: "decimal", oninput: move |v| draft.write().sauspiel = v }
            TextField { label: "Solo (€)", value: d.solo, mode: "decimal", oninput: move |v| draft.write().solo = v }
            TextField { label: "Bettel (€)", value: d.bettel, mode: "decimal", oninput: move |v| draft.write().bettel = v }
            TextField { label: "Ramsch (€)", value: d.ramsch, mode: "decimal", oninput: move |v| draft.write().ramsch = v }
            TextField { label: "Schneider (€)", value: d.schneider, mode: "decimal", oninput: move |v| draft.write().schneider = v }
            TextField { label: "Schwarz (€)", value: d.schwarz, mode: "decimal", oninput: move |v| draft.write().schwarz = v }
            TextField { label: "Laufende je (€)", value: d.laufende, mode: "decimal", oninput: move |v| draft.write().laufende = v }
        }
        p { class: "hint", "Laufende zählen ab dieser Anzahl:" }
        div { class: "grid3",
            TextField { label: "Sauspiel", value: d.min_sauspiel, mode: "numeric", oninput: move |v| draft.write().min_sauspiel = v }
            TextField { label: "Wenz, Geier", value: d.min_wenz_geier, mode: "numeric", oninput: move |v| draft.write().min_wenz_geier = v }
            TextField { label: "Solo", value: d.min_solo, mode: "numeric", oninput: move |v| draft.write().min_solo = v }
        }
        p { class: "hint", "Ramsch:" }
        div { class: "chips",
            Chip {
                label: "Klopfen verdoppelt den Preis".to_string(),
                on: d.ramsch_knock_doubles,
                onclick: move |_| {
                    let on = !draft.read().ramsch_knock_doubles;
                    draft.write().ramsch_knock_doubles = on;
                },
            }
            Chip {
                label: "Bock verdoppelt den Preis".to_string(),
                on: d.ramsch_bock_doubles,
                onclick: move |_| {
                    let on = !draft.read().ramsch_bock_doubles;
                    draft.write().ramsch_bock_doubles = on;
                },
            }
        }
        p { class: "hint", "Aus: Klopfen und Bock beim Ramsch werden trotzdem eingetragen und in der Statistik gezählt, ändern den Preis aber nicht." }
        p { class: "hint", "Falsch verlangen:" }
        div { class: "chips",
            Chip {
                label: "Zu viel Verlangtes darf zurückverlangt werden".to_string(),
                on: d.reclaim_enabled,
                onclick: move |_| {
                    let on = !draft.read().reclaim_enabled;
                    draft.write().reclaim_enabled = on;
                },
            }
        }
        p { class: "hint", "An: Verlangen die Gewinner zu viel und fällt es den Verlierern auf, tragen sie beim Spiel \"zurückverlangt\" ein, dann gilt der richtige Preis. Aus: Der verlangte Preis gilt immer." }
        p { class: "hint", "Strafzahlungen:" }
        div { class: "chips",
            Chip {
                label: "Vergehen und Neumischen kosten eine Strafe".to_string(),
                on: d.penalties_enabled,
                onclick: move |_| {
                    let on = !draft.read().penalties_enabled;
                    draft.write().penalties_enabled = on;
                },
            }
        }
        if d.penalties_enabled {
            div { class: "grid2",
                TextField { label: "Vergehen je Mitspieler (€)", value: d.offence_penalty, mode: "decimal", oninput: move |v| draft.write().offence_penalty = v }
                TextField { label: "Neumischen je Mitspieler (€)", value: d.misdeal_penalty, mode: "decimal", oninput: move |v| draft.write().misdeal_penalty = v }
            }
            p { class: "hint", "Der Täter zahlt die Strafe an jeden Mitspieler der laufenden Runde (wer aussetzt, bekommt nichts). Beim Neumischen (Fehler beim Geben) ist es der Geber, 0 heißt: Neumischen kostet nichts." }
        } else {
            p { class: "hint", "Aus: Vergehen und Neumischen werden trotzdem eingetragen und gezählt, kosten aber nichts." }
        }
    }
}

/// Speichert die Bibliothek und liefert bei einem Fehler einen anzeigbaren Text.
pub fn persist_library(library: &schafkopf_core::Library) -> Result<(), String> {
    crate::storage::save_library(library)
        .map(|_| ())
        .map_err(|e| format!("Die Bibliothek konnte nicht gespeichert werden: {e}"))
}

/// Versionsnummer der App (kommt aus `Cargo.toml`, `[workspace.package]`).
#[component]
pub fn AppVersion() -> Element {
    rsx! {
        p { class: "muted app-version", "Version {env!(\"CARGO_PKG_VERSION\")}" }
    }
}
