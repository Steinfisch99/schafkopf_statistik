use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::{Cents, GameInput, Seat};
use crate::scoring::{game_value, payouts, validate, Tariff};

/// Ein abgerechnetes Spiel. Beträge werden beim Eintragen festgeschrieben,
/// damit eine spätere Tarifänderung die Historie nicht verändert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub input: GameInput,
    /// Einzelwert des Spiels in Cent.
    pub value: Cents,
    /// Gewinn/Verlust pro Sitzplatz in Cent.
    pub payouts: Vec<Cents>,
}

/// Ein Tisch: Spieler in Sitzreihenfolge (Spielrichtung, im Uhrzeigersinn), Tarif und alle Spiele.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    pub version: u32,
    pub name: String,
    pub players: Vec<String>,
    /// Wer gibt im ersten Spiel? Danach wandert der Geber jeweils einen Platz weiter.
    pub first_dealer: Seat,
    pub tariff: Tariff,
    #[serde(default)]
    pub created_at: Option<i64>,
    games: Vec<Game>,
}

impl Table {
    pub const CURRENT_VERSION: u32 = 1;

    pub fn new(
        name: impl Into<String>,
        players: Vec<String>,
        first_dealer: Seat,
        tariff: Tariff,
    ) -> Result<Self> {
        if !(4..=5).contains(&players.len()) {
            return Err(Error::PlayerCount(players.len()));
        }
        let players: Vec<String> = players.into_iter().map(|p| p.trim().to_owned()).collect();
        for (i, p) in players.iter().enumerate() {
            if p.is_empty() {
                return Err(Error::EmptyName);
            }
            if players[..i].contains(p) {
                return Err(Error::DuplicateName(p.clone()));
            }
        }
        if first_dealer >= players.len() {
            return Err(Error::UnknownSeat(first_dealer));
        }
        Ok(Table {
            version: Self::CURRENT_VERSION,
            name: name.into(),
            players,
            first_dealer,
            tariff,
            created_at: None,
            games: Vec::new(),
        })
    }

    pub fn games(&self) -> &[Game] {
        &self.games
    }

    /// Anzahl der Spieler am Tisch (4 oder 5).
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Mitspielende Sitzplätze bei diesem Geber. Bei fünf Spielern setzt der Geber aus.
    pub fn active_seats(&self, dealer: Seat) -> Vec<Seat> {
        (0..self.player_count())
            .filter(|&s| self.player_count() == 4 || s != dealer)
            .collect()
    }

    /// Geber des nächsten Spiels.
    pub fn next_dealer(&self) -> Seat {
        match self.games.last() {
            Some(g) => (g.input.dealer + 1) % self.player_count(),
            None => self.first_dealer,
        }
    }

    /// Prüft die Eingabe, rechnet ab und hängt das Spiel an.
    pub fn add_game(&mut self, input: GameInput) -> Result<&Game> {
        let active = self.active_seats(input.dealer);
        validate(&input, self.player_count(), &active)?;
        let value = game_value(&self.tariff, &input);
        let payouts = payouts(&self.tariff, &input, self.player_count(), &active);
        self.games.push(Game {
            input,
            value,
            payouts,
        });
        Ok(self.games.last().expect("gerade eingefügt"))
    }

    /// Nimmt das letzte Spiel zurück, z. B. nach einer Fehleingabe.
    pub fn undo_last(&mut self) -> Option<Game> {
        self.games.pop()
    }

    /// Aktueller Kontostand pro Sitzplatz in Cent.
    pub fn balances(&self) -> Vec<Cents> {
        let mut sum = vec![0; self.player_count()];
        for g in &self.games {
            for (s, p) in sum.iter_mut().zip(&g.payouts) {
                *s += p;
            }
        }
        sum
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(|e| Error::Json(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        let table: Table = serde_json::from_str(json).map_err(|e| Error::Json(e.to_string()))?;
        if table.version != Self::CURRENT_VERSION {
            return Err(Error::UnsupportedVersion(table.version));
        }
        Ok(table)
    }
}
