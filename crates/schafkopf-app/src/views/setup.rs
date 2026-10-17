//! Neuer Abend: Spieler, erster Geber, Regelwerk und Tarif festlegen.

use dioxus::prelude::*;
use schafkopf_core::{GameKind, Library, RuleSet, Table};

use super::common::{persist_library, Chip, Notice, TariffFields};
use crate::draft::TariffDraft;
use crate::{now_secs, storage, Screen};

const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 10;

/// Ein Platz am Tisch bei der Einrichtung: ein Name, der aus der Liste der gespeicherten Spieler
/// gewählt oder neu getippt wird.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlayerSlot {
    pub name: String,
    /// `true`: Der Name wird getippt, `false`: Er wird aus der Liste gewählt.
    pub typing: bool,
}

/// Die Plätze bei der Einrichtung: die Spieler des letzten Abends, falls es einen gibt, sonst vier
/// leere. Gespeicherte Namen werden aus der Liste gewählt, alles andere getippt. Ohne gespeicherte
/// Spieler gibt es nichts zu wählen, dann wird getippt.
pub fn initial_slots(library: &Library, previous: &[String]) -> Vec<PlayerSlot> {
    let known = library.players();
    let slot = |name: &str| PlayerSlot {
        name: name.to_owned(),
        typing: known.is_empty() || (!name.is_empty() && !known.iter().any(|k| k == name)),
    };
    if previous.is_empty() {
        vec![slot(""); 4]
    } else {
        previous.iter().map(|n| slot(n)).collect()
    }
}

/// Alles, was beim Einrichten eines Abends eingegeben wird. Es liegt auf App-Ebene, damit nichts
/// verloren geht, wenn man zwischendurch die Verwaltung der Regelwerke und Tarife öffnet.
#[derive(Clone, Copy, PartialEq)]
pub struct SetupState {
    pub names: Signal<Vec<PlayerSlot>>,
    pub first_dealer: Signal<usize>,
    pub rule_set: Signal<String>,
    pub tariff_name: Signal<String>,
    pub tariff: Signal<TariffDraft>,
    /// Ein Testabend, der nicht in die Gesamtstatistik eingeht.
    pub debug: Signal<bool>,
}

/// Legt den Zustand der Einrichtung an, vorbelegt mit den Standards der Bibliothek und den
/// Spielern des laufenden Abends.
pub fn use_setup_state(library: Signal<Library>, table: Signal<Option<Table>>) -> SetupState {
    let lib = library.peek();
    let rule_set = lib.default_rule_set().name.clone();
    let tariff = lib.default_tariff().clone();
    let previous: Vec<String> = table
        .peek()
        .as_ref()
        .map(|t| t.players.clone())
        .unwrap_or_default();
    let slots = initial_slots(&lib, &previous);
    drop(lib);
    SetupState {
        names: use_signal(|| slots),
        first_dealer: use_signal(|| 0),
        rule_set: use_signal(|| rule_set),
        tariff_name: use_signal(|| tariff.name.clone()),
        tariff: use_signal(|| TariffDraft::from_tariff(&tariff.tariff)),
        debug: use_signal(|| false),
    }
}

/// Wert der Auswahl-Zeile, mit der man einen neuen Namen tippt.
const NEW_PLAYER: &str = "\u{1}neu";

/// Ein Platz am Tisch: Auswahl aus den gespeicherten Spielern oder Eingabefeld, dazu der Knopf zum
/// Entfernen.
#[component]
fn PlayerSlotRow(
    slots: Signal<Vec<PlayerSlot>>,
    first_dealer: Signal<usize>,
    library: Signal<Library>,
    index: usize,
) -> Element {
    let mut slots = slots;
    let mut first_dealer = first_dealer;
    let (slot, taken, count) = {
        let list = slots.read();
        // Beim Entfernen kann dieser Platz kurz vor dem Aufräumen noch einmal gezeichnet werden.
        let Some(slot) = list.get(index).cloned() else {
            return rsx! {};
        };
        let taken: Vec<String> = list
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != index)
            .map(|(_, s)| s.name.trim().to_lowercase())
            .collect();
        (slot, taken, list.len())
    };
    let known = library.read().players_sorted();
    // Ein Name, den es in der Liste nicht (mehr) gibt, wird als getippter Name gezeigt.
    let typing = slot.typing || (!slot.name.is_empty() && !known.contains(&slot.name));

    rsx! {
        div { class: "row",
            label { class: "field",
                span { "Spieler {index + 1}" }
                if typing {
                    input {
                        r#type: "text",
                        autocomplete: "off",
                        placeholder: "Neuer Name",
                        value: "{slot.name}",
                        oninput: move |e| {
                            if let Some(s) = slots.write().get_mut(index) {
                                s.name = e.value();
                            }
                        },
                    }
                } else {
                    select {
                        onchange: move |e| {
                            let value = e.value();
                            if let Some(s) = slots.write().get_mut(index) {
                                if value == NEW_PLAYER {
                                    *s = PlayerSlot { name: String::new(), typing: true };
                                } else {
                                    s.name = value;
                                }
                            }
                        },
                        option { value: "", selected: slot.name.is_empty(), "Spieler wählen" }
                        for name in known.iter().cloned() {
                            option {
                                key: "{name}",
                                value: "{name}",
                                selected: name == slot.name,
                                disabled: taken.contains(&name.to_lowercase()),
                                "{name}"
                            }
                        }
                        option { value: NEW_PLAYER, "+ Neuer Spieler …" }
                    }
                }
            }
            if typing && !known.is_empty() {
                button {
                    class: "secondary icon",
                    r#type: "button",
                    "aria-label": "Aus der Liste wählen",
                    onclick: move |_| {
                        if let Some(s) = slots.write().get_mut(index) {
                            *s = PlayerSlot::default();
                        }
                    },
                    "▾"
                }
            }
            button {
                class: "secondary icon",
                r#type: "button",
                "aria-label": "Spieler entfernen",
                disabled: count <= MIN_PLAYERS,
                onclick: move |_| {
                    slots.write().remove(index);
                    if first_dealer() >= slots.read().len() {
                        first_dealer.set(0);
                    }
                },
                "−"
            }
        }
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
        mut debug,
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
        let players: Vec<String> = names
            .read()
            .iter()
            .map(|slot| slot.name.trim().to_owned())
            .collect();
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
        if let Err(e) = new_table.use_rule_set(&rules) {
            return notice.set(Some(Notice::error(e.to_string())));
        }
        new_table.use_tariff(Some(tariff_label.clone()), tariff);
        let now = now_secs();
        new_table.created_at = Some(now as i64);
        new_table.debug = debug();
        new_table.utc_offset_min = crate::utc_offset_min();
        // Wurde der alte Abend nicht beendet, endet er mit seiner letzten bekannten Aktivität.
        if let Some(previous) = table.write().as_mut() {
            if previous.ended_at.is_none() {
                if let Some(last) = previous.last_activity() {
                    previous.finish(last);
                    let _ = storage::save(previous, None);
                }
            }
        }
        // Ein Abend ohne ein einziges Spiel ist nichts, was man aufheben müsste.
        let previous_is_empty = table.read().as_ref().is_some_and(|t| t.games().is_empty());
        let archived = if previous_is_empty {
            storage::discard_current()
        } else {
            storage::archive_current(now).map(|_| ())
        };
        if let Err(e) = archived {
            return notice.set(Some(Notice::error(format!(
                "Der alte Abend konnte nicht archiviert werden: {e}"
            ))));
        }
        if let Err(e) = storage::save(&new_table, None) {
            return notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}"
            ))));
        }
        // Namen, die zum ersten Mal getippt wurden, stehen beim nächsten Mal zur Auswahl.
        let added = library.write().ensure_players(&new_table.players);
        // Jetzt stehen sie in der Liste: Beim nächsten Abend sind sie Auswahl statt Eingabe.
        for slot in names.write().iter_mut() {
            slot.name = slot.name.trim().to_owned();
            slot.typing = !library.read().players().contains(&slot.name);
        }
        let library_problem = if added > 0 {
            persist_library(&library.read()).err()
        } else {
            None
        };
        table.set(Some(new_table));
        screen.set(Screen::Table);
        notice.set(Some(match library_problem {
            Some(e) => Notice::error(format!(
                "Abend gestartet, aber die neuen Spieler konnten nicht gemerkt werden: {e}"
            )),
            None if added == 1 => Notice::ok("Abend gestartet, 1 neuer Spieler gemerkt"),
            None if added > 1 => {
                Notice::ok(format!("Abend gestartet, {added} neue Spieler gemerkt"))
            }
            None => Notice::ok("Abend gestartet"),
        }));
    };

    rsx! {
        h2 { "Neuer Abend" }
        section {
            h3 { "Spieler in Sitzreihenfolge" }
            p { class: "hint", "Im Uhrzeigersinn, so wie ihr sitzt. Zwei bis zehn Spieler." }
            for i in 0..count {
                PlayerSlotRow { key: "{i}", slots: names, first_dealer, library, index: i }
            }
            button {
                class: "secondary",
                r#type: "button",
                disabled: count >= MAX_PLAYERS,
                onclick: move |_| {
                    let typing = library.read().players().is_empty();
                    names.write().push(PlayerSlot { name: String::new(), typing });
                },
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
                for (i, slot) in names.read().iter().enumerate() {
                    option {
                        key: "{i}",
                        value: "{i}",
                        selected: first_dealer() == i,
                        if slot.name.trim().is_empty() { "Spieler {i + 1}" } else { "{slot.name}" }
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
            p { class: "hint", "Gespielt wird: {kinds_text(&selected_rules.kinds)}. Bock: {selected_rules.bock.label()}." }
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
        div { class: "chips",
            Chip {
                label: "Testabend (nicht in der Gesamtstatistik)".to_string(),
                on: debug(),
                onclick: move |_| debug.set(!debug()),
            }
        }
        p { class: "hint", "Spieler, Regelwerke und Tarife legst du in der Verwaltung an und änderst sie dort, bei Regelwerken und Tarifen auch den Standard." }
        div { class: "actions stack",
            button {
                class: "secondary",
                r#type: "button",
                onclick: move |_| screen.set(Screen::Library { from_setup: true }),
                "Spieler, Regelwerke und Tarife verwalten"
            }
            button {
                class: "secondary",
                r#type: "button",
                onclick: move |_| screen.set(Screen::PastList),
                "Frühere Abende ansehen"
            }
        }
        div { class: "actions",
            button { class: "primary big", r#type: "button", onclick: start, "Abend starten" }
            if running {
                button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "Abbrechen" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn library_with(players: &[&str]) -> Library {
        let mut library = Library::with_defaults();
        library.ensure_players(&players.iter().map(|p| p.to_string()).collect::<Vec<_>>());
        library
    }

    fn slot(name: &str, typing: bool) -> PlayerSlot {
        PlayerSlot {
            name: name.to_owned(),
            typing,
        }
    }

    #[test]
    fn first_start_without_saved_players_means_typing() {
        let slots = initial_slots(&library_with(&[]), &[]);
        assert_eq!(slots, vec![slot("", true); 4]);
    }

    #[test]
    fn with_saved_players_empty_places_are_choices() {
        let slots = initial_slots(&library_with(&["Anna", "Ben"]), &[]);
        assert_eq!(slots, vec![slot("", false); 4]);
    }

    #[test]
    fn the_last_evening_prefills_the_places() {
        let library = library_with(&["Anna", "Ben", "Cem"]);
        let previous = ["Anna", "Ben", "Gast", "Cem", "Dora"]
            .map(String::from)
            .to_vec();
        let slots = initial_slots(&library, &previous);
        assert_eq!(slots.len(), 5, "so viele Plätze wie am letzten Abend");
        assert_eq!(slots[0], slot("Anna", false));
        assert_eq!(
            slots[2],
            slot("Gast", true),
            "unbekannte Namen bleiben als getippt stehen"
        );
        assert_eq!(slots[4], slot("Dora", true));
    }

    #[test]
    fn without_a_library_every_previous_name_is_typed() {
        let previous = ["A", "B"].map(String::from).to_vec();
        assert_eq!(
            initial_slots(&library_with(&[]), &previous),
            vec![slot("A", true), slot("B", true)]
        );
    }
}
