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
    /// Ab wie vielen Laufenden gezählt wird.
    pub laufende_min_sauspiel: u8,
    pub laufende_min_wenz: u8,
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
            laufende_min_wenz: 2,
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
            GameKind::Wenz | GameKind::Farbwenz => self.laufende_min_wenz,
            _ => self.laufende_min_solo,
        }
    }
}

/// Prüft eine Spieleingabe auf Plausibilität. `active` sind die Sitzplätze, die mitspielen.
pub fn validate(input: &GameInput, player_count: usize, active: &[Seat]) -> Result<()> {
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
    check_active(input.declarer)?;

    let kind = input.kind;
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
    if input.laufende > 8 {
        return Err(Error::InvalidLaufende(input.laufende));
    }
    Ok(())
}

/// Wert des Spiels, den jeder Verlierer an jeden Gewinner zahlt (Einzelwert).
///
/// Grundtarif plus Schneider, Schwarz und Laufende, danach verdoppelt durch
/// Klopfen, Kontra, Re und Tout. Setzt eine gültige Eingabe voraus, siehe [`validate`].
pub fn game_value(tariff: &Tariff, input: &GameInput) -> Cents {
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
pub fn payouts(
    tariff: &Tariff,
    input: &GameInput,
    player_count: usize,
    active: &[Seat],
) -> Vec<Cents> {
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
