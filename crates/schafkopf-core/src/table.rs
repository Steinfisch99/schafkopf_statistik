use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::library::RuleSet;
use crate::model::{BockAtRamsch, BockMode, BockStacking, Cents, GameInput, GameKind, Seat};
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
    /// Name des Tarifs und des Regelwerks aus der Bibliothek, mit denen das Spiel gespielt wurde (nur zur
    /// Auswertung). Ältere Spiele haben keinen.
    #[serde(default)]
    pub tariff_name: Option<String>,
    #[serde(default)]
    pub rule_set_name: Option<String>,
}

impl Game {
    /// Wurde ein vom berechneten abweichender Preis verlangt (zu viel oder zu wenig)?
    pub fn demanded_wrong(&self) -> bool {
        self.input
            .price_override
            .is_some_and(|p| p != self.calculated_value)
    }

    /// Wurde zu viel verlangt?
    pub fn overcharged(&self) -> bool {
        self.input
            .price_override
            .is_some_and(|p| p > self.calculated_value)
    }

    /// Wurde zu wenig verlangt?
    pub fn undercharged(&self) -> bool {
        self.input
            .price_override
            .is_some_and(|p| p < self.calculated_value)
    }
}

/// Was am Tisch passiert ist, außer den Spielen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// Ein neuer Spieler setzt sich dazu.
    Joined,
    /// Ein Spieler geht.
    Left,
    /// Ein Spieler, der gegangen war, kommt zurück.
    Returned,
    /// Ein Spieler setzt sich um.
    Moved,
}

impl EventKind {
    pub fn label(self) -> &'static str {
        match self {
            EventKind::Joined => "Kommt dazu",
            EventKind::Left => "Geht",
            EventKind::Returned => "Kommt zurück",
            EventKind::Moved => "Setzt sich um",
        }
    }
}

/// Ein Kommen, Gehen oder Umsetzen im Lauf des Abends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableEvent {
    /// Zeitpunkt in Unix-Sekunden, falls bekannt.
    #[serde(default)]
    pub at: Option<i64>,
    pub kind: EventKind,
    pub seat: Seat,
    /// Hinter wem der Spieler saß oder sich hingesetzt hat, bei Dazukommen und Umsetzen.
    #[serde(default)]
    pub behind: Option<Seat>,
    /// Wie viele Spiele zu dem Zeitpunkt schon gespielt waren.
    #[serde(default)]
    pub after_games: usize,
}

/// Was ein Spieler falsch gemacht hat. Das Neumischen nach einem Fehler beim Geben gehört dazu: Der
/// Geber bleibt dann derselbe und mischt neu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OffenceKind {
    /// Nicht bedient, obwohl man konnte (Farbe oder Trumpf).
    NotFollowed,
    /// Aus der Reihe gespielt oder zu früh ausgespielt.
    OutOfTurn,
    /// Karten gezeigt oder hineingeschaut.
    ShowedCards,
    /// Absprache oder Zeichen gegeben.
    Collusion,
    /// Fehler beim Geben: Neu mischen, derselbe Geber.
    Misdeal,
    /// Etwas anderes.
    Other,
}

impl OffenceKind {
    pub const ALL: [OffenceKind; 6] = [
        OffenceKind::NotFollowed,
        OffenceKind::OutOfTurn,
        OffenceKind::ShowedCards,
        OffenceKind::Collusion,
        OffenceKind::Misdeal,
        OffenceKind::Other,
    ];

    pub fn label(self) -> &'static str {
        match self {
            OffenceKind::NotFollowed => "Nicht bedient",
            OffenceKind::OutOfTurn => "Aus der Reihe gespielt",
            OffenceKind::ShowedCards => "Karten gezeigt",
            OffenceKind::Collusion => "Absprache oder Zeichen",
            OffenceKind::Misdeal => "Falsch gegeben (neu gemischt)",
            OffenceKind::Other => "Etwas anderes",
        }
    }
}

/// Ein Vergehen oder Neumischen im Lauf des Abends, mit der Strafzahlung, die dabei festgeschrieben
/// wurde (auch wenn der Tarif sich später ändert).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Offence {
    pub kind: OffenceKind,
    /// Wer das Vergehen begangen hat (beim Neumischen: der Geber).
    pub offender: Seat,
    /// Strafe, die der Täter jedem Empfänger zahlt, in Cent. 0: keine Strafe.
    pub amount: Cents,
    /// Gewinn/Verlust pro Sitzplatz in Cent. Die Summe ist null.
    pub payouts: Vec<Cents>,
    /// Zeitpunkt in Unix-Sekunden, falls bekannt.
    #[serde(default)]
    pub at: Option<i64>,
    /// Wie viele Spiele zu dem Zeitpunkt schon gespielt waren (für den Verlauf).
    #[serde(default)]
    pub after_games: usize,
    #[serde(default)]
    pub note: Option<String>,
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
    /// Ob sich mehrere Bockrunden hintereinander reihen oder überlagern. Alte Dateien: hintereinander.
    #[serde(default)]
    pub bock_stacking: BockStacking,
    /// Was beim Ramsch mit einer Bockrunde passiert. Alte Dateien: Der Ramsch zählt als Bockspiel.
    #[serde(default)]
    pub bock_at_ramsch: BockAtRamsch,
    /// Abweichung der Ortszeit von UTC in Minuten, damit Uhrzeiten lesbar sind (Deutschland: 60/120).
    #[serde(default)]
    pub utc_offset_min: i32,
    /// Wann der Abend beendet wurde (Unix-Sekunden). `None`: noch offen.
    #[serde(default)]
    pub ended_at: Option<i64>,
    /// Ein Testabend: Er geht nicht in die Gesamtstatistik über mehrere Abende ein.
    #[serde(default)]
    pub debug: bool,
    /// Wer im Lauf des Abends kam, ging oder sich umsetzte.
    #[serde(default)]
    pub events: Vec<TableEvent>,
    /// Vergehen und Neumischen mit ihren Strafzahlungen. Alte Dateien: keine.
    #[serde(default)]
    pub offences: Vec<Offence>,
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
            bock_stacking: BockStacking::Sequential,
            bock_at_ramsch: BockAtRamsch::Counts,
            utc_offset_min: 0,
            ended_at: None,
            debug: false,
            events: Vec::new(),
            offences: Vec::new(),
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

    /// Hält fest, dass etwas am Tisch passiert ist.
    fn record(&mut self, at: Option<i64>, kind: EventKind, seat: Seat, behind: Option<Seat>) {
        self.events.push(TableEvent {
            at,
            kind,
            seat,
            behind,
            after_games: self.games.len(),
        });
    }

    /// Setzt einen neuen Spieler an den Tisch, hinter den Sitzplatz `after` (am Ende der Runde,
    /// wenn `None`). Für die früheren Spiele zahlt er nichts. `at` ist der Zeitpunkt in
    /// Unix-Sekunden für das Protokoll. Gibt seinen Sitzplatz zurück.
    pub fn add_player(&mut self, name: &str, after: Option<Seat>, at: Option<i64>) -> Result<Seat> {
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
        for offence in &mut self.offences {
            offence.payouts.push(0);
        }
        let behind = self.ring_before(seat);
        self.record(at, EventKind::Joined, seat, behind);
        Ok(seat)
    }

    /// Wer in der Sitzreihenfolge direkt vor diesem Platz sitzt, unabhängig davon, ob er da ist.
    fn ring_before(&self, seat: Seat) -> Option<Seat> {
        let ring = self.ring();
        let i = ring.iter().position(|&s| s == seat)?;
        (ring.len() > 1).then(|| ring[(i + ring.len() - 1) % ring.len()])
    }

    /// Ein Spieler verlässt den Tisch. Seine Spiele und sein Stand bleiben erhalten.
    pub fn leave(&mut self, seat: Seat, at: Option<i64>) -> Result<()> {
        if !self.is_present(seat) {
            return Err(Error::UnknownSeat(seat));
        }
        if self.player_count() <= MIN_PLAYERS {
            return Err(Error::PlayerCount(self.player_count() - 1));
        }
        self.absent.push(seat);
        self.record(at, EventKind::Left, seat, None);
        Ok(())
    }

    /// Ein Spieler, der gegangen war, setzt sich wieder an den Tisch, auf seinen alten Platz.
    pub fn rejoin(&mut self, seat: Seat, at: Option<i64>) -> Result<()> {
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
        self.record(at, EventKind::Returned, seat, None);
        Ok(())
    }

    /// Ein Spieler setzt sich um: Er sitzt danach direkt hinter dem Sitzplatz `after`. Der Geber
    /// wandert ab dem nächsten Spiel in der neuen Reihenfolge weiter.
    pub fn move_player(&mut self, seat: Seat, after: Seat, at: Option<i64>) -> Result<()> {
        for s in [seat, after] {
            if s >= self.players.len() {
                return Err(Error::UnknownSeat(s));
            }
        }
        if seat == after {
            return Err(Error::UnknownSeat(after));
        }
        let mut ring = self.ring();
        ring.retain(|&s| s != seat);
        let i = ring
            .iter()
            .position(|&s| s == after)
            .map_or(ring.len(), |i| i + 1);
        ring.insert(i, seat);
        if ring == self.ring() {
            return Ok(());
        }
        self.seating = ring;
        self.record(at, EventKind::Moved, seat, Some(after));
        Ok(())
    }

    /// Beendet den Abend zum Zeitpunkt `at` (Unix-Sekunden).
    pub fn finish(&mut self, at: i64) {
        self.ended_at = Some(at);
    }

    /// Der späteste bekannte Zeitpunkt am Abend: ein Spiel, ein Kommen oder Gehen, sonst der Beginn.
    pub fn last_activity(&self) -> Option<i64> {
        self.games
            .iter()
            .filter_map(|g| g.input.played_at)
            .chain(self.events.iter().filter_map(|e| e.at))
            .chain(self.offences.iter().filter_map(|o| o.at))
            .chain(self.created_at)
            .max()
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
        self.bock_stacking = rules.stacking;
        self.bock_at_ramsch = rules.bock_at_ramsch;
        Ok(())
    }

    /// Wie viele Spiele noch Bockspiele sein sollen, wenn jetzt das nächste Spiel beginnt.
    pub fn bock_pending(&self) -> u32 {
        self.bock_pending_before(self.games.len())
    }

    /// Wie viele Spiele vor dem Spiel mit dieser Nummer (ab 0) noch Bockspiele sein sollen.
    pub fn bock_pending_before(&self, index: usize) -> u32 {
        let layers = self.bock_layers_before(index);
        match self.bock_stacking {
            BockStacking::Sequential => layers.first().copied().unwrap_or(0),
            BockStacking::Stacked => layers.into_iter().max().unwrap_or(0),
        }
    }

    /// Wie oft das nächste Spiel gedoppelt ist, wenn jetzt das nächste Spiel beginnt.
    pub fn bock_level(&self) -> u32 {
        self.bock_level_before(self.games.len())
    }

    /// Wie oft das Spiel mit dieser Nummer (ab 0) durch Bock gedoppelt wäre: 0 ohne offenen Bock,
    /// sonst 1, bei überlagerten Bockrunden so viele, wie gerade laufen.
    pub fn bock_level_before(&self, index: usize) -> u32 {
        self.bock_layers_before(index).len() as u32
    }

    /// Die laufenden Bockrunden vor dem Spiel mit dieser Nummer, je mit den Spielen, die sie noch
    /// dauern. Hintereinander gibt es höchstens eine (sie wird länger), überlagert eine je Auslöser.
    ///
    /// Das wird aus den Spielen abgeleitet, nicht mitgezählt: Jedes Bockspiel verbraucht ein Spiel
    /// jeder laufenden Runde, jeder Auslöser bringt eine neue (ein Spiel oder so viele, wie Spieler
    /// am Tisch saßen, je nach Modus). So bleibt alles nach Bearbeiten, Rückgängig und Löschen richtig.
    fn bock_layers_before(&self, index: usize) -> Vec<u32> {
        let mut layers: Vec<u32> = Vec::new();
        for game in self.games.iter().take(index) {
            let ramsch = game.input.kind == GameKind::Ramsch;
            if ramsch && self.bock_at_ramsch == BockAtRamsch::Expires {
                layers.clear();
            } else if game.input.bock && !(ramsch && self.bock_at_ramsch == BockAtRamsch::Waits) {
                for l in &mut layers {
                    *l -= 1;
                }
                layers.retain(|&l| l > 0);
            }
            let length = match self.bock_mode {
                BockMode::Off => 0,
                BockMode::NextGame => 1,
                BockMode::NextRound => self.players_at_table(game) as u32,
            };
            for _ in 0..game.input.bock_triggers() {
                match (self.bock_stacking, layers.first_mut()) {
                    (BockStacking::Sequential, Some(first)) => *first += length,
                    _ if length > 0 => layers.push(length),
                    _ => {}
                }
            }
        }
        layers
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
        let names = (self.tariff_name.clone(), self.rule_set_name.clone());
        self.settle(&self.tariff, input, self.present_seats(), names)
    }

    /// Rechnet ein Spiel ab. `at_table` sind die Sitzplätze, die dabei am Tisch saßen.
    fn settle(
        &self,
        tariff: &Tariff,
        input: GameInput,
        at_table: Vec<Seat>,
        (tariff_name, rule_set_name): (Option<String>, Option<String>),
    ) -> Result<Game> {
        let mut input = input;
        if input.participants.is_empty() && input.dealer < self.players.len() {
            input.participants = self.default_participants(input.dealer);
        }
        if input.sie {
            // Beim Sie hat der Spieler immer alle 8 Laufende, nie die Gegner.
            input.laufende = 8;
            input.gegenlauf = false;
        }
        validate(&input, self.players.len())?;
        let calculated_value = calculated_value(tariff, &input);
        let demanded = input.price_override.filter(|&p| p != calculated_value);
        if demanded.is_none() {
            // Ohne abweichenden Preis gab es nichts, was falsch verlangt oder zurückverlangt wurde.
            input.mistakes.clear();
            input.price_error_by.clear();
        } else {
            input.mistakes.sort();
            input.mistakes.dedup();
            // Falsch verlangt hat nur die Gewinnerseite, ohne Angabe alle dort.
            let winners = input.winners();
            input.price_error_by.retain(|s| winners.contains(s));
            input.price_error_by.sort_unstable();
            input.price_error_by.dedup();
            if input.price_error_by.is_empty() {
                input.price_error_by = winners;
            }
        }
        // Zurückverlangen geht nur bei zu viel verlangtem Preis und wenn der Tarif es erlaubt.
        let too_much = demanded.is_some_and(|p| p > calculated_value);
        if input.reclaimed && too_much && tariff.reclaim_enabled {
            let losers = input.losers();
            input.reclaimed_by.retain(|s| losers.contains(s));
            input.reclaimed_by.sort_unstable();
            input.reclaimed_by.dedup();
            if input.reclaimed_by.is_empty() {
                input.reclaimed_by = losers;
            }
        } else {
            input.reclaimed = false;
            input.reclaimed_by.clear();
        }
        let value = game_value(tariff, &input);
        let payouts = payouts(tariff, &input, self.players.len());
        Ok(Game {
            input,
            value,
            calculated_value,
            payouts,
            tariff: Some(tariff.clone()),
            at_table,
            tariff_name,
            rule_set_name,
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
        let old = &self.games[index];
        let names = (old.tariff_name.clone(), old.rule_set_name.clone());
        self.settle(&tariff, input, old.at_table.clone(), names)
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

    /// Die Strafe, die der Tarif des Abends für diese Art von Vergehen vorsieht, je Mitspieler. 0, wenn
    /// es keine Strafzahlungen gibt.
    pub fn penalty_for(&self, kind: OffenceKind) -> Cents {
        match (self.tariff.penalties_enabled, kind) {
            (false, _) => 0,
            (true, OffenceKind::Misdeal) => self.tariff.misdeal_penalty,
            (true, _) => self.tariff.offence_penalty,
        }
    }

    /// Wer eine Strafe bekommt, wenn nichts anderes gewählt ist: die Mitspieler der laufenden Runde, also
    /// die, die beim nächsten Spiel mitspielen (siehe [`Table::default_participants`]), ohne den Täter.
    /// Wer gerade aussetzt, bekommt nichts.
    pub fn default_penalty_recipients(&self, offender: Seat) -> Vec<Seat> {
        self.default_participants(self.next_dealer())
            .into_iter()
            .filter(|&s| s != offender)
            .collect()
    }

    /// Hält ein Vergehen oder Neumischen fest. Der Täter zahlt `amount` an jeden der `recipients`, ohne
    /// Angabe an die Mitspieler der laufenden Runde ([`Table::default_penalty_recipients`]), nicht an alle
    /// am Tisch. Ohne `amount` gilt die Strafe aus dem Tarif ([`Table::penalty_for`]). Gibt der Tarif keine
    /// Strafzahlungen vor, kostet es nichts, es wird aber gezählt.
    pub fn add_offence(
        &mut self,
        kind: OffenceKind,
        offender: Seat,
        recipients: Option<Vec<Seat>>,
        amount: Option<Cents>,
        at: Option<i64>,
    ) -> Result<&Offence> {
        if offender >= self.players.len() {
            return Err(Error::UnknownSeat(offender));
        }
        if !self.is_present(offender) {
            return Err(Error::SeatNotAtTable(offender));
        }
        let amount = if self.tariff.penalties_enabled {
            amount.unwrap_or_else(|| self.penalty_for(kind))
        } else {
            0
        };
        if amount < 0 {
            return Err(Error::NegativePrice(amount));
        }
        let mut others = recipients.unwrap_or_else(|| self.default_penalty_recipients(offender));
        others.sort_unstable();
        others.dedup();
        others.retain(|&s| s != offender);
        for &s in &others {
            if s >= self.players.len() {
                return Err(Error::UnknownSeat(s));
            }
            if !self.is_present(s) {
                return Err(Error::SeatNotAtTable(s));
            }
        }
        let mut payouts = vec![0; self.players.len()];
        for &s in &others {
            payouts[s] = amount;
        }
        payouts[offender] = -amount * others.len() as Cents;
        self.offences.push(Offence {
            kind,
            offender,
            amount,
            payouts,
            at,
            after_games: self.games.len(),
            note: None,
        });
        Ok(self.offences.last().expect("gerade eingefügt"))
    }

    /// Nimmt ein Vergehen zurück. `None`, wenn es den Index nicht gibt.
    pub fn remove_offence(&mut self, index: usize) -> Option<Offence> {
        (index < self.offences.len()).then(|| self.offences.remove(index))
    }

    /// Aktueller Kontostand pro Sitzplatz in Cent, mit Strafzahlungen.
    pub fn balances(&self) -> Vec<Cents> {
        let mut sum = vec![0; self.players.len()];
        let payouts = self
            .games
            .iter()
            .map(|g| &g.payouts)
            .chain(self.offences.iter().map(|o| &o.payouts));
        for payouts in payouts {
            for (s, p) in sum.iter_mut().zip(payouts) {
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
