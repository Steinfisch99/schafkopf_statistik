//! Regeln und Tarif während des Abends ändern. Die Änderungen gelten ab dem nächsten Spiel.

use dioxus::prelude::*;
use schafkopf_core::{Library, Table, Tariff};

use super::common::{persist_library, Chip, Notice, TariffFields, TextField};
use super::setup::{kinds_text, tariff_label};
use crate::draft::TariffDraft;
use crate::{storage, Screen};

#[component]
pub fn SettingsScreen(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
    library: Signal<Library>,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;
    let mut library = library;
    let mut draft = use_signal(|| {
        let tariff = table
            .read()
            .as_ref()
            .map_or_else(Tariff::default, |t| t.tariff.clone());
        TariffDraft::from_tariff(&tariff)
    });
    // Welcher gespeicherte Tarif als Vorlage gewählt ist; leer: der Tarif des Abends.
    let mut chosen = use_signal(String::new);
    let mut save_name = use_signal(String::new);

    let is_debug = table.read().as_ref().is_some_and(|t| t.debug);
    let (current_rules, current_tariff, enabled, bock_mode) = {
        let guard = table.read();
        let Some(t) = guard.as_ref() else {
            return rsx! {};
        };
        (
            t.rule_set_name.clone().unwrap_or_default(),
            t.tariff_name
                .clone()
                .unwrap_or_else(|| "Eigener Tarif".into()),
            t.enabled_kinds.clone(),
            t.bock_mode,
        )
    };
    let lib = library.read();
    let rule_names: Vec<String> = lib.rule_sets().iter().map(|r| r.name.clone()).collect();
    let tariff_names: Vec<String> = lib.tariffs().iter().map(|t| t.name.clone()).collect();
    let rules_known = lib.rule_set(&current_rules).is_some();
    drop(lib);

    let mut apply_rules = move |name: String| {
        let Some(rules) = library.read().rule_set(&name).cloned() else {
            return;
        };
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        if let Err(e) = t.use_rule_set(&rules) {
            return notice.set(Some(Notice::error(e.to_string())));
        }
        let saved = storage::save(t, None);
        drop(guard);
        match saved {
            Ok(_) => notice.set(Some(Notice::ok(format!(
                "Regelwerk \"{}\" gilt ab dem nächsten Spiel",
                rules.name
            )))),
            Err(e) => notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}"
            )))),
        }
    };

    let mut apply_tariff = move |label: Option<String>| {
        let tariff = match draft.read().to_tariff() {
            Ok(t) => t,
            Err(e) => return notice.set(Some(Notice::error(e))),
        };
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        t.use_tariff(label, tariff);
        let saved = storage::save(t, None);
        drop(guard);
        match saved {
            Ok(_) => notice.set(Some(Notice::ok(
                "Tarif gespeichert, er gilt ab dem nächsten Spiel",
            ))),
            Err(e) => notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}"
            )))),
        }
        screen.set(Screen::Table);
    };

    let save_as_new = move |_| {
        let tariff = match draft.read().to_tariff() {
            Ok(t) => t,
            Err(e) => return notice.set(Some(Notice::error(e))),
        };
        let name = {
            let mut lib = library.write();
            match lib.save_tariff(None, &save_name.read(), tariff) {
                Ok(name) => name,
                Err(e) => return notice.set(Some(Notice::error(e.to_string()))),
            }
        };
        if let Err(e) = persist_library(&library.read()) {
            return notice.set(Some(Notice::error(e)));
        }
        chosen.set(name.clone());
        save_name.set(String::new());
        apply_tariff(Some(name));
    };

    rsx! {
        h2 { "Regeln und Tarif" }
        p { class: "hint", "Änderungen gelten ab dem nächsten Spiel. Schon eingetragene Spiele behalten ihre Beträge." }

        section {
            h3 { "Regelwerk" }
            select {
                aria_label: "Regelwerk",
                onchange: move |e| apply_rules(e.value()),
                if !rules_known {
                    option { value: "", selected: true, disabled: true, "Eigene Auswahl" }
                }
                for name in rule_names {
                    option { key: "{name}", value: "{name}", selected: name == current_rules, "{name}" }
                }
            }
            p { class: "hint", "Gespielt wird: {kinds_text(&enabled)}. Bock: {bock_mode.label()}." }
        }

        section {
            h3 { "Tarif" }
            p { class: "hint", "Jetzt: {current_tariff}" }
            label { class: "field",
                span { "Als Vorlage einen gespeicherten Tarif laden" }
                select {
                    onchange: move |e| {
                        let name = e.value();
                        if let Some(named) = library.read().tariff(&name) {
                            draft.set(TariffDraft::from_tariff(&named.tariff));
                        }
                        chosen.set(name);
                    },
                    option { value: "", selected: chosen.read().is_empty(), "Tarif dieses Abends" }
                    for name in tariff_names {
                        option { key: "{name}", value: "{name}", selected: name == *chosen.read(), "{name}" }
                    }
                }
            }
            TariffFields { draft }
            div { class: "actions",
                button {
                    class: "primary big",
                    r#type: "button",
                    onclick: move |_| {
                        let label = tariff_label(&library.read(), &chosen.read(), &draft.read());
                        apply_tariff(Some(label));
                    },
                    "Für diesen Abend übernehmen"
                }
            }
            h3 { "Als Tarif speichern" }
            p { class: "hint", "Legt die Beträge unter einem Namen in der Bibliothek ab, damit du sie an späteren Abenden wählen kannst, und übernimmt sie für diesen Abend." }
            div { class: "row",
                TextField {
                    label: "Name des Tarifs".to_string(),
                    value: save_name(),
                    hint: "zum Beispiel Kneipe".to_string(),
                    oninput: move |v| save_name.set(v),
                }
                button { class: "secondary", r#type: "button", style: "margin-bottom:0.6rem", onclick: save_as_new, "Speichern" }
            }
        }

        section {
            h3 { "Sicherung" }
            p { class: "hint", "Alle Daten als ZIP sichern, zum Beispiel für ein neues Handy, oder eine Sicherung einspielen." }
            div { class: "actions",
                button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Backup), "Sicherung öffnen" }
            }
        }

        section {
            h3 { "Testabend" }
            div { class: "chips",
                Chip {
                    label: "Testabend (nicht in der Gesamtstatistik)".to_string(),
                    on: is_debug,
                    onclick: move |_| {
                        let mut guard = table.write();
                        let Some(t) = guard.as_mut() else { return };
                        t.debug = !t.debug;
                        let saved = storage::save(t, None);
                        drop(guard);
                        if let Err(e) = saved {
                            notice.set(Some(Notice::error(format!("Speichern fehlgeschlagen: {e}"))));
                        }
                    },
                }
            }
            p { class: "hint", "Ein Testabend steht in den früheren Abenden, geht aber nicht in die Gesamtstatistik über mehrere Abende ein. Zum Ausprobieren." }
        }

        div { class: "actions",
            button {
                class: "secondary",
                r#type: "button",
                onclick: move |_| screen.set(Screen::Library { from_setup: false }),
                "Spieler, Regelwerke und Tarife verwalten"
            }
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "Zurück" }
        }
    }
}
