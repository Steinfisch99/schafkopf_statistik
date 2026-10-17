//! Schafkopf-Statistik: Abend einrichten, Spiele eintragen, Geldstand ansehen.

#[cfg_attr(not(target_os = "android"), allow(dead_code))]
mod android;
mod draft;
mod export;
mod gestures;
mod storage;
mod summary;
mod views;

use std::time::{SystemTime, UNIX_EPOCH};

use dioxus::prelude::*;
use views::{ExportScreen, GameForm, Notice, NoticeBar, Setup, TableView, TariffScreen};

const CSS: &str = include_str!("style.css");

/// Der Bildschirm, der gerade zu sehen ist.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Setup,
    Table,
    Game,
    Tariff,
    Export,
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
    let screen = use_signal(|| {
        if table.peek().is_some() {
            Screen::Table
        } else {
            Screen::Setup
        }
    });
    let notice = use_signal(|| None::<Notice>);
    // Ohne Tisch gibt es nur die Einrichtung.
    let current = if table.read().is_some() {
        screen()
    } else {
        Screen::Setup
    };

    rsx! {
        style { dangerous_inner_html: CSS }
        main {
            h1 { "Schafkopf" }
            NoticeBar { notice }
            match current {
                Screen::Setup => rsx! { Setup { table, screen, notice } },
                Screen::Table => rsx! { TableView { table, screen, notice } },
                Screen::Game => rsx! { GameForm { table, screen, notice } },
                Screen::Tariff => rsx! { TariffScreen { table, screen, notice } },
                Screen::Export => rsx! { ExportScreen { table, screen, notice } },
            }
        }
    }
}
