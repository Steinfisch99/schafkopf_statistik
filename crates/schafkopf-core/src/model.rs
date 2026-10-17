use serde::{Deserialize, Serialize};

/// Sitzplatz am Tisch, entspricht dem Index in `Table::players` (Spielrichtung im Uhrzeigersinn).
pub type Seat = usize;

/// Geldbeträge in Cent, damit nie mit Fließkomma gerechnet wird. Negativ = verloren.
pub type Cents = i64;

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
}

/// Alles, was am Tisch über ein Spiel eingetragen wird.
///
/// Das Geld wird nicht eingetragen, sondern aus diesen Angaben und dem Tarif des Tisches berechnet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameInput {
    /// Wer hat gegeben? Bei fünf Spielern setzt der Geber aus.
    pub dealer: Seat,
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
    /// Hat der Spieler mit Re geantwortet (Verdopplung)?
    #[serde(default)]
    pub re: bool,
    /// Tout: Spieler sagt an, alle Stiche zu machen (Verdopplung).
    #[serde(default)]
    pub tout: bool,
    /// Hat die Spielerpartei gewonnen?
    pub won: bool,
    /// Hat die Siegerpartei Schneider erreicht?
    #[serde(default)]
    pub schneider: bool,
    /// Hat die Siegerpartei Schwarz erreicht (alle Stiche)? Schwarz zählt immer auch als Schneider.
    #[serde(default)]
    pub schwarz: bool,
    /// Anzahl Laufende (Karten in Folge von oben), die die Siegerpartei hatte.
    #[serde(default)]
    pub laufende: u8,
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
            kind,
            suit: None,
            declarer,
            partner: None,
            knockers: Vec::new(),
            kontra_by: None,
            re: false,
            tout: false,
            won: true,
            schneider: false,
            schwarz: false,
            laufende: 0,
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

    pub fn is_on_declarer_team(&self, seat: Seat) -> bool {
        seat == self.declarer || self.partner == Some(seat)
    }

    /// Anzahl der Verdopplungen durch Klopfen, Kontra, Re und Tout.
    pub fn doublings(&self) -> u32 {
        self.knockers.len() as u32
            + u32::from(self.kontra_by.is_some())
            + u32::from(self.re)
            + u32::from(self.tout)
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
