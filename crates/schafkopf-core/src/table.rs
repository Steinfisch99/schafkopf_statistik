use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::library::RuleSet;
use crate::model::{BockMode, Cents, GameInput, GameKind, Seat};
use crate::scoring::{calculated_value, game_value, payouts, validate, Tariff};

/// Am Tisch sitzen zwischen 2 und 10 Spieler.
const MIN_PLAYERS: usize = 2;
const MAX_PLAYERS: usize = 10;

/// Ein abgerechnetes Spiel. Beträge werden beim Eintragen festgeschrieben,
/// damit eine spätere Tarifänderung die Historie nicht verändert.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Game {
    pub input: GameInput,
    /// Abgerechneter Einzelwert in Cent: der verlangte Preis, falls einer eingetragen wurde.
    pub value: Cents,
    /// Einzelwert nach Tarif. Weicht von `value` ab, wenn am Tisch ein anderer Preis verlangt wurde.
    pub calculated_value: Cents,
    /// Gewinn/Verlust pro Sitzplatz in Cent.
    pub payouts: Vec<Cents>,
    /// Der Tarif, mit dem dieses Spiel abgerechnet wurde. Beim Bearbeiten gilt er weiter, auch wenn
    /// der Tarif des Abends inzwischen geändert wurde. Spiele aus alten Dateien haben keinen und
    /// werden dann mit dem aktuellen Tarif des Abends neu gerechnet.
    #[serde(default)]
    pub tariff: Option<Tariff>,
    /// Wer bei diesem Spiel am Tisch saß (Sitzplätze, in Sitzreihenfolge). Leer in alten Dateien:
    /// dann saßen alle da, für die das Spiel eine Auszahlung hat.
    #[serde(default)]
    pub at_table: Vec<Seat>,
}

/// Ein Tisch mit 2 bis 10 Spielern: Spieler in Sitzreihenfolge (Spielrichtung, im Uhrzeigersinn), Tarif und alle Spiele.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    pub version: u32,
    pub name: String,
    /// Alle Spieler des Abends, auch die, die schon gegangen sind. Wer dazukommt, wird hinten
    /// angehängt, damit die Sitzplätze der früheren Spiele gleich bleiben.
    pub players: Vec<String>,
    /// Die Sitzreihenfolge aller Sitzplätze (Spielrichtung). Leer in alten Dateien: dann die
    /// Reihenfolge von `players`.
    #[serde(default)]
    pub seating: Vec<Seat>,
    /// Sitzplätze, deren Spieler gerade nicht am Tisch sind.
    #[serde(default)]
    pub absent: Vec<Seat>,
    /// Wer gibt im ersten Spiel? Danach wandert der Geber jeweils einen Platz weiter.
    pub first_dealer: Seat,
    pub tariff: Tariff,
    /// Name des Tarifs in der Bibliothek, falls er von dort stammt. Nur zur Anzeige.
    #[serde(default)]
    pub tariff_name: Option<String>,
    /// Die Spielarten, die in diesem Abend gespielt werden. Alte Dateien ohne Angabe: alle.
    #[serde(default = "all_kinds")]
    pub enabled_kinds: Vec<GameKind>,
    /// Name des Regelwerks in der Bibliothek, falls es von dort stammt. Nur zur Anzeige.
    #[serde(default)]
    pub rule_set_name: Option<String>,
    /// Ob und wie lange nach einem Auslöser Bockspiele folgen. Alte Dateien: kein Bock.
    #[serde(default)]
    pub bock_mode: BockMode,
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
        if !(MIN_PLAYERS..=MAX_PLAYERS).contains(&players.len()) {
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
            seating: Vec::new(),
            absent: Vec::new(),
            first_dealer,
            tariff,
            tariff_name: None,
            enabled_kinds: GameKind::ALL.to_vec(),
            rule_set_name: None,
            bock_mode: BockMode::Off,
            created_at: None,
            games: Vec::new(),
        })
    }

    pub fn games(&self) -> &[Game] {
        &self.games
    }

    /// Anzahl der Spieler, die gerade am Tisch sitzen (2 bis 10).
    pub fn player_count(&self) -> usize {
        self.players.len() - self.absent.len()
    }

    /// Alle Sitzplätze, auch die der Gegangenen, in Sitzreihenfolge.
    pub fn ring(&self) -> Vec<Seat> {
        if self.seating.len() == self.players.len() {
            self.seating.clone()
        } else {
            (0..self.players.len()).collect()
        }
    }

    /// Die Sitzplätze der Spieler, die gerade am Tisch sitzen, in Sitzreihenfolge.
    pub fn present_seats(&self) -> Vec<Seat> {
        self.ring()
            .into_iter()
            .filter(|s| !self.absent.contains(s))
            .collect()
    }

    /// Ob dieser Spieler gerade am Tisch sitzt.
    pub fn is_present(&self, seat: Seat) -> bool {
        seat < self.players.len() && !self.absent.contains(&seat)
    }

    /// Der nächste Sitzplatz nach `seat` in Spielrichtung, an dem jemand sitzt. Mit `inclusive`
    /// zählt `seat` selbst, falls dort jemand sitzt.
    fn next_present(&self, seat: Seat, inclusive: bool) -> Seat {
        let ring = self.ring();
        let start = ring.iter().position(|&s| s == seat).unwrap_or(0);
        let len = ring.len();
        for step in usize::from(!inclusive)..=len {
            let s = ring[(start + step) % len];
            if self.is_present(s) {
                return s;
            }
        }
        seat
    }

    /// Voreinstellung, wer bei diesem Geber mitspielt: der Geber, der Spieler vor ihm (er hebt ab)
    /// und die zwei Spieler nach ihm. Bei weniger als vier Spielern spielen alle mit.
    /// Die Reihenfolge ist die Spielreihenfolge: Vorhand, Mittelhand, der Abheber, der Geber.
    /// Wer nicht am Tisch sitzt, wird übersprungen.
    pub fn default_participants(&self, dealer: Seat) -> Vec<Seat> {
        let ring = self.present_seats();
        let n = ring.len();
        if n == 0 {
            return Vec::new();
        }
        let mut seats: Vec<Seat> = Vec::new();
        match ring.iter().position(|&s| s == dealer) {
            Some(d) => {
                for s in [d + 1, d + 2, d + n - 1, d] {
                    if !seats.contains(&ring[s % n]) {
                        seats.push(ring[s % n]);
                    }
                }
            }
            None => {
                let first = self.next_present(dealer, false);
                let d = ring.iter().position(|&s| s == first).unwrap_or(0);
                for i in 0..n.min(4) {
                    seats.push(ring[(d + i) % n]);
                }
            }
        }
        seats
    }

    /// Setzt einen neuen Spieler an den Tisch, hinter den Sitzplatz `after` (am Ende der Runde,
    /// wenn `None`). Für die früheren Spiele zahlt er nichts. Gibt seinen Sitzplatz zurück.
    pub fn add_player(&mut self, name: &str, after: Option<Seat>) -> Result<Seat> {
        let name = name.trim();
        if name.is_empty() {
            return Err(Error::EmptyName);
        }
        if self.players.iter().any(|p| p == name) {
            return Err(Error::DuplicateName(name.to_owned()));
        }
        if self.player_count() >= MAX_PLAYERS {
            return Err(Error::PlayerCount(self.player_count() + 1));
        }
        if let Some(a) = after {
            if a >= self.players.len() {
                return Err(Error::UnknownSeat(a));
            }
        }
        let mut ring = self.ring();
        let seat = self.players.len();
        let index = after
            .and_then(|a| ring.iter().position(|&s| s == a))
            .map_or(ring.len(), |i| i + 1);
        ring.insert(index, seat);
        self.players.push(name.to_owned());
        self.seating = ring;
        for game in &mut self.games {
            game.payouts.push(0);
        }
        Ok(seat)
    }

    /// Ein Spieler verlässt den Tisch. Seine Spiele und sein Stand bleiben erhalten.
    pub fn leave(&mut self, seat: Seat) -> Result<()> {
        if !self.is_present(seat) {
            return Err(Error::UnknownSeat(seat));
        }
        if self.player_count() <= MIN_PLAYERS {
            return Err(Error::PlayerCount(self.player_count() - 1));
        }
        self.absent.push(seat);
        Ok(())
    }

    /// Ein Spieler, der gegangen war, setzt sich wieder an den Tisch, auf seinen alten Platz.
    pub fn rejoin(&mut self, seat: Seat) -> Result<()> {
        if seat >= self.players.len() {
            return Err(Error::UnknownSeat(seat));
        }
        if self.is_present(seat) {
            return Ok(());
        }
        if self.player_count() >= MAX_PLAYERS {
            return Err(Error::PlayerCount(self.player_count() + 1));
        }
        self.absent.retain(|&s| s != seat);
        Ok(())
    }

    /// Ändert den Tarif für alle künftigen Spiele (ohne Namen aus der Bibliothek). Bereits
    /// eingetragene Spiele behalten ihre Beträge.
    pub fn set_tariff(&mut self, tariff: Tariff) {
        self.tariff = tariff;
        self.tariff_name = None;
    }

    /// Wie [`Table::set_tariff`], mit dem Namen des Tarifs aus der Bibliothek.
    pub fn use_tariff(&mut self, name: Option<String>, tariff: Tariff) {
        self.tariff = tariff;
        self.tariff_name = name;
    }

    /// Legt fest, welche Spielarten gespielt werden. Bereits eingetragene Spiele bleiben, auch mit
    /// einer Spielart, die danach nicht mehr aktiv ist. `name` ist der Name des Regelwerks aus der
    /// Bibliothek (nur zur Anzeige).
    pub fn use_rules(&mut self, name: Option<String>, kinds: &[GameKind]) -> Result<()> {
        let kinds: Vec<GameKind> = GameKind::ALL
            .into_iter()
            .filter(|k| kinds.contains(k))
            .collect();
        if kinds.is_empty() {
            return Err(Error::NoKinds);
        }
        self.enabled_kinds = kinds;
        self.rule_set_name = name;
        Ok(())
    }

    /// Übernimmt Spielarten, Bock-Modus und Namen eines Regelwerks aus der Bibliothek.
    pub fn use_rule_set(&mut self, rules: &RuleSet) -> Result<()> {
        self.use_rules(Some(rules.name.clone()), &rules.kinds)?;
        self.bock_mode = rules.bock;
        Ok(())
    }

    /// Wie viele Spiele noch Bockspiele sein sollen, wenn jetzt das nächste Spiel beginnt.
    pub fn bock_pending(&self) -> u32 {
        self.bock_pending_before(self.games.len())
    }

    /// Wie viele Spiele vor dem Spiel mit dieser Nummer (ab 0) noch Bockspiele sein sollen.
    ///
    /// Das wird aus den Spielen abgeleitet, nicht mitgezählt: Jedes Bockspiel verbraucht eines, jeder
    /// Auslöser bringt neue (eines oder so viele, wie Spieler am Tisch saßen, je nach Modus). So bleibt
    /// die Zahl nach Bearbeiten, Rückgängig und Löschen von Spielen richtig.
    pub fn bock_pending_before(&self, index: usize) -> u32 {
        let mut remaining: u32 = 0;
        for game in self.games.iter().take(index) {
            if game.input.bock {
                remaining = remaining.saturating_sub(1);
            }
            if game.input.starts_bock {
                remaining += match self.bock_mode {
                    BockMode::Off => 0,
                    BockMode::NextGame => 1,
                    BockMode::NextRound => self.players_at_table(game) as u32,
                };
            }
        }
        remaining
    }

    /// Wie viele Spieler bei diesem Spiel am Tisch saßen.
    fn players_at_table(&self, game: &Game) -> usize {
        if game.at_table.is_empty() {
            game.payouts.len()
        } else {
            game.at_table.len()
        }
    }

    /// Ob dieser Sitzplatz bei diesem Spiel am Tisch saß.
    pub fn was_at_table(&self, game: &Game, seat: Seat) -> bool {
        if game.at_table.is_empty() {
            seat < game.payouts.len()
        } else {
            game.at_table.contains(&seat)
        }
    }

    /// Ob diese Spielart in diesem Abend gespielt wird.
    pub fn is_enabled(&self, kind: GameKind) -> bool {
        self.enabled_kinds.contains(&kind)
    }

    /// Geber des nächsten Spiels: der Nächste in Spielrichtung, der am Tisch sitzt.
    pub fn next_dealer(&self) -> Seat {
        match self.games.last() {
            Some(g) => self.next_present(g.input.dealer, false),
            None => self.next_present(self.first_dealer, true),
        }
    }

    /// Prüft die Eingabe und rechnet sie ab, ohne das Spiel zu speichern.
    /// Damit zeigt die App den Preis und die Auszahlung schon vor dem Eintragen.
    pub fn preview_game(&self, input: GameInput) -> Result<Game> {
        for seat in input.participants.iter().chain([&input.dealer]) {
            if *seat < self.players.len() && !self.is_present(*seat) {
                return Err(Error::SeatNotAtTable(*seat));
            }
        }
        self.settle(&self.tariff, input, self.present_seats())
    }

    /// Rechnet ein Spiel ab. `at_table` sind die Sitzplätze, die dabei am Tisch saßen.
    fn settle(&self, tariff: &Tariff, input: GameInput, at_table: Vec<Seat>) -> Result<Game> {
        let mut input = input;
        if input.participants.is_empty() && input.dealer < self.players.len() {
            input.participants = self.default_participants(input.dealer);
        }
        validate(&input, self.players.len())?;
        let calculated_value = calculated_value(tariff, &input);
        let value = game_value(tariff, &input);
        let payouts = payouts(tariff, &input, self.players.len());
        Ok(Game {
            input,
            value,
            calculated_value,
            payouts,
            tariff: Some(tariff.clone()),
            at_table,
        })
    }

    /// Der Tarif, mit dem ein bestehendes Spiel abgerechnet wird: sein eigener, sonst der des Abends.
    fn tariff_of(&self, index: usize) -> Result<Tariff> {
        let game = self.games.get(index).ok_or(Error::UnknownGame(index))?;
        Ok(game.tariff.clone().unwrap_or_else(|| self.tariff.clone()))
    }

    /// Rechnet eine geänderte Fassung eines bestehenden Spiels, ohne es zu speichern. Es gilt der
    /// Tarif, mit dem das Spiel ursprünglich abgerechnet wurde.
    pub fn preview_edit(&self, index: usize, input: GameInput) -> Result<Game> {
        let tariff = self.tariff_of(index)?;
        self.settle(&tariff, input, self.games[index].at_table.clone())
    }

    /// Ersetzt ein bestehendes Spiel durch eine geänderte Fassung. Beträge werden mit dem Tarif
    /// von damals neu gerechnet, nicht mit dem heutigen.
    pub fn replace_game(&mut self, index: usize, input: GameInput) -> Result<&Game> {
        let game = self.preview_edit(index, input)?;
        self.games[index] = game;
        Ok(&self.games[index])
    }

    /// Entfernt ein beliebiges Spiel. `None`, wenn es den Index nicht gibt.
    pub fn remove_game(&mut self, index: usize) -> Option<Game> {
        (index < self.games.len()).then(|| self.games.remove(index))
    }

    /// Prüft die Eingabe, rechnet ab und hängt das Spiel an.
    pub fn add_game(&mut self, input: GameInput) -> Result<&Game> {
        let game = self.preview_game(input)?;
        self.games.push(game);
        Ok(self.games.last().expect("gerade eingefügt"))
    }

    /// Nimmt das letzte Spiel zurück, z. B. nach einer Fehleingabe.
    pub fn undo_last(&mut self) -> Option<Game> {
        self.games.pop()
    }

    /// Aktueller Kontostand pro Sitzplatz in Cent.
    pub fn balances(&self) -> Vec<Cents> {
        let mut sum = vec![0; self.players.len()];
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

fn all_kinds() -> Vec<GameKind> {
    GameKind::ALL.to_vec()
}
