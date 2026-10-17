use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::library::RuleSet;
use crate::model::{BockMode, Cents, GameInput, GameKind, Seat};
use crate::scoring::{calculated_value, game_value, payouts, validate, Tariff};

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
}

/// Ein Tisch mit 2 bis 10 Spielern: Spieler in Sitzreihenfolge (Spielrichtung, im Uhrzeigersinn), Tarif und alle Spiele.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    pub version: u32,
    pub name: String,
    pub players: Vec<String>,
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
        if !(2..=10).contains(&players.len()) {
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

    /// Anzahl der Spieler am Tisch (2 bis 10).
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Voreinstellung, wer bei diesem Geber mitspielt: der Geber, der Spieler vor ihm (er hebt ab)
    /// und die zwei Spieler nach ihm. Bei weniger als vier Spielern spielen alle mit.
    /// Die Reihenfolge ist die Spielreihenfolge: Vorhand, Mittelhand, der Abheber, der Geber.
    pub fn default_participants(&self, dealer: Seat) -> Vec<Seat> {
        let n = self.player_count();
        let mut seats: Vec<Seat> = Vec::new();
        for s in [dealer + 1, dealer + 2, dealer + n - 1, dealer] {
            if !seats.contains(&(s % n)) {
                seats.push(s % n);
            }
        }
        seats
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
        game.payouts.len()
    }

    /// Ob diese Spielart in diesem Abend gespielt wird.
    pub fn is_enabled(&self, kind: GameKind) -> bool {
        self.enabled_kinds.contains(&kind)
    }

    /// Geber des nächsten Spiels.
    pub fn next_dealer(&self) -> Seat {
        match self.games.last() {
            Some(g) => (g.input.dealer + 1) % self.player_count(),
            None => self.first_dealer,
        }
    }

    /// Prüft die Eingabe und rechnet sie ab, ohne das Spiel zu speichern.
    /// Damit zeigt die App den Preis und die Auszahlung schon vor dem Eintragen.
    pub fn preview_game(&self, input: GameInput) -> Result<Game> {
        self.settle(&self.tariff, input)
    }

    fn settle(&self, tariff: &Tariff, input: GameInput) -> Result<Game> {
        let mut input = input;
        if input.participants.is_empty() && input.dealer < self.player_count() {
            input.participants = self.default_participants(input.dealer);
        }
        validate(&input, self.player_count())?;
        let calculated_value = calculated_value(tariff, &input);
        let value = game_value(tariff, &input);
        let payouts = payouts(tariff, &input, self.player_count());
        Ok(Game {
            input,
            value,
            calculated_value,
            payouts,
            tariff: Some(tariff.clone()),
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
        self.settle(&self.tariff_of(index)?, input)
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

fn all_kinds() -> Vec<GameKind> {
    GameKind::ALL.to_vec()
}
