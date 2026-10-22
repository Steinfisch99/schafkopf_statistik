use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::model::{Cents, GameInput, GameKind, Seat, Suit, MAX_BOCK};

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
    /// Verdoppelt Klopfen beim Ramsch den Preis? Das Klopfen wird in jedem Fall erfasst. Alte
    /// Tarife ohne diese Angabe verhalten sich wie bisher, das Klopfen zählt.
    #[serde(default = "yes")]
    pub ramsch_knock_doubles: bool,
    /// Verdoppelt ein Bockspiel den Ramsch? Alte Tarife ohne diese Angabe: ja.
    #[serde(default = "yes")]
    pub ramsch_bock_doubles: bool,
    /// Dürfen die Verlierer zu viel verlangtes Geld zurückverlangen, wenn es ihnen auffällt? Alte
    /// Tarife ohne diese Angabe: ja.
    #[serde(default = "yes")]
    pub reclaim_enabled: bool,
    /// Gibt es Strafzahlungen für Vergehen und Neumischen? Aus: Vergehen werden trotzdem erfasst und
    /// gezählt, kosten aber nichts. Alte Tarife ohne diese Angabe: ja.
    #[serde(default = "yes")]
    pub penalties_enabled: bool,
    /// Strafe für ein Vergehen (nicht bedient, aus der Reihe gespielt, ...), die der Täter jedem
    /// anderen am Tisch zahlt.
    #[serde(default = "default_offence_penalty")]
    pub offence_penalty: Cents,
    /// Strafe fürs Neumischen nach einem Fehler beim Geben, die der Geber jedem Mitspieler der Runde
    /// zahlt. 0: Neumischen kostet nichts.
    #[serde(default)]
    pub misdeal_penalty: Cents,
    /// Was jeder in den Stock legt, wenn keiner spielt (nur bei der Regel "Stock"), in Cent.
    #[serde(default = "default_stock_amount")]
    pub stock_amount: Cents,
    /// Muss die Spielerpartei den Stock doppeln, wenn sie ein Sauspiel verliert? Aus: Verliert sie,
    /// bleibt der Stock, wie er ist. Alte Tarife ohne diese Angabe: ja.
    #[serde(default = "yes")]
    pub stock_doubling: bool,
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
            ramsch_knock_doubles: true,
            ramsch_bock_doubles: true,
            reclaim_enabled: true,
            penalties_enabled: true,
            offence_penalty: default_offence_penalty(),
            misdeal_penalty: 0,
            stock_amount: default_stock_amount(),
            stock_doubling: true,
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

    /// Ab wie vielen Laufenden in dieser Spielart gezählt wird, also ab wann es einen Lauf oder
    /// Gegenlauf gibt.
    pub fn laufende_min(&self, kind: GameKind) -> u8 {
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
    for extra in [input.bock_extra, input.extra_bock_rounds] {
        if extra >= MAX_BOCK {
            return Err(Error::TooManyBock(extra + 1));
        }
    }
    check_active(input.declarer)?;
    for &by in input.price_error_by.iter().chain(&input.reclaimed_by) {
        check_active(by)?;
    }

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
    if let Some(r) = input.re_by {
        check_active(r)?;
        if !input.re || !input.is_on_declarer_team(r) {
            return Err(Error::ReFromOtherTeam);
        }
    }
    if input.re && input.kontra_by.is_none() {
        return Err(Error::ReWithoutKontra);
    }
    if input.tout && !kind.allows_tout() {
        return Err(Error::ToutNotAllowed);
    }
    if input.sie && !(input.tout && kind == GameKind::Farbsolo) {
        return Err(Error::SieNotAllowed);
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
    let max_laufende = kind.max_laufende(input.gegenlauf);
    if input.laufende > max_laufende {
        return Err(Error::InvalidLaufende {
            kind,
            got: input.laufende,
            max: max_laufende,
        });
    }
    if input.gegenlauf && input.laufende == 0 {
        return Err(Error::GegenlaufWithoutLaufende);
    }
    Ok(())
}

/// Wert des Spiels, den jeder Verlierer an jeden Gewinner zahlt (Einzelwert).
///
/// Das ist der verlangte Preis, falls einer eingetragen wurde, sonst [`calculated_value`]. Wurde zu
/// viel Verlangtes zurückverlangt, gilt [`reclaimed_value`].
pub fn game_value(tariff: &Tariff, input: &GameInput) -> Cents {
    let calculated = calculated_value(tariff, input);
    match input.price_override {
        Some(demanded) if input.reclaimed => reclaimed_value(calculated, demanded),
        Some(demanded) => demanded,
        None => calculated,
    }
}

/// Der Einzelwert, wenn die Verlierer zu viel Verlangtes zurückverlangen: Sie dürfen das Doppelte des
/// Mehrbetrags zurückverlangen und tun es immer in voller Höhe. Kostet das Spiel 60 und wurden 80
/// verlangt, ist der Mehrbetrag 20, sie holen 40 zurück, und es bleiben 40. Weniger als 0 wird es nie.
pub fn reclaimed_value(calculated: Cents, demanded: Cents) -> Cents {
    let extra = (demanded - calculated).max(0);
    (demanded - 2 * extra).max(0)
}

/// Der nach Tarif berechnete Einzelwert.
///
/// Grundtarif plus Schneider, Schwarz und Laufende, danach verdoppelt durch
/// Klopfen, Kontra, Re und Tout. Beim Tout zählen Schneider und Schwarz nicht zum Grundpreis.
/// Setzt eine gültige Eingabe voraus, siehe [`validate`].
pub fn calculated_value(tariff: &Tariff, input: &GameInput) -> Cents {
    let kind = input.kind;
    let mut value = tariff.base(kind);
    if kind.has_extras() {
        // Beim Tout muss der Spieler alle Stiche machen: Schneider und Schwarz sind darin schon
        // enthalten und kommen nicht zum Preis. Sie bleiben im Spiel (für die Statistik).
        if !input.tout {
            if input.schneider || input.schwarz {
                value += tariff.schneider;
            }
            if input.schwarz {
                value += tariff.schwarz;
            }
        }
        if input.laufende >= tariff.laufende_min(kind) {
            value += Cents::from(input.laufende) * tariff.laufende;
        }
    }
    value << price_doublings(tariff, input)
}

/// Wie oft der Preis des Spiels nach dem Tarif verdoppelt wird: Klopfen, Kontra, Re, Tout und Bock.
/// Beim Ramsch zählen Klopfen und Bock nur, wenn der Tarif es sagt.
pub fn price_doublings(tariff: &Tariff, input: &GameInput) -> u32 {
    let mut doublings = input.doublings();
    if input.kind == GameKind::Ramsch && !tariff.ramsch_knock_doubles {
        doublings -= input.knockers.len() as u32;
    }
    if input.kind == GameKind::Ramsch && !tariff.ramsch_bock_doubles {
        doublings -= input.bock_doublings();
    }
    doublings
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

fn yes() -> bool {
    true
}

fn default_stock_amount() -> Cents {
    20
}

fn default_offence_penalty() -> Cents {
    50
}
