//! Kleine Bausteine, die mehrere Bildschirme brauchen.

use dioxus::prelude::*;

use crate::draft::TariffDraft;

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
) -> Element {
    rsx! {
        button {
            class: if on { "chip on" } else { "chip" },
            r#type: "button",
            disabled: disabled,
            "aria-pressed": "{on}",
            onclick: move |_| onclick.call(()),
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
    }
}

/// Speichert die Bibliothek und liefert bei einem Fehler einen anzeigbaren Text.
pub fn persist_library(library: &schafkopf_core::Library) -> Result<(), String> {
    crate::storage::save_library(library)
        .map(|_| ())
        .map_err(|e| format!("Die Bibliothek konnte nicht gespeichert werden: {e}"))
}
