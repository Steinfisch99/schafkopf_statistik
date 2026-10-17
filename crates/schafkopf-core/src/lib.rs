//! Kern der Schafkopf-Statistik-App: Datenmodell, Abrechnung und Auswertung.
//!
//! Das Crate ist bewusst frei von UI- und Plattformcode, damit es auf Android
//! (und überall sonst) unverändert läuft und sich normal mit `cargo test` testen lässt.

mod csv;
mod error;
mod model;
mod scoring;
mod stats;
mod table;

pub use error::{Error, Result};
pub use model::{outcome_from_points, Cents, GameInput, GameKind, Outcome, Seat, Suit};
pub use scoring::{calculated_value, game_value, payouts, validate, Tariff};
pub use stats::{balance_history, table_stats, KindStats, PlayerStats, RoleStats, TableStats};
pub use table::{Game, Table};

#[cfg(test)]
mod tests;
