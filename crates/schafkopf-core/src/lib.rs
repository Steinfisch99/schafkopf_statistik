//! Kern der Schafkopf-Statistik-App: Datenmodell, Abrechnung und Auswertung.
//!
//! Das Crate ist bewusst frei von UI- und Plattformcode, damit es auf Android
//! (und überall sonst) unverändert läuft und sich normal mit `cargo test` testen lässt.

mod backup;
mod clock;
mod csv;
mod error;
mod library;
mod meta;
mod model;
mod money;
mod overall;
mod scoring;
mod stats;
mod table;

pub use backup::{
    pack, read_backup, unpack, Backup, CURRENT_FILE, LIBRARY_FILE, MAX_UNPACKED_BYTES,
};
pub use clock::{
    civil, format_clock, format_duration, format_local, format_minutes_after_noon,
    minutes_after_noon,
};
pub use error::{Error, Result};
pub use library::{Library, MergeReport, NamedTariff, RuleSet};
pub use meta::{table_meta, Presence, TableMeta, UseCount};
pub use model::{
    outcome_from_points, BockAtRamsch, BockMode, BockStacking, Cents, GameInput, GameKind, Outcome,
    PriceMistake, Seat, Suit, MAX_BOCK,
};
pub use money::parse_euro;
pub use overall::{overall_stats, time_spread, OverallPlayer, OverallStats};
pub use scoring::{
    calculated_value, game_value, payouts, price_doublings, reclaimed_value, validate, Tariff,
};
pub use stats::{
    balance_history, table_stats, KindStats, LaufendeStats, PlayerStats, RoleStats, TableStats,
};
pub use table::{EventKind, Game, Offence, OffenceKind, Table, TableEvent};

#[cfg(test)]
mod tests;
