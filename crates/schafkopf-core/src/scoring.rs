use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::{Cents, GameInput, GameKind, Seat, Suit};

/// Tarif eines Tisches in Cent. Die Voreinstellung ist ein verbreiteter 20/50-Tarif,
/// er unterscheidet sich aber von Runde zu Runde und ist deshalb pro Tisch einstellbar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tariff {
    pub sauspiel: Cents,
    pub solo: Cents,
    pub bettel: Cents,
    pub ramsch: Cents,
    /// Aufschlag für Schneider.
    pub schneider: Cents,
    /// Zusätzlicher Aufschlag für Schwarz (zählt zusätzlich zu Schneider).
    pub schwarz: Cents,
    /// Aufschlag pro Laufendem.
    pub laufende: Cents,
    /// Ab wie vielen Laufenden gezählt wird: beim Sauspiel und der Hochzeit, beim Wenz und Geier
    /// (auch mit Farbe) und beim Farbsolo.
    pub laufende_min_sauspiel: u8,
    /// Früher `laufende_min_wenz`: Wenz und Geier zählen gleich, deshalb ein gemeinsames Feld.
    #[serde(alias = "laufende_min_wenz")]
    pub laufende_min_wenz_geier: u8,
    pub laufende_min_solo: u8,
}

impl Default for Tariff {
    fn default() -> Self {
        Tariff {
            sauspiel: 20,
            solo: 50,
            bettel: 50,
            ramsch: 20,
            schneider: 10,
            schwarz: 10,
            laufende: 10,
            laufende_min_sauspiel: 3,
            laufende_min_wenz_geier: 2,
            laufende_min_solo: 3,
        }
    }
}

impl Tariff {
    fn base(&self, kind: GameKind) -> Cents {
        match kind {
            GameKind::Sauspiel | GameKind::Hochzeit => self.sauspiel,
            GameKind::Bettel => self.bettel,
            GameKind::Ramsch => self.ramsch,
            _ => self.solo,
        }
    }

    fn laufende_min(&self, kind: GameKind) -> u8 {
        match kind {
            GameKind::Sauspiel | GameKind::Hochzeit => self.laufende_min_sauspiel,
            GameKind::Wenz | GameKind::Farbwenz | GameKind::Geier | GameKind::Farbgeier => {
                self.laufende_min_wenz_geier
            }
            _ => self.laufende_min_solo,
        }
    }
}

/// Prüft eine Spieleingabe auf Plausibilität. Die Mitspielenden müssen gesetzt sein.
pub fn validate(input: &GameInput, player_count: usize) -> Result<()> {
    let active = &input.participants;
    let check_active = |seat: Seat| -> Result<()> {
        if seat >= player_count {
            Err(Error::UnknownSeat(seat))
        } else if !active.contains(&seat) {
            Err(Error::SeatNotActive(seat))
        } else {
            Ok(())
        }
    };
    if input.dealer >= player_count {
        return Err(Error::UnknownSeat(input.dealer));
    }
    if !(2..=4).contains(&active.len()) {
        return Err(Error::BadParticipantCount(active.len()));
    }
    for (i, &s) in active.iter().enumerate() {
        if s >= player_count {
            return Err(Error::UnknownSeat(s));
        }
        if active[..i].contains(&s) {
            return Err(Error::DuplicateParticipant(s));
        }
    }
    check_active(input.declarer)?;

    let kind = input.kind;
    if kind.is_team_game() && active.len() != 4 {
        return Err(Error::TeamGameNeedsFour(kind));
    }
    match (kind.is_team_game(), input.partner) {
        (true, None) => return Err(Error::PartnerRequired(kind)),
        (false, Some(_)) => return Err(Error::PartnerNotAllowed(kind)),
        (true, Some(p)) => {
            check_active(p)?;
            if p == input.declarer {
                return Err(Error::PartnerIsDeclarer);
            }
        }
        (false, None) => {}
    }

    match (kind, input.suit) {
        (GameKind::Sauspiel, None) => return Err(Error::SuitRequired(kind)),
        (GameKind::Sauspiel, Some(Suit::Herz)) => return Err(Error::HeartsAceCalled),
        (k, None) if k.needs_trump_suit() => return Err(Error::SuitRequired(kind)),
        (GameKind::Sauspiel, Some(_)) => {}
        (k, Some(_)) if !k.needs_trump_suit() => return Err(Error::SuitNotAllowed(kind)),
        _ => {}
    }

    for (i, &k) in input.knockers.iter().enumerate() {
        check_active(k)?;
        if input.knockers[..i].contains(&k) {
            return Err(Error::DuplicateKnock(k));
        }
    }
    if let Some(k) = input.kontra_by {
        check_active(k)?;
        if input.is_on_declarer_team(k) {
            return Err(Error::KontraFromOwnTeam);
        }
    }
    if input.re && input.kontra_by.is_none() {
        return Err(Error::ReWithoutKontra);
    }
    if input.tout && !kind.allows_tout() {
        return Err(Error::ToutNotAllowed);
    }
    if let Some(p) = input.points {
        if p > 120 {
            return Err(Error::InvalidPoints(p));
        }
    }
    if let Some(price) = input.price_override {
        if price < 0 {
            return Err(Error::NegativePrice(price));
        }
    }
    if input.laufende > 8 {
        return Err(Error::InvalidLaufende(input.laufende));
    }
    Ok(())
}

/// Wert des Spiels, den jeder Verlierer an jeden Gewinner zahlt (Einzelwert).
///
/// Das ist der verlangte Preis, falls einer eingetragen wurde, sonst [`calculated_value`].
pub fn game_value(tariff: &Tariff, input: &GameInput) -> Cents {
    input
        .price_override
        .unwrap_or_else(|| calculated_value(tariff, input))
}

/// Der nach Tarif berechnete Einzelwert.
///
/// Grundtarif plus Schneider, Schwarz und Laufende, danach verdoppelt durch
/// Klopfen, Kontra, Re und Tout. Setzt eine gültige Eingabe voraus, siehe [`validate`].
pub fn calculated_value(tariff: &Tariff, input: &GameInput) -> Cents {
    let kind = input.kind;
    let mut value = tariff.base(kind);
    if kind.has_extras() {
        if input.schneider || input.schwarz {
            value += tariff.schneider;
        }
        if input.schwarz {
            value += tariff.schwarz;
        }
        if input.laufende >= tariff.laufende_min(kind) {
            value += Cents::from(input.laufende) * tariff.laufende;
        }
    }
    value << input.doublings()
}

/// Gewinn und Verlust pro Sitzplatz in Cent. Die Summe ist immer null.
///
/// Bei Zweierteams bekommt jeder Gewinner den Einzelwert und jeder Verlierer zahlt ihn.
/// Beim Solo bekommt oder zahlt der Spieler den Einzelwert von bzw. an jeden Gegner.
pub fn payouts(tariff: &Tariff, input: &GameInput, player_count: usize) -> Vec<Cents> {
    let active = &input.participants;
    let value = game_value(tariff, input);
    let sign = if input.won { 1 } else { -1 };
    let mut result = vec![0; player_count];
    let opponents = active
        .iter()
        .filter(|&&s| !input.is_on_declarer_team(s))
        .count() as Cents;
    for &seat in active {
        result[seat] = if !input.is_on_declarer_team(seat) {
            -sign * value
        } else if input.kind.is_team_game() {
            sign * value
        } else {
            sign * value * opponents
        };
    }
    result
}
