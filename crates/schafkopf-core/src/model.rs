use serde::{Deserialize, Serialize};

/// Sitzplatz am Tisch, entspricht dem Index in `Table::players` (Spielrichtung im Uhrzeigersinn).
pub type Seat = usize;

/// Geldbeträge in Cent, damit nie mit Fließkomma gerechnet wird. Negativ = verloren.
pub type Cents = i64;

/// Was passiert, wenn eine Bockrunde entsteht, während schon eine läuft (oder zwei in einem Spiel
/// entstehen)? Ein Regelwerk legt es fest.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BockStacking {
    /// Sie reihen sich hintereinander: Bei zwei Bockrunden sind die nächsten zwei Runden gedoppelt.
    #[default]
    Sequential,
    /// Sie überlagern sich: Bei zwei Bockrunden zugleich ist der Preis vervierfacht.
    Stacked,
}

impl BockStacking {
    pub const ALL: [BockStacking; 2] = [BockStacking::Sequential, BockStacking::Stacked];

    pub fn label(self) -> &'static str {
        match self {
            BockStacking::Sequential => "Hintereinander",
            BockStacking::Stacked => "Übereinander (Preis mal 4, 8, ...)",
        }
    }
}

/// Liest einen oder mehrere Sitzplätze: Alte Dateien haben hier `null` oder eine Zahl, neue eine Liste.
fn seats_or_one<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<Seat>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        None(()),
        One(Seat),
        Many(Vec<Seat>),
    }
    Ok(match OneOrMany::deserialize(d)? {
        OneOrMany::None(()) => Vec::new(),
        OneOrMany::One(seat) => vec![seat],
        OneOrMany::Many(seats) => seats,
    })
}

/// So viele Bockrunden können in einem Spiel entstehen, und so oft kann ein Bockspiel gedoppelt sein.
pub const MAX_BOCK: u8 = 4;

/// Was beim Ramsch mit einer laufenden Bockrunde passiert, ein Regelwerk legt es fest.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BockAtRamsch {
    /// Der Ramsch zählt als Bockspiel und verbraucht eines. Ob er dadurch teurer wird, sagt der Tarif.
    #[default]
    Counts,
    /// Der Ramsch verbraucht keinen Bock, die Bockrunde wartet auf das nächste Spiel.
    Waits,
    /// Wird ein Ramsch gespielt, verfällt die Bockrunde.
    Expires,
}

impl BockAtRamsch {
    pub const ALL: [BockAtRamsch; 3] = [
        BockAtRamsch::Counts,
        BockAtRamsch::Waits,
        BockAtRamsch::Expires,
    ];

    pub fn label(self) -> &'static str {
        match self {
            BockAtRamsch::Counts => "Ramsch zählt als Bockspiel",
            BockAtRamsch::Waits => "Bockrunde wartet",
            BockAtRamsch::Expires => "Bockrunde verfällt",
        }
    }
}

/// Was beim Verlangen des Preises vergessen oder falsch gemacht wurde, wenn der Preis korrigiert wird.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum PriceMistake {
    /// Falscher Grundtarif, zum Beispiel Solo- statt Sauspielpreis.
    WrongBase,
    /// Falsche Anzahl an Doppler verlangt (Klopfen, Kontra, Re, Tout).
    WrongDoublings,
    /// Die Bockrunde vergessen.
    BockForgotten,
    /// Nicht alle Laufende oder gar keine verlangt, oder zu viele.
    WrongLaufende,
    /// Schneider nicht verlangt, oder zu viel.
    SchneiderMissed,
    /// Schwarz nicht verlangt (oder zu viel).
    SchwarzMissed,
    /// Falsch gezählt: Die falsche Partei hat gewonnen.
    WrongWinner,
    /// Verrechnet.
    Miscalculated,
    /// Etwas anderes.
    Other,
    /// Schneider verlangt, obwohl keiner erreicht wurde (dazugedichtet).
    SchneiderInvented,
    /// Schwarz verlangt, obwohl keiner erreicht wurde (dazugedichtet).
    SchwarzInvented,
    /// Bock verlangt, obwohl es keinen gab.
    BockInvented,
}

impl PriceMistake {
    pub const ALL: [PriceMistake; 12] = [
        PriceMistake::WrongBase,
        PriceMistake::WrongDoublings,
        PriceMistake::BockForgotten,
        PriceMistake::WrongLaufende,
        PriceMistake::SchneiderMissed,
        PriceMistake::SchwarzMissed,
        PriceMistake::WrongWinner,
        PriceMistake::Miscalculated,
        PriceMistake::Other,
        PriceMistake::SchneiderInvented,
        PriceMistake::SchwarzInvented,
        PriceMistake::BockInvented,
    ];

    pub fn label(self) -> &'static str {
        match self {
            PriceMistake::WrongBase => "Falscher Grundtarif",
            PriceMistake::WrongDoublings => "Falsche Anzahl Doppler",
            PriceMistake::BockForgotten => "Bockrunde vergessen",
            PriceMistake::WrongLaufende => "Laufende nicht alle oder keine",
            PriceMistake::SchneiderMissed => "Schneider nicht verlangt",
            PriceMistake::SchwarzMissed => "Schwarz nicht verlangt",
            PriceMistake::WrongWinner => "Falsche Partei gewonnen",
            PriceMistake::Miscalculated => "Verrechnet",
            PriceMistake::Other => "Etwas anderes",
            PriceMistake::SchneiderInvented => "Schneider dazugedichtet",
            PriceMistake::SchwarzInvented => "Schwarz dazugedichtet",
            PriceMistake::BockInvented => "Bock dazugedichtet",
        }
    }

    /// Nur bei zu hohem (`Some(true)`) oder nur bei zu niedrigem Preis (`Some(false)`) sinnvoll, sonst
    /// bei beidem. Wer zu viel verlangt, hat nichts vergessen, sondern etwas dazugedichtet.
    pub fn only_when_too_high(self) -> Option<bool> {
        match self {
            PriceMistake::BockForgotten
            | PriceMistake::SchneiderMissed
            | PriceMistake::SchwarzMissed => Some(false),
            PriceMistake::SchneiderInvented
            | PriceMistake::SchwarzInvented
            | PriceMistake::BockInvented => Some(true),
            _ => None,
        }
    }

    /// Die Fehlerarten, die zu einem zu hohen (`too_high`) oder zu niedrigen Preis passen.
    pub fn for_price(too_high: bool) -> Vec<PriceMistake> {
        Self::ALL
            .into_iter()
            .filter(|m| m.only_when_too_high().is_none_or(|high| high == too_high))
            .collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Suit {
    Eichel,
    Gras,
    Herz,
    Schellen,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Eichel, Suit::Gras, Suit::Herz, Suit::Schellen];

    pub fn label(self) -> &'static str {
        match self {
            Suit::Eichel => "Eichel",
            Suit::Gras => "Gras",
            Suit::Herz => "Herz",
            Suit::Schellen => "Schellen",
        }
    }
}

/// Wie lange eine ausgelöste Bockrunde dauert, ein Regelwerk legt es fest.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BockMode {
    /// Es gibt keinen Bock.
    #[default]
    Off,
    /// Nur das nächste Spiel ist ein Bockspiel.
    NextGame,
    /// Die ganze nächste Runde sind Bockspiele: so viele Spiele, wie Spieler am Tisch sitzen.
    NextRound,
}

impl BockMode {
    pub const ALL: [BockMode; 3] = [BockMode::Off, BockMode::NextGame, BockMode::NextRound];

    pub fn label(self) -> &'static str {
        match self {
            BockMode::Off => "Kein Bock",
            BockMode::NextGame => "Nur das nächste Spiel",
            BockMode::NextRound => "Die ganze nächste Runde",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GameKind {
    /// Rufspiel: Spieler ruft eine Sau (nicht Herz), der Besitzer ist der heimliche Mitspieler.
    Sauspiel,
    /// Hochzeit: Spieler sucht sich per Tauschkarte einen Mitspieler.
    Hochzeit,
    Wenz,
    Geier,
    /// Solo mit Farbe als Trumpf (Eichel-, Gras-, Herz-, Schellen-Solo).
    Farbsolo,
    Farbwenz,
    Farbgeier,
    /// Spieler gewinnt, wenn er keinen Stich macht.
    Bettel,
    /// Wer die meisten Augen hat, verliert. `declarer` ist der Verlierer (bzw. Durchmarschierer).
    Ramsch,
}

impl GameKind {
    pub const ALL: [GameKind; 9] = [
        GameKind::Sauspiel,
        GameKind::Hochzeit,
        GameKind::Wenz,
        GameKind::Geier,
        GameKind::Farbsolo,
        GameKind::Farbwenz,
        GameKind::Farbgeier,
        GameKind::Bettel,
        GameKind::Ramsch,
    ];

    pub fn label(self) -> &'static str {
        match self {
            GameKind::Sauspiel => "Sauspiel",
            GameKind::Hochzeit => "Hochzeit",
            GameKind::Wenz => "Wenz",
            GameKind::Geier => "Geier",
            GameKind::Farbsolo => "Farbsolo",
            GameKind::Farbwenz => "Farbwenz",
            GameKind::Farbgeier => "Farbgeier",
            GameKind::Bettel => "Bettel",
            GameKind::Ramsch => "Ramsch",
        }
    }

    /// Zwei gegen zwei: Spieler plus Mitspieler gegen die anderen beiden.
    pub fn is_team_game(self) -> bool {
        matches!(self, GameKind::Sauspiel | GameKind::Hochzeit)
    }

    /// Solospiele, bei denen "Tout" angesagt werden kann.
    pub fn allows_tout(self) -> bool {
        matches!(
            self,
            GameKind::Wenz
                | GameKind::Geier
                | GameKind::Farbsolo
                | GameKind::Farbwenz
                | GameKind::Farbgeier
        )
    }

    /// Alleinspiele, bei denen der Spieler gegen alle anderen spielt (alles außer Sauspiel, Hochzeit
    /// und Ramsch). Die Spieleliste färbt sie anders als die Spiele zu zweit.
    pub fn is_solo(self) -> bool {
        !self.is_team_game() && self != GameKind::Ramsch
    }

    /// Spiele mit Trumpffarbe.
    pub fn needs_trump_suit(self) -> bool {
        matches!(
            self,
            GameKind::Farbsolo | GameKind::Farbwenz | GameKind::Farbgeier
        )
    }

    /// Spiele, bei denen Schneider, Schwarz und Laufende zählen.
    pub fn has_extras(self) -> bool {
        !matches!(self, GameKind::Bettel | GameKind::Ramsch)
    }

    /// Wie viele Laufende es höchstens geben kann: so viele Trümpfe in Folge von oben, wie es in
    /// dieser Spielart gibt, und nicht mehr, als die haltende Partei Karten hat.
    ///
    /// Trümpfe: Beim Sauspiel, der Hochzeit und dem Farbsolo alle 14 (4 Ober, 4 Unter, 6 Herz),
    /// beim Wenz die 4 Unter, beim Geier die 4 Ober, beim Farbwenz und Farbgeier 4 plus 7 Karten
    /// der Farbe, also 11. Ohne Zuschläge gibt es keine Laufenden.
    ///
    /// Karten: Beim Solo hat die Spielerpartei nur die 8 Karten des Spielers, ein **Lauf** ist dort
    /// also höchstens 8. Die Gegner haben zusammen 24 Karten, ein **Gegenlauf** kann also alle
    /// Trümpfe umfassen. Beim Sauspiel und der Hochzeit hat jede Partei 16 Karten, das reicht für alle.
    pub fn max_laufende(self, gegenlauf: bool) -> u8 {
        let trumps = match self {
            GameKind::Sauspiel | GameKind::Hochzeit | GameKind::Farbsolo => 14,
            GameKind::Wenz | GameKind::Geier => 4,
            GameKind::Farbwenz | GameKind::Farbgeier => 11,
            GameKind::Bettel | GameKind::Ramsch => 0,
        };
        let solo_hand = if self.is_team_game() || gegenlauf {
            u8::MAX
        } else {
            8
        };
        trumps.min(solo_hand)
    }
}

/// Alles, was am Tisch über ein Spiel eingetragen wird.
///
/// Das Geld wird nicht eingetragen, sondern aus diesen Angaben und dem Tarif des Tisches berechnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameInput {
    /// Wer hat gegeben?
    pub dealer: Seat,
    /// Wer spielt in diesem Spiel mit (2 bis 4 Plätze, die übrigen setzen aus)? Leer lassen, dann
    /// setzt `Table::add_game` die Voreinstellung `Table::default_participants` ein.
    #[serde(default)]
    pub participants: Vec<Seat>,
    pub kind: GameKind,
    /// Beim Sauspiel die gerufene Sau, bei Farbsolo/-wenz/-geier die Trumpffarbe.
    #[serde(default)]
    pub suit: Option<Suit>,
    /// Der Spieler (bei Ramsch: der Verlierer).
    pub declarer: Seat,
    /// Mitspieler bei Sauspiel und Hochzeit.
    #[serde(default)]
    pub partner: Option<Seat>,
    /// Wer hat vor dem ersten Ausspiel geklopft (jeweils eine Verdopplung)?
    #[serde(default)]
    pub knockers: Vec<Seat>,
    /// Wer hat Kontra gegeben (Verdopplung)?
    #[serde(default)]
    pub kontra_by: Option<Seat>,
    /// Hat die Spielerpartei mit Re geantwortet (Verdopplung)?
    #[serde(default)]
    pub re: bool,
    /// Wer von der Spielerpartei Re gegeben hat. Fehlt bei älteren Spielen: dann gilt der Spieler.
    #[serde(default)]
    pub re_by: Option<Seat>,
    /// Tout: Spieler sagt an, alle Stiche zu machen (Verdopplung).
    #[serde(default)]
    pub tout: bool,
    /// Sie: die Steigerung des Tout beim Farbsolo. Der Spieler hat dabei immer alle 8 Laufende,
    /// Schneider und Schwarz zählen nicht zum Preis. Eine Verdopplung über das Tout hinaus gibt es nicht.
    /// Geht nur zusammen mit Tout.
    #[serde(default)]
    pub sie: bool,
    /// Hat die Spielerpartei gewonnen?
    pub won: bool,
    /// Hat die Siegerpartei Schneider erreicht?
    #[serde(default)]
    pub schneider: bool,
    /// Hat die Siegerpartei Schwarz erreicht (alle Stiche)? Schwarz zählt immer auch als Schneider.
    #[serde(default)]
    pub schwarz: bool,
    /// Anzahl Laufende (Karten in Folge von oben), die eine der beiden Parteien hatte.
    #[serde(default)]
    pub laufende: u8,
    /// Hatte die Gegenpartei die Laufende (Gegenlauf) statt der Spielerpartei (Lauf)? Der Preis ist
    /// in beiden Fällen derselbe, es zählt für die Statistik.
    #[serde(default)]
    pub gegenlauf: bool,
    /// War dieses Spiel ein Bockspiel? Dann ist der Preis doppelt.
    #[serde(default)]
    pub bock: bool,
    /// Hat dieses Spiel eine Bockrunde ausgelöst, zum Beispiel durch Schwarz? Die folgenden Spiele
    /// sind dann Bockspiele, wie viele, bestimmt der Bock-Modus des Regelwerks.
    #[serde(default)]
    pub starts_bock: bool,
    /// Wie oft ein Bockspiel zusätzlich gedoppelt ist (überlagerte Bockrunden): 1 heißt mal 4.
    /// Zählt nur bei `bock`.
    #[serde(default)]
    pub bock_extra: u8,
    /// Wie viele Bockrunden zusätzlich zur ersten durch dieses Spiel entstanden sind (sehr selten).
    /// Zählt nur bei `starts_bock`.
    #[serde(default)]
    pub extra_bock_rounds: u8,
    /// Verlangter Einzelwert in Cent, falls am Tisch etwas anderes verlangt wurde als der Tarif
    /// ergibt. Ersetzt dann die Berechnung.
    #[serde(default)]
    pub price_override: Option<Cents>,
    /// Was beim Verlangen des Preises schiefging. Nur bei einem korrigierten Preis gespeichert.
    #[serde(default)]
    pub mistakes: Vec<PriceMistake>,
    /// Wer den Preis falsch verlangt hat. Nur die Gewinnerseite verlangt, denn der Verlierer nie, beim
    /// Sauspiel sind es also beide Gewinner. Nur bei einem korrigierten Preis gespeichert, ohne Angabe
    /// alle auf der Gewinnerseite.
    #[serde(default, deserialize_with = "seats_or_one")]
    pub price_error_by: Vec<Seat>,
    /// Haben die Verlierer zu viel verlangten Betrag zurückverlangt? Sie verlangen das Doppelte des
    /// Mehrbetrags zurück (siehe `reclaimed_value`). Zählt nur, wenn der Preis zu hoch verlangt wurde und
    /// der Tarif das Zurückverlangen erlaubt, sonst wird es beim Abrechnen verworfen.
    #[serde(default)]
    pub reclaimed: bool,
    /// Wer zurückverlangt hat (Verlierer). Ohne Angabe alle Verlierer. Nur bei `reclaimed` gespeichert.
    #[serde(default)]
    pub reclaimed_by: Vec<Seat>,
    /// Augen der Spielerpartei, optional für die Statistik.
    #[serde(default)]
    pub points: Option<u8>,
    /// Zeitpunkt als Unix-Sekunden, von der App gesetzt.
    #[serde(default)]
    pub played_at: Option<i64>,
    #[serde(default)]
    pub note: Option<String>,
}

impl GameInput {
    /// Neues Spiel mit Standardwerten: gewonnen, ohne Klopfen, Kontra, Schneider oder Laufende.
    pub fn new(dealer: Seat, kind: GameKind, declarer: Seat) -> Self {
        GameInput {
            dealer,
            participants: Vec::new(),
            kind,
            suit: None,
            declarer,
            partner: None,
            knockers: Vec::new(),
            kontra_by: None,
            re: false,
            re_by: None,
            tout: false,
            sie: false,
            won: true,
            schneider: false,
            schwarz: false,
            laufende: 0,
            gegenlauf: false,
            bock: false,
            starts_bock: false,
            bock_extra: 0,
            extra_bock_rounds: 0,
            price_override: None,
            mistakes: Vec::new(),
            price_error_by: Vec::new(),
            reclaimed: false,
            reclaimed_by: Vec::new(),
            points: None,
            played_at: None,
            note: None,
        }
    }

    /// Die Spielerpartei: Spieler allein oder Spieler plus Mitspieler.
    pub fn declarer_team(&self) -> Vec<Seat> {
        let mut team = vec![self.declarer];
        team.extend(self.partner);
        team
    }

    /// Wer Re gegeben hat, falls Re gefallen ist (bei älteren Spielen ohne Angabe der Spieler).
    pub fn re_giver(&self) -> Option<Seat> {
        self.re.then(|| self.re_by.unwrap_or(self.declarer))
    }

    pub fn is_on_declarer_team(&self, seat: Seat) -> bool {
        seat == self.declarer || self.partner == Some(seat)
    }

    /// Die Gewinnerseite unter den Mitspielenden: die Spielerpartei, wenn sie gewonnen hat, sonst die
    /// Gegner. Nur sie verlangt den Preis, der Verlierer nie.
    pub fn winners(&self) -> Vec<Seat> {
        self.participants
            .iter()
            .copied()
            .filter(|&s| self.is_on_declarer_team(s) == self.won)
            .collect()
    }

    /// Die Verliererseite unter den Mitspielenden. Nur sie kann zu viel Verlangtes zurückverlangen.
    pub fn losers(&self) -> Vec<Seat> {
        self.participants
            .iter()
            .copied()
            .filter(|&s| self.is_on_declarer_team(s) != self.won)
            .collect()
    }

    /// Anzahl der Verdopplungen durch Klopfen, Kontra, Re, Tout und Bock (Sie verdoppelt nicht zusätzlich).
    pub fn doublings(&self) -> u32 {
        self.knockers.len() as u32
            + u32::from(self.kontra_by.is_some())
            + u32::from(self.re)
            + u32::from(self.tout)
            + self.bock_doublings()
    }

    /// Verdopplungen durch Bock: keine, eine oder bei überlagerten Bockrunden mehrere.
    pub fn bock_doublings(&self) -> u32 {
        if self.bock {
            1 + u32::from(self.bock_extra)
        } else {
            0
        }
    }

    /// Wie viele Bockrunden dieses Spiel ausgelöst hat.
    pub fn bock_triggers(&self) -> u32 {
        if self.starts_bock {
            1 + u32::from(self.extra_bock_rounds)
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Outcome {
    pub won: bool,
    pub schneider: bool,
}

/// Leitet Sieg und Schneider aus den Augen der Spielerpartei ab.
///
/// Gewonnen ab 61 Augen. Schneider, wenn die Verliererpartei höchstens 30 Augen hat
/// (Siegerpartei also mindestens 90). Schwarz lässt sich aus den Augen nicht ableiten.
pub fn outcome_from_points(declarer_points: u8) -> Outcome {
    let p = declarer_points.min(120);
    if p >= 61 {
        Outcome {
            won: true,
            schneider: p >= 90,
        }
    } else {
        Outcome {
            won: false,
            schneider: p <= 30,
        }
    }
}
