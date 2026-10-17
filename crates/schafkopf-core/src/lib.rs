//! Kern der Schafkopf-Statistik-App: Datenmodell, Abrechnung und Auswertung.
//!
//! Das Crate ist bewusst frei von UI- und Plattformcode, damit es auf Android
//! (und überall sonst) unverändert läuft und sich normal mit `cargo test` testen lässt.

mod clock;
mod csv;
mod error;
mod library;
mod model;
mod money;
mod scoring;
mod stats;
mod table;

pub use clock::{civil, format_clock, format_duration, format_local};
pub use error::{Error, Result};
pub use library::{Library, NamedTariff, RuleSet};
pub use model::{
    outcome_from_points, BockAtRamsch, BockMode, BockStacking, Cents, GameInput, GameKind, Outcome,
    PriceMistake, Seat, Suit, MAX_BOCK,
};
pub use money::parse_euro;
pub use scoring::{calculated_value, game_value, payouts, validate, Tariff};
pub use stats::{
    balance_history, table_stats, KindStats, LaufendeStats, PlayerStats, RoleStats, TableStats,
};
pub use table::{EventKind, Game, Table, TableEvent};

#[cfg(test)]
mod tests;
