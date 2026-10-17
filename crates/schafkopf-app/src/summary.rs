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
            let side = if input.gegenlauf { " (Gegenlauf)" } else { "" };
            parts.push(format!("{} Laufende{side}", input.laufende));
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
    if input.bock {
        parts.push("Bockspiel".into());
    }
    if input.starts_bock {
        parts.push("löst Bockrunde aus".into());
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

/// Datum wie "17.10.2026" aus Unix-Sekunden.
pub fn german_date(unix_secs: u64) -> String {
    let (year, month, day) = crate::export::date_ymd(unix_secs);
    format!("{day}.{month}.{year}")
}

/// Das Datum, das zu einem Abend gezeigt wird: wann er begonnen wurde, bei alten Dateien ohne diese
/// Angabe der Zeitpunkt, an dem er ins Archiv kam.
pub fn evening_date(table: &Table, archived_at: u64) -> String {
    german_date(table.created_at.map_or(archived_at, |t| t.max(0) as u64))
}

/// "1 Spiel" oder "14 Spiele".
pub fn games_text(count: usize) -> String {
    if count == 1 {
        "1 Spiel".to_owned()
    } else {
        format!("{count} Spiele")
    }
}

/// Die Namen aller Spieler, durch Komma getrennt.
pub fn players_text(table: &Table) -> String {
    table.players.join(", ")
}

/// Wer an diesem Abend vorne lag, mit dem Kontostand: "Anna +8,40 €". Ohne Spiele gibt es keinen.
pub fn leader_text(table: &Table) -> Option<String> {
    if table.games().is_empty() {
        return None;
    }
    let balances = table.balances();
    let (seat, balance) = balances
        .iter()
        .enumerate()
        .max_by_key(|(seat, b)| (**b, std::cmp::Reverse(*seat)))?;
    Some(format!("{} {}", table.players[seat], signed_euro(*balance)))
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
    fn marks_a_gegenlauf() {
        let mut t = table();
        let input = GameInput {
            suit: Some(Suit::Gras),
            partner: Some(3),
            laufende: 4,
            gegenlauf: true,
            ..GameInput::new(0, GameKind::Sauspiel, 1)
        };
        let game = t.add_game(input).unwrap().clone();
        assert_eq!(
            detail(&t, &game),
            "Ben mit Dora gewonnen · 4 Laufende (Gegenlauf)"
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
    fn dates_and_headlines_of_evenings() {
        assert_eq!(german_date(0), "1.1.1970");
        assert_eq!(german_date(1_792_246_327), "17.10.2026");
        let mut t = table();
        assert_eq!(
            evening_date(&t, 951_782_400),
            "29.2.2000",
            "ohne Beginn gilt der Archivzeitpunkt"
        );
        t.created_at = Some(1_000_000_000);
        assert_eq!(evening_date(&t, 951_782_400), "9.9.2001");
        assert_eq!(games_text(1), "1 Spiel");
        assert_eq!(games_text(0), "0 Spiele");
        assert_eq!(games_text(14), "14 Spiele");
        assert_eq!(players_text(&t), "Anna, Ben, Cem, Dora");
    }

    #[test]
    fn leader_of_an_evening() {
        let mut t = table();
        assert_eq!(leader_text(&t), None, "ohne Spiele kein Erster");
        t.add_game(GameInput {
            won: false,
            ..GameInput::new(0, GameKind::Wenz, 2)
        })
        .unwrap();
        // Cem verliert allein dreimal 0,50, die anderen gewinnen je 0,50, Anna steht als Erste.
        assert_eq!(leader_text(&t).as_deref(), Some("Anna +0,50\u{a0}€"));
    }

    #[test]
    fn signs_amounts() {
        assert_eq!(signed_euro(40), "+0,40\u{a0}€");
        assert_eq!(signed_euro(-40), "\u{2212}0,40\u{a0}€");
        assert_eq!(signed_euro(0), "0,00\u{a0}€");
    }
}
