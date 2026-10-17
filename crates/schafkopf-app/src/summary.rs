//! Kurze Texte für die Spieleliste.

use schafkopf_charts::format_euro;
use schafkopf_core::{Cents, Game, GameKind, Seat, Table};

/// Spielart, bei Sau und Farbspielen mit Farbe: "Sauspiel Eichel".
pub fn title(game: &Game) -> String {
    let input = &game.input;
    match input.suit {
        Some(suit) => format!("{} {}", input.kind.label(), suit.label()),
        None => input.kind.label().to_owned(),
    }
}

/// Wer gespielt hat, wie es ausging und was dazukam: "Ben mit Dora gewonnen · Schneider · 3 Laufende".
pub fn detail(table: &Table, game: &Game) -> String {
    let input = &game.input;
    let name = |seat: Seat| table.players[seat].as_str();
    let who = match input.partner {
        Some(partner) => format!("{} mit {}", name(input.declarer), name(partner)),
        None => name(input.declarer).to_owned(),
    };
    let outcome = match (input.kind, input.won) {
        (GameKind::Ramsch, false) => "verliert",
        (GameKind::Ramsch, true) => "mit Durchmarsch",
        (_, true) => "gewonnen",
        (_, false) => "verloren",
    };
    let mut parts = vec![format!("{who} {outcome}")];
    if input.kind.has_extras() {
        if input.schwarz {
            parts.push("Schwarz".into());
        } else if input.schneider {
            parts.push("Schneider".into());
        }
        if input.laufende > 0 {
            parts.push(format!("{} Laufende", input.laufende));
        }
    }
    if !input.knockers.is_empty() {
        parts.push(format!("{}× geklopft", input.knockers.len()));
    }
    if input.kontra_by.is_some() {
        parts.push(if input.re { "Kontra, Re" } else { "Kontra" }.into());
    }
    if input.tout {
        parts.push("Tout".into());
    }
    if game.value != game.calculated_value {
        parts.push("Preis korrigiert".into());
    }
    parts.join(" · ")
}

/// Betrag mit Vorzeichen, damit Gewinn und Verlust auf einen Blick zu unterscheiden sind.
pub fn signed_euro(cents: Cents) -> String {
    if cents > 0 {
        format!("+{}", format_euro(cents))
    } else {
        format_euro(cents)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schafkopf_core::{GameInput, Suit, Tariff};

    fn table() -> Table {
        let players = ["Anna", "Ben", "Cem", "Dora"].map(String::from).to_vec();
        Table::new("T", players, 0, Tariff::default()).unwrap()
    }

    #[test]
    fn describes_a_sauspiel_with_extras() {
        let mut t = table();
        let input = GameInput {
            suit: Some(Suit::Eichel),
            partner: Some(3),
            knockers: vec![2],
            kontra_by: Some(0),
            re: true,
            schneider: true,
            laufende: 3,
            price_override: Some(100),
            ..GameInput::new(0, GameKind::Sauspiel, 1)
        };
        let game = t.add_game(input).unwrap().clone();
        assert_eq!(title(&game), "Sauspiel Eichel");
        assert_eq!(
            detail(&t, &game),
            "Ben mit Dora gewonnen · Schneider · 3 Laufende · 1× geklopft · Kontra, Re · Preis korrigiert"
        );
    }

    #[test]
    fn describes_solo_and_ramsch() {
        let mut t = table();
        let wenz = t
            .add_game(GameInput {
                won: false,
                tout: true,
                ..GameInput::new(0, GameKind::Wenz, 2)
            })
            .unwrap()
            .clone();
        assert_eq!(title(&wenz), "Wenz");
        assert_eq!(detail(&t, &wenz), "Cem verloren · Tout");
        let ramsch = t
            .add_game(GameInput {
                won: false,
                ..GameInput::new(1, GameKind::Ramsch, 3)
            })
            .unwrap()
            .clone();
        assert_eq!(detail(&t, &ramsch), "Dora verliert");
    }

    #[test]
    fn signs_amounts() {
        assert_eq!(signed_euro(40), "+0,40\u{a0}€");
        assert_eq!(signed_euro(-40), "\u{2212}0,40\u{a0}€");
        assert_eq!(signed_euro(0), "0,00\u{a0}€");
    }
}
