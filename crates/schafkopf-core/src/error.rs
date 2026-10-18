use crate::model::{GameKind, Seat};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("Ein Tisch braucht 2 bis 10 Spieler, nicht {0}")]
    PlayerCount(usize),
    #[error("Spielernamen dürfen nicht leer sein")]
    EmptyName,
    #[error("Spielername kommt doppelt vor: {0}")]
    DuplicateName(String),
    #[error("Es gibt kein Spiel mit der Nummer {}", .0 + 1)]
    UnknownGame(usize),
    #[error("Unbekannter Sitzplatz {0}")]
    UnknownSeat(Seat),
    #[error("Beim Bock sind höchstens 4 Bockrunden möglich, nicht {0}")]
    TooManyBock(u8),
    #[error("Sitzplatz {0} ist gerade nicht am Tisch")]
    SeatNotAtTable(Seat),
    #[error("Sitzplatz {0} sitzt in diesem Spiel aus")]
    SeatNotActive(Seat),
    #[error("{} braucht einen Mitspieler", .0.label())]
    PartnerRequired(GameKind),
    #[error("{} hat keinen Mitspieler", .0.label())]
    PartnerNotAllowed(GameKind),
    #[error("{} geht nur mit genau vier Mitspielenden", .0.label())]
    TeamGameNeedsFour(GameKind),
    #[error("Ein Spiel braucht 2 bis 4 verschiedene Mitspielende, nicht {0}")]
    BadParticipantCount(usize),
    #[error("Sitzplatz {0} ist doppelt unter den Mitspielenden")]
    DuplicateParticipant(Seat),
    #[error("Mitspieler und Spieler müssen verschiedene Personen sein")]
    PartnerIsDeclarer,
    #[error("{} braucht eine Farbe", .0.label())]
    SuitRequired(GameKind),
    #[error("{} hat keine Farbe", .0.label())]
    SuitNotAllowed(GameKind),
    #[error("Auf die Herz-Sau kann man nicht rufen")]
    HeartsAceCalled,
    #[error("Sitzplatz {0} hat mehrfach geklopft")]
    DuplicateKnock(Seat),
    #[error("Kontra muss von einem Gegenspieler kommen")]
    KontraFromOwnTeam,
    #[error("Re muss von der Spielerpartei kommen")]
    ReFromOtherTeam,
    #[error("Re ohne Kontra gibt es nicht")]
    ReWithoutKontra,
    #[error("Tout gibt es nur bei Solospielen (Wenz, Geier, Farbsolo, ...)")]
    ToutNotAllowed,
    #[error("Sie gibt es nur beim Farbsolo und nur zusammen mit Tout")]
    SieNotAllowed,
    #[error("Der verlangte Preis darf nicht negativ sein: {0}")]
    NegativePrice(i64),
    #[error("Augen müssen zwischen 0 und 120 liegen, nicht {0}")]
    InvalidPoints(u8),
    #[error("Beim {} gibt es höchstens {max} Laufende, nicht {got}", .kind.label())]
    InvalidLaufende { kind: GameKind, got: u8, max: u8 },
    #[error("Ein Gegenlauf braucht mindestens eine Laufende")]
    GegenlaufWithoutLaufende,
    #[error("Der Name darf nicht leer sein")]
    NameEmpty,
    #[error("Es gibt schon einen Eintrag namens \"{0}\"")]
    NameTaken(String),
    #[error("Es gibt keinen Eintrag namens \"{0}\"")]
    UnknownEntry(String),
    #[error("Der letzte Eintrag lässt sich nicht löschen")]
    LastEntry,
    #[error("Mindestens eine Spielart muss aktiv sein")]
    NoKinds,
    #[error("Sicherung: {0}")]
    Backup(String),
    #[error("JSON-Fehler: {0}")]
    Json(String),
    #[error("Nicht unterstützte Dateiversion {0}")]
    UnsupportedVersion(u32),
}
