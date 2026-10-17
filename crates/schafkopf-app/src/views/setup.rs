//! Neuer Abend: Spieler, erster Geber, Regelwerk und Tarif festlegen.

use dioxus::prelude::*;
use schafkopf_core::{GameKind, Library, RuleSet, Table};

use super::common::{Notice, TariffFields, TextField};
use crate::draft::TariffDraft;
use crate::{now_secs, storage, Screen};

const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 10;

/// Alles, was beim Einrichten eines Abends eingegeben wird. Es liegt auf App-Ebene, damit nichts
/// verloren geht, wenn man zwischendurch die Verwaltung der Regelwerke und Tarife öffnet.
#[derive(Clone, Copy, PartialEq)]
pub struct SetupState {
    pub names: Signal<Vec<String>>,
    pub first_dealer: Signal<usize>,
    pub rule_set: Signal<String>,
    pub tariff_name: Signal<String>,
    pub tariff: Signal<TariffDraft>,
}

/// Legt den Zustand der Einrichtung an, vorbelegt mit den Standards der Bibliothek.
pub fn use_setup_state(library: Signal<Library>) -> SetupState {
    let lib = library.peek();
    let rule_set = lib.default_rule_set().name.clone();
    let tariff = lib.default_tariff().clone();
    drop(lib);
    SetupState {
        names: use_signal(|| vec![String::new(); 4]),
        first_dealer: use_signal(|| 0),
        rule_set: use_signal(|| rule_set),
        tariff_name: use_signal(|| tariff.name.clone()),
        tariff: use_signal(|| TariffDraft::from_tariff(&tariff.tariff)),
    }
}

/// "Sauspiel, Wenz, Geier": die Spielarten eines Regelwerks zum Anzeigen.
pub fn kinds_text(kinds: &[GameKind]) -> String {
    let labels: Vec<&str> = kinds.iter().map(|k| k.label()).collect();
    labels.join(", ")
}

/// Anzeigename des Tarifs eines Abends: der Name aus der Bibliothek, mit "(angepasst)", wenn die
/// Beträge davon abweichen, sonst "Eigener Tarif".
pub fn tariff_label(library: &Library, chosen: &str, draft: &TariffDraft) -> String {
    match library.tariff(chosen) {
        Some(named) if TariffDraft::from_tariff(&named.tariff) == *draft => named.name.clone(),
        Some(named) => format!("{} (angepasst)", named.name),
        None => "Eigener Tarif".to_string(),
    }
}

/// Das gewählte Regelwerk; gibt es den Namen nicht mehr, das Standard-Regelwerk.
fn resolve_rule_set(library: &Library, name: &str) -> RuleSet {
    library
        .rule_set(name)
        .unwrap_or_else(|| library.default_rule_set())
        .clone()
}

#[component]
pub fn Setup(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
    library: Signal<Library>,
    state: SetupState,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;
    let SetupState {
        mut names,
        mut first_dealer,
        mut rule_set,
        mut tariff_name,
        tariff,
    } = state;
    let mut tariff_signal = tariff;
    let running = table.read().is_some();
    let count = names.read().len();

    let lib = library.read();
    let selected_rules = resolve_rule_set(&lib, &rule_set.read());
    let rule_names: Vec<String> = lib.rule_sets().iter().map(|r| r.name.clone()).collect();
    let tariff_names: Vec<String> = lib.tariffs().iter().map(|t| t.name.clone()).collect();
    let tariff_label = tariff_label(&lib, &tariff_name.read(), &tariff_signal.read());
    drop(lib);

    let start = move |_| {
        let players: Vec<String> = names.read().iter().map(|n| n.trim().to_owned()).collect();
        let rules = resolve_rule_set(&library.read(), &rule_set.read());
        let built = tariff_signal.read().to_tariff().and_then(|t| {
            Table::new("Abend", players, first_dealer(), t.clone())
                .map(|table| (table, t))
                .map_err(|e| e.to_string())
        });
        let (mut new_table, tariff) = match built {
            Ok(built) => built,
            Err(e) => return notice.set(Some(Notice::error(e))),
        };
        if let Err(e) = new_table.use_rules(Some(rules.name.clone()), &rules.kinds) {
            return notice.set(Some(Notice::error(e.to_string())));
        }
        new_table.use_tariff(Some(tariff_label.clone()), tariff);
        let now = now_secs();
        new_table.created_at = Some(now as i64);
        if let Err(e) = storage::archive_current(now) {
            return notice.set(Some(Notice::error(format!(
                "Der alte Abend konnte nicht archiviert werden: {e}"
            ))));
        }
        if let Err(e) = storage::save(&new_table, None) {
            return notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}"
            ))));
        }
        table.set(Some(new_table));
        screen.set(Screen::Table);
        notice.set(Some(Notice::ok("Abend gestartet")));
    };

    rsx! {
        h2 { "Neuer Abend" }
        section {
            h3 { "Spieler in Sitzreihenfolge" }
            p { class: "hint", "Im Uhrzeigersinn, so wie ihr sitzt. Zwei bis zehn Spieler." }
            for i in 0..count {
                div { class: "row", key: "{i}",
                    TextField {
                        label: format!("Spieler {}", i + 1),
                        value: names.read()[i].clone(),
                        hint: "Name".to_string(),
                        oninput: move |v| names.write()[i] = v,
                    }
                    button {
                        class: "secondary icon",
                        r#type: "button",
                        "aria-label": "Spieler entfernen",
                        disabled: count <= MIN_PLAYERS,
                        onclick: move |_| {
                            names.write().remove(i);
                            if first_dealer() >= names.read().len() {
                                first_dealer.set(0);
                            }
                        },
                        "−"
                    }
                }
            }
            button {
                class: "secondary",
                r#type: "button",
                disabled: count >= MAX_PLAYERS,
                onclick: move |_| names.write().push(String::new()),
                "+ Spieler"
            }
        }
        section {
            h3 { "Erster Geber" }
            select {
                aria_label: "Erster Geber",
                onchange: move |e| {
                    if let Ok(i) = e.value().parse::<usize>() {
                        first_dealer.set(i);
                    }
                },
                for (i, name) in names.read().iter().enumerate() {
                    option {
                        key: "{i}",
                        value: "{i}",
                        selected: first_dealer() == i,
                        if name.trim().is_empty() { "Spieler {i + 1}" } else { "{name}" }
                    }
                }
            }
        }
        section {
            h3 { "Regelwerk" }
            select {
                aria_label: "Regelwerk",
                onchange: move |e| rule_set.set(e.value()),
                for name in rule_names {
                    option { key: "{name}", value: "{name}", selected: name == selected_rules.name,
                        if library.read().is_default_rule_set(&name) { "{name} (Standard)" } else { "{name}" }
                    }
                }
            }
            p { class: "hint", "Gespielt wird: {kinds_text(&selected_rules.kinds)}" }
        }
        section {
            h3 { "Tarif" }
            select {
                aria_label: "Tarif",
                onchange: move |e| {
                    let name = e.value();
                    if let Some(named) = library.read().tariff(&name) {
                        tariff_signal.set(TariffDraft::from_tariff(&named.tariff));
                    }
                    tariff_name.set(name);
                },
                for name in tariff_names {
                    option { key: "{name}", value: "{name}", selected: name == *tariff_name.read(),
                        if library.read().is_default_tariff(&name) { "{name} (Standard)" } else { "{name}" }
                    }
                }
            }
            details {
                summary { "Für diesen Abend anpassen" }
                p { class: "hint", "Ändert nur diesen Abend, nicht den gespeicherten Tarif. Der Tarif lässt sich später jederzeit ändern, schon eingetragene Spiele bleiben dabei unverändert." }
                TariffFields { draft: tariff_signal }
            }
        }
        p { class: "hint", "Regelwerke und Tarife legst du in der Verwaltung an, ändern und wählst einen Standard." }
        button {
            class: "secondary",
            r#type: "button",
            onclick: move |_| screen.set(Screen::Library { from_setup: true }),
            "Regelwerke und Tarife verwalten"
        }
        div { class: "actions",
            button { class: "primary big", r#type: "button", onclick: start, "Abend starten" }
            if running {
                button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "Abbrechen" }
            }
        }
    }
}
