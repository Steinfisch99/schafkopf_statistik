//! Schafkopf-Statistik: Abend einrichten, Spiele eintragen, Geldstand ansehen.

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod android;
mod draft;
mod export;
mod gestures;
mod stats_text;
mod storage;
mod summary;
mod views;

use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use views::{
    use_setup_state, ExportScreen, GameForm, LibraryScreen, Notice, NoticeBar, SettingsScreen,
    Setup, StatsScreen, TableView,
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
    /// Verwaltung der gespeicherten Regelwerke und Tarife. `from_setup` sagt, wohin "Zurück" führt.
    Library {
        from_setup: bool,
    },
    Export,
    Stats,
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
    let setup = use_setup_state(library);
    let screen = use_signal(|| {
        if table.peek().is_some() {
            Screen::Table
        } else {
            Screen::Setup
        }
    });
    let notice = use_signal(|| None::<Notice>);
    let current = match (table.read().as_ref(), screen()) {
        // Die Verwaltung braucht keinen Abend, sie ist auch aus der Einrichtung erreichbar.
        (None, library_screen @ Screen::Library { from_setup: true }) => library_screen,
        // Ohne Abend gibt es nur die Einrichtung.
        (None, _) => Screen::Setup,
        // Ein Spiel, das es nicht mehr gibt, kann nicht bearbeitet werden.
        (Some(t), Screen::EditGame(index)) if index >= t.games().len() => Screen::Table,
        (_, other) => other,
    };

    rsx! {
        style { dangerous_inner_html: CSS }
        main {
            h1 { "Schafkopf" }
            NoticeBar { notice }
            match current {
                Screen::Setup => rsx! { Setup { table, screen, notice, library, state: setup } },
                Screen::Table => rsx! { TableView { table, screen, notice } },
                Screen::Game => rsx! { GameForm { table, screen, notice, editing: None } },
                Screen::EditGame(index) => rsx! { GameForm { key: "{index}", table, screen, notice, editing: Some(index) } },
                Screen::Settings => rsx! { SettingsScreen { table, screen, notice, library } },
                Screen::Library { from_setup } => rsx! { LibraryScreen { screen, notice, library, from_setup } },
                Screen::Export => rsx! { ExportScreen { table, screen, notice } },
                Screen::Stats => rsx! { StatsScreen { table, screen } },
            }
        }
    }
}
