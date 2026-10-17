//! Die Zurück-Taste des Handys: Wohin führt "zurück", und wann braucht die WebView dafür einen
//! Verlaufseintrag? Reine Logik ohne Oberfläche.
//!
//! Die WebView geht bei der Zurück-Taste im Verlauf der Seite zurück, solange es dort etwas zum
//! Zurückgehen gibt, sonst schiebt Android die App in den Hintergrund. Die App legt deshalb einen
//! Verlaufseintrag an, sobald sie die Übersicht verlässt, und geht bei der Rückkehr dorthin
//! selbst zur übergeordneten Ansicht.

use schafkopf_core::Table;

use crate::Screen;

/// Der Bildschirm, der wirklich gezeigt wird: Ohne Abend gibt es nur die Einrichtung und die
/// Verwaltung, und ein Spiel, das es nicht mehr gibt, lässt sich nicht bearbeiten.
pub fn effective_screen(table: Option<&Table>, screen: Screen) -> Screen {
    match (table, screen) {
        // Die Verwaltung und die früheren Abende brauchen keinen laufenden Abend, sie sind auch
        // aus der Einrichtung erreichbar.
        (None, library @ Screen::Library { from_setup: true }) => library,
        (None, past) if is_past_screen(past) => past,
        (None, Screen::Backup) => Screen::Backup,
        (None, _) => Screen::Setup,
        (Some(t), Screen::EditGame(index)) if index >= t.games().len() => Screen::Table,
        (_, other) => other,
    }
}

/// Gehört der Bildschirm zu den früheren Abenden?
pub fn is_past_screen(screen: Screen) -> bool {
    matches!(
        screen,
        Screen::PastList
            | Screen::Past(_)
            | Screen::PastStats { .. }
            | Screen::PastExport(_)
            | Screen::Overall
    )
}

/// Wohin die Zurück-Taste von diesem Bildschirm führt. `None`: Das ist die oberste Ansicht, dort
/// darf die App in den Hintergrund gehen.
pub fn back_target(screen: Screen, has_table: bool) -> Option<Screen> {
    match screen {
        Screen::Table => None,
        Screen::Setup => has_table.then_some(Screen::Table),
        Screen::Game
        | Screen::EditGame(_)
        | Screen::Settings
        | Screen::Seats
        | Screen::Export
        | Screen::Stats { .. } => Some(Screen::Table),
        Screen::Library { from_setup: true } => Some(Screen::Setup),
        Screen::Library { from_setup: false } => Some(Screen::Settings),
        Screen::PastList => Some(if has_table {
            Screen::Table
        } else {
            Screen::Setup
        }),
        Screen::Past(_) | Screen::Overall => Some(Screen::PastList),
        Screen::Backup => Some(if has_table {
            Screen::Settings
        } else {
            Screen::Setup
        }),
        Screen::PastStats { stamp, .. } | Screen::PastExport(stamp) => Some(Screen::Past(stamp)),
    }
}

/// Was mit dem Verlaufseintrag der WebView zu tun ist.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HistoryAction {
    /// Einen Eintrag anlegen, damit die Zurück-Taste in der App bleibt.
    Push,
    /// Den Eintrag wieder entfernen, weil die App von selbst zur obersten Ansicht zurückkehrte.
    Pop,
    Nothing,
}

/// Es gibt immer höchstens einen Eintrag: einen, solange nicht die oberste Ansicht zu sehen ist.
/// `depth` ist die Zahl der Einträge, die die App gerade angelegt hat (0 oder 1).
pub fn history_action(wants_entry: bool, depth: u8) -> HistoryAction {
    match (wants_entry, depth) {
        (true, 0) => HistoryAction::Push,
        (false, 1..) => HistoryAction::Pop,
        _ => HistoryAction::Nothing,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schafkopf_core::{GameInput, GameKind, Tariff};

    fn table_with_one_game() -> Table {
        let players = ["A", "B", "C", "D"].map(String::from).to_vec();
        let mut t = Table::new("T", players, 0, Tariff::default()).unwrap();
        t.add_game(GameInput::new(0, GameKind::Wenz, 1)).unwrap();
        t
    }

    #[test]
    fn every_screen_has_a_way_back_except_the_overview() {
        assert_eq!(back_target(Screen::Table, true), None);
        for screen in [
            Screen::Game,
            Screen::EditGame(3),
            Screen::Settings,
            Screen::Seats,
            Screen::Export,
            Screen::Stats { player: None },
        ] {
            assert_eq!(back_target(screen, true), Some(Screen::Table), "{screen:?}");
        }
    }

    #[test]
    fn library_and_setup_go_back_to_where_they_came_from() {
        assert_eq!(
            back_target(Screen::Library { from_setup: true }, false),
            Some(Screen::Setup)
        );
        assert_eq!(
            back_target(Screen::Library { from_setup: false }, true),
            Some(Screen::Settings)
        );
        assert_eq!(back_target(Screen::Setup, true), Some(Screen::Table));
        assert_eq!(
            back_target(Screen::Setup, false),
            None,
            "ohne Abend ist die Einrichtung ganz oben"
        );
    }

    #[test]
    fn following_the_back_targets_always_ends_at_the_top() {
        let all = [
            Screen::Setup,
            Screen::Table,
            Screen::Game,
            Screen::EditGame(0),
            Screen::Settings,
            Screen::Seats,
            Screen::Library { from_setup: true },
            Screen::Library { from_setup: false },
            Screen::Export,
            Screen::Stats { player: None },
            Screen::PastList,
            Screen::Past(7),
            Screen::PastStats {
                stamp: 7,
                player: Some(1),
            },
            Screen::PastExport(7),
            Screen::Overall,
            Screen::Backup,
        ];
        for has_table in [true, false] {
            for start in all {
                let mut screen = start;
                for _ in 0..5 {
                    match back_target(screen, has_table) {
                        Some(next) => screen = next,
                        None => break,
                    }
                }
                assert_eq!(
                    back_target(screen, has_table),
                    None,
                    "{start:?} endet in einer Schleife"
                );
            }
        }
    }

    #[test]
    fn effective_screen_matches_what_exists() {
        let t = table_with_one_game();
        assert_eq!(effective_screen(None, Screen::Table), Screen::Setup);
        assert_eq!(
            effective_screen(None, Screen::Stats { player: None }),
            Screen::Setup
        );
        assert_eq!(
            effective_screen(None, Screen::Library { from_setup: true }),
            Screen::Library { from_setup: true }
        );
        assert_eq!(
            effective_screen(None, Screen::Library { from_setup: false }),
            Screen::Setup
        );
        assert_eq!(
            effective_screen(Some(&t), Screen::EditGame(0)),
            Screen::EditGame(0)
        );
        assert_eq!(
            effective_screen(Some(&t), Screen::EditGame(1)),
            Screen::Table
        );
        assert_eq!(
            effective_screen(Some(&t), Screen::Stats { player: Some(2) }),
            Screen::Stats { player: Some(2) }
        );
    }

    #[test]
    fn past_evenings_go_back_step_by_step() {
        assert_eq!(back_target(Screen::PastList, true), Some(Screen::Table));
        assert_eq!(back_target(Screen::PastList, false), Some(Screen::Setup));
        assert_eq!(back_target(Screen::Past(7), true), Some(Screen::PastList));
        assert_eq!(
            back_target(
                Screen::PastStats {
                    stamp: 7,
                    player: Some(2)
                },
                true
            ),
            Some(Screen::Past(7))
        );
        assert_eq!(
            back_target(Screen::PastExport(7), false),
            Some(Screen::Past(7))
        );
    }

    #[test]
    fn past_evenings_are_reachable_without_a_running_evening() {
        for screen in [
            Screen::PastList,
            Screen::Past(3),
            Screen::PastStats {
                stamp: 3,
                player: None,
            },
            Screen::PastExport(3),
            Screen::Overall,
        ] {
            assert!(is_past_screen(screen));
            assert_eq!(effective_screen(None, screen), screen, "{screen:?}");
        }
        assert!(!is_past_screen(Screen::Table) && !is_past_screen(Screen::Stats { player: None }));
    }

    #[test]
    fn at_most_one_history_entry_is_kept() {
        assert_eq!(history_action(true, 0), HistoryAction::Push);
        assert_eq!(history_action(true, 1), HistoryAction::Nothing);
        assert_eq!(history_action(false, 1), HistoryAction::Pop);
        assert_eq!(history_action(false, 0), HistoryAction::Nothing);
    }

    #[test]
    fn the_backup_screen_works_without_an_evening_and_goes_back_sensibly() {
        assert_eq!(effective_screen(None, Screen::Backup), Screen::Backup);
        assert_eq!(back_target(Screen::Backup, false), Some(Screen::Setup));
        assert_eq!(back_target(Screen::Backup, true), Some(Screen::Settings));
    }
}
