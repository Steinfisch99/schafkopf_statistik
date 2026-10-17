//! Verwaltung der gespeicherten Regelwerke (welche Spielarten) und Tarife (was es kostet).

use dioxus::prelude::*;
use schafkopf_core::{GameKind, Library, Tariff};

use super::common::{persist_library, Chip, ConfirmButton, Notice, TariffFields, TextField};
use crate::draft::TariffDraft;
use crate::Screen;

/// Meldung nach dem Speichern oder Löschen: Erfolg oder der Grund des Fehlers.
fn report(mut notice: Signal<Option<Notice>>, result: Result<String, String>) {
    notice.set(Some(match result {
        Ok(text) => Notice::ok(text),
        Err(text) => Notice::error(text),
    }));
}

/// Ändert die Bibliothek und speichert sie. Bei einem Fehler in der Änderung bleibt die Datei
/// unberührt, und die Bibliothek im Speicher wird zurückgesetzt.
fn change_library(
    mut library: Signal<Library>,
    change: impl FnOnce(&mut Library) -> Result<String, String>,
) -> Result<String, String> {
    let before = library.read().clone();
    let text = {
        let mut lib = library.write();
        change(&mut lib)
    };
    let text = match text {
        Ok(text) => text,
        Err(e) => {
            library.set(before);
            return Err(e);
        }
    };
    let saved = persist_library(&library.read());
    if let Err(e) = saved {
        library.set(before);
        return Err(e);
    }
    Ok(text)
}

#[component]
fn Badge() -> Element {
    rsx! { span { class: "badge", "Standard" } }
}

/// Ein Regelwerk anlegen (`original` ist `None`) oder ein bestehendes ändern.
#[component]
fn RuleSetEditor(
    library: Signal<Library>,
    notice: Signal<Option<Notice>>,
    original: Option<String>,
) -> Element {
    let (name0, kinds0, is_default, can_delete) = {
        let lib = library.read();
        match original.as_deref().and_then(|n| lib.rule_set(n)) {
            Some(r) => (
                r.name.clone(),
                r.kinds.clone(),
                lib.is_default_rule_set(&r.name),
                lib.rule_sets().len() > 1,
            ),
            None => (
                String::new(),
                lib.default_rule_set().kinds.clone(),
                false,
                false,
            ),
        }
    };
    let mut name = use_signal(|| name0.clone());
    let mut kinds = use_signal(|| kinds0.clone());
    let is_new = original.is_none();

    let save = {
        let original = original.clone();
        move |_| {
            let result = change_library(library, |lib| {
                lib.save_rule_set(original.as_deref(), &name.read(), &kinds.read())
                    .map(|saved| format!("Regelwerk \"{saved}\" gespeichert"))
                    .map_err(|e| e.to_string())
            });
            if result.is_ok() && original.is_none() {
                name.set(String::new());
            }
            report(notice, result);
        }
    };
    let make_default = {
        let original = original.clone();
        move |_| {
            let Some(own) = original.clone() else { return };
            let result = change_library(library, |lib| {
                lib.set_default_rule_set(&own)
                    .map(|()| format!("\"{own}\" ist jetzt das Standard-Regelwerk"))
                    .map_err(|e| e.to_string())
            });
            report(notice, result);
        }
    };
    let delete = {
        let original = original.clone();
        move |_| {
            let Some(own) = original.clone() else { return };
            let result = change_library(library, |lib| {
                lib.delete_rule_set(&own)
                    .map(|()| format!("Regelwerk \"{own}\" gelöscht"))
                    .map_err(|e| e.to_string())
            });
            report(notice, result);
        }
    };

    rsx! {
        details { class: "player-detail",
            summary {
                span { class: "name", if is_new { "+ Neues Regelwerk" } else { "{name0}" } }
                if is_default { Badge {} }
            }
            TextField {
                label: "Name".to_string(),
                value: name(),
                hint: "zum Beispiel Kneipe".to_string(),
                oninput: move |v| name.set(v),
            }
            p { class: "hint", "Diese Spielarten stehen im Formular zur Wahl:" }
            div { class: "chips",
                for kind in GameKind::ALL {
                    Chip {
                        key: "{kind.label()}",
                        label: kind.label().to_string(),
                        on: kinds.read().contains(&kind),
                        onclick: move |_| {
                            let mut list = kinds.write();
                            if list.contains(&kind) {
                                list.retain(|&k| k != kind);
                            } else {
                                list.push(kind);
                            }
                        },
                    }
                }
            }
            div { class: "actions",
                button { class: "primary", r#type: "button", onclick: save, if is_new { "Anlegen" } else { "Speichern" } }
                if !is_new && !is_default {
                    button { class: "secondary", r#type: "button", onclick: make_default, "Als Standard" }
                }
                if !is_new && can_delete {
                    ConfirmButton { label: "Löschen", confirm_label: "Ja, löschen", onconfirm: delete }
                }
            }
        }
    }
}

/// Einen Tarif anlegen (`original` ist `None`) oder einen bestehenden ändern.
#[component]
fn TariffEditor(
    library: Signal<Library>,
    notice: Signal<Option<Notice>>,
    original: Option<String>,
) -> Element {
    let (name0, tariff0, is_default, can_delete) = {
        let lib = library.read();
        match original.as_deref().and_then(|n| lib.tariff(n)) {
            Some(t) => (
                t.name.clone(),
                t.tariff.clone(),
                lib.is_default_tariff(&t.name),
                lib.tariffs().len() > 1,
            ),
            None => (
                String::new(),
                lib.default_tariff().tariff.clone(),
                false,
                false,
            ),
        }
    };
    let mut name = use_signal(|| name0.clone());
    let draft = use_signal(|| TariffDraft::from_tariff(&tariff0));
    let is_new = original.is_none();

    let save = {
        let original = original.clone();
        move |_| {
            let tariff: Tariff = match draft.read().to_tariff() {
                Ok(t) => t,
                Err(e) => return report(notice, Err(e)),
            };
            let result = change_library(library, |lib| {
                lib.save_tariff(original.as_deref(), &name.read(), tariff)
                    .map(|saved| format!("Tarif \"{saved}\" gespeichert"))
                    .map_err(|e| e.to_string())
            });
            if result.is_ok() && original.is_none() {
                name.set(String::new());
            }
            report(notice, result);
        }
    };
    let make_default = {
        let original = original.clone();
        move |_| {
            let Some(own) = original.clone() else { return };
            let result = change_library(library, |lib| {
                lib.set_default_tariff(&own)
                    .map(|()| format!("\"{own}\" ist jetzt der Standardtarif"))
                    .map_err(|e| e.to_string())
            });
            report(notice, result);
        }
    };
    let delete = {
        let original = original.clone();
        move |_| {
            let Some(own) = original.clone() else { return };
            let result = change_library(library, |lib| {
                lib.delete_tariff(&own)
                    .map(|()| format!("Tarif \"{own}\" gelöscht"))
                    .map_err(|e| e.to_string())
            });
            report(notice, result);
        }
    };

    rsx! {
        details { class: "player-detail",
            summary {
                span { class: "name", if is_new { "+ Neuer Tarif" } else { "{name0}" } }
                if is_default { Badge {} }
            }
            TextField {
                label: "Name".to_string(),
                value: name(),
                hint: "zum Beispiel Kneipe".to_string(),
                oninput: move |v| name.set(v),
            }
            TariffFields { draft }
            div { class: "actions",
                button { class: "primary", r#type: "button", onclick: save, if is_new { "Anlegen" } else { "Speichern" } }
                if !is_new && !is_default {
                    button { class: "secondary", r#type: "button", onclick: make_default, "Als Standard" }
                }
                if !is_new && can_delete {
                    ConfirmButton { label: "Löschen", confirm_label: "Ja, löschen", onconfirm: delete }
                }
            }
        }
    }
}

#[component]
pub fn LibraryScreen(
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
    library: Signal<Library>,
    from_setup: bool,
) -> Element {
    let mut screen = screen;
    let rule_sets: Vec<String> = library
        .read()
        .rule_sets()
        .iter()
        .map(|r| r.name.clone())
        .collect();
    let tariffs: Vec<String> = library
        .read()
        .tariffs()
        .iter()
        .map(|t| t.name.clone())
        .collect();
    let back = if from_setup {
        Screen::Setup
    } else {
        Screen::Settings
    };

    rsx! {
        h2 { "Regelwerke und Tarife" }
        p { class: "hint",
            "Hier legst du fest, was ihr spielt und was es kostet. Beim Start eines Abends wählst du aus, der Standard ist vorgewählt. Änderungen betreffen nur künftige Abende, ein laufender Abend behält seine Einstellungen."
        }

        section {
            h3 { "Regelwerke" }
            p { class: "hint", "Welche Spielarten stehen zur Wahl? Ein Spiel mit einer deaktivierten Spielart lässt sich nicht eintragen." }
            for name in rule_sets {
                RuleSetEditor { key: "{name}", library, notice, original: Some(name.clone()) }
            }
            RuleSetEditor { key: "neu", library, notice, original: None }
        }

        section {
            h3 { "Tarife" }
            for name in tariffs {
                TariffEditor { key: "{name}", library, notice, original: Some(name.clone()) }
            }
            TariffEditor { key: "neu", library, notice, original: None }
        }

        div { class: "actions",
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(back), "Zurück" }
        }
    }
}
