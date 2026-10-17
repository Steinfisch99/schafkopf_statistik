//! Schafkopf-Statistik: Abend einrichten, Spiele eintragen, Geldstand ansehen.

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod android;
mod draft;
mod export;
mod gestures;
mod navigation;
mod stats_text;
mod storage;
mod summary;
mod views;

use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::document::eval;
use dioxus::prelude::*;
use navigation::{back_target, effective_screen, history_action, HistoryAction};
use views::{
    use_setup_state, ExportScreen, GameForm, LibraryScreen, Notice, NoticeBar, PastEveningView,
    PastList, PastSection, SeatsScreen, SettingsScreen, Setup, StatsScreen, TableView,
};

const CSS: &str = include_str!("style.css");

/// Der Bildschirm, der gerade zu sehen ist.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Setup,
    Table,
    Game,
    /// Ein früheres Spiel bearbeiten, gezählt ab 0.
    EditGame(usize),
    /// Regeln und Tarif des laufenden Abends.
    Settings,
    /// Wer am Tisch sitzt: Spieler dazusetzen, gehen lassen, zurückholen.
    Seats,
    /// Verwaltung der gespeicherten Regelwerke und Tarife. `from_setup` sagt, wohin "Zurück" führt.
    Library {
        from_setup: bool,
    },
    Export,
    /// Die Statistik des laufenden Abends, auf Wunsch mit der Karte eines Spielers geöffnet.
    Stats {
        player: Option<usize>,
    },
    /// Die Liste der früheren Abende.
    PastList,
    /// Ein früherer Abend, bestimmt durch den Zeitpunkt, an dem er ins Archiv kam.
    Past(u64),
    PastStats {
        stamp: u64,
        player: Option<usize>,
    },
    PastExport(u64),
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let table = use_signal(storage::load);
    let library = use_signal(storage::load_library);
    let setup = use_setup_state(library, table);
    let mut screen = use_signal(|| {
        if table.peek().is_some() {
            Screen::Table
        } else {
            Screen::Setup
        }
    });
    let notice = use_signal(|| None::<Notice>);
    let current = effective_screen(table.read().as_ref(), screen());

    // Zurück-Taste des Handys: Solange nicht die Übersicht zu sehen ist, liegt ein Eintrag im
    // Verlauf der WebView, damit die Taste in der App zurückgeht statt sie in den Hintergrund zu
    // schieben. `pending_pops` zählt die Einträge, die die App selbst wieder entfernt hat: Deren
    // Rückmeldung der WebView ist keine Eingabe des Nutzers.
    let mut entries = use_signal(|| 0u8);
    let mut pending_pops = use_signal(|| 0u8);
    use_effect(move || {
        let current = effective_screen(table.read().as_ref(), screen());
        let wants_entry = back_target(current, table.read().is_some()).is_some();
        let depth = *entries.peek();
        match history_action(wants_entry, depth) {
            HistoryAction::Push => {
                let _ = eval("history.pushState({ schafkopf: true }, '')");
                entries.set(1);
            }
            HistoryAction::Pop => {
                let _ = eval("history.back()");
                entries.set(0);
                pending_pops += 1;
            }
            HistoryAction::Nothing => {}
        }
    });
    use_future(move || async move {
        let mut events = eval("window.addEventListener('popstate', () => dioxus.send('back'))");
        while events.recv::<String>().await.is_ok() {
            if *pending_pops.peek() > 0 {
                pending_pops -= 1;
                continue;
            }
            let shown = effective_screen(table.read().as_ref(), screen());
            if let Some(target) = back_target(shown, table.read().is_some()) {
                // Der Verlaufseintrag ist verbraucht, der Effekt legt bei Bedarf einen neuen an.
                entries.set(0);
                screen.set(target);
            }
        }
    });

    // Alle Teile eines früheren Abends laufen über eine Komponente an einer Stelle, damit der
    // geladene Abend zwischen Übersicht, Statistik und Export nicht neu gelesen wird.
    let past = match current {
        Screen::Past(stamp) => Some((stamp, PastSection::Overview)),
        Screen::PastStats { stamp, player } => Some((stamp, PastSection::Stats(player))),
        Screen::PastExport(stamp) => Some((stamp, PastSection::Export)),
        _ => None,
    };
    let has_table = table.read().is_some();

    rsx! {
        style { dangerous_inner_html: CSS }
        main {
            h1 { "Schafkopf" }
            NoticeBar { notice }
            if let Some((stamp, section)) = past {
                PastEveningView { key: "{stamp}", stamp, section, screen, notice }
            } else {
                match current {
                    Screen::Setup => rsx! { Setup { table, screen, notice, library, state: setup } },
                    Screen::Table => rsx! { TableView { table, screen, notice } },
                    Screen::Game => rsx! { GameForm { table, screen, notice, editing: None } },
                    Screen::EditGame(index) => rsx! { GameForm { key: "{index}", table, screen, notice, editing: Some(index) } },
                    Screen::Settings => rsx! { SettingsScreen { table, screen, notice, library } },
                    Screen::Seats => rsx! { SeatsScreen { table, screen, notice, library } },
                    Screen::Library { from_setup } => rsx! { LibraryScreen { screen, notice, library, from_setup } },
                    Screen::Export => rsx! { ExportScreen { table, screen, notice, back: Screen::Table } },
                    Screen::Stats { player } => rsx! { StatsScreen { table, screen, back: Screen::Table, focus: player } },
                    Screen::PastList => rsx! { PastList { screen, has_table } },
                    // Diese drei zeigt `PastEveningView` oben.
                    Screen::Past(_) | Screen::PastStats { .. } | Screen::PastExport(_) => rsx! {},
                }
            }
        }
    }
}
