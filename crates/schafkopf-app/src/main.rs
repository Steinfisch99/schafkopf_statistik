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

use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::document::eval;
use dioxus::prelude::*;
use navigation::{back_target, effective_screen, history_action, HistoryAction};
use views::{
    use_setup_state, BackupScreen, ExportScreen, GameForm, LibraryScreen, Notice, NoticeBar,
    OffenceScreen, OverallScreen, PastEveningView, PastList, PastSection, SeatsScreen,
    SettingsScreen, Setup, StatsScreen, TableView,
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
    /// Ein Vergehen oder Neumischen eintragen.
    Offence,
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
    /// Die Statistik über mehrere Abende, erreichbar von den früheren Abenden und von der Einrichtung.
    Overall {
        from_past: bool,
    },
    /// Sicherung aller Daten als ZIP und Einspielen einer Sicherung.
    Backup,
}

pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

/// Abweichung der Ortszeit von UTC in Minuten, sobald die WebView sie genannt hat, sonst 0.
static UTC_OFFSET_MIN: AtomicI32 = AtomicI32::new(0);

pub fn utc_offset_min() -> i32 {
    UTC_OFFSET_MIN.load(Ordering::Relaxed)
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

    // Die Zeitzone kennt nur die WebView: Einmal abfragen, damit Uhrzeiten in Ortszeit stehen.
    use_future(|| async {
        let mut tz = eval("dioxus.send(-new Date().getTimezoneOffset())");
        if let Ok(minutes) = tz.recv::<i32>().await {
            UTC_OFFSET_MIN.store(minutes, Ordering::Relaxed);
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
                PastEveningView { key: "{stamp}", stamp, section, table, screen, notice }
            } else {
                match current {
                    Screen::Setup => rsx! { Setup { table, screen, notice, library, state: setup } },
                    Screen::Table => rsx! { TableView { table, screen, notice } },
                    Screen::Game => rsx! { GameForm { table, screen, notice, editing: None } },
                    Screen::EditGame(index) => rsx! { GameForm { key: "{index}", table, screen, notice, editing: Some(index) } },
                    Screen::Settings => rsx! { SettingsScreen { table, screen, notice, library } },
                    Screen::Seats => rsx! { SeatsScreen { table, screen, notice, library } },
                    Screen::Offence => rsx! { OffenceScreen { table, screen, notice } },
                    Screen::Library { from_setup } => rsx! { LibraryScreen { screen, notice, library, from_setup } },
                    Screen::Export => rsx! { ExportScreen { table, screen, notice, back: Screen::Table } },
                    Screen::Stats { player } => rsx! { StatsScreen { table, screen, back: Screen::Table, focus: player } },
                    Screen::PastList => rsx! { PastList { screen, has_table } },
                    // Diese drei zeigt `PastEveningView` oben.
                    Screen::Overall { from_past } => rsx! { OverallScreen { table, screen, notice, library, from_past } },
                    Screen::Backup => rsx! { BackupScreen { table, screen, notice, library } },
                    Screen::Past(_) | Screen::PastStats { .. } | Screen::PastExport(_) => rsx! {},
                }
            }
        }
    }
}
