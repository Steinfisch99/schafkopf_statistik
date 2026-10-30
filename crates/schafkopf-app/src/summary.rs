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
    if input.sie {
        parts.push("Tout, Sie".into());
    } else if input.tout {
        parts.push("Tout".into());
    }
    if input.bock {
        parts.push(match input.bock_doublings() {
            1 => "Bockspiel".to_owned(),
            n => format!("Bockspiel (Preis mal {})", 1u32 << n),
        });
    }
    match input.bock_triggers() {
        0 => {}
        1 => parts.push("löst Bockrunde aus".into()),
        n => parts.push(format!("löst {n} Bockrunden aus")),
    }
    if game.demanded_wrong() {
        let mut text = if game.input.reclaimed {
            let names: Vec<&str> = input
                .reclaimed_by
                .iter()
                .map(|&s| table.players[s].as_str())
                .collect();
            format!(
                "Zu viel verlangt, zurückverlangt von {}",
                names.join(" und ")
            )
        } else if game.overcharged() {
            "Zu viel verlangt, nicht aufgefallen".to_owned()
        } else {
            "Zu wenig verlangt".to_owned()
        };
        if !input.price_error_by.is_empty() {
            let names: Vec<&str> = input
                .price_error_by
                .iter()
                .map(|&s| table.players[s].as_str())
                .collect();
            text.push_str(&format!(", falsch verlangt von {}", names.join(" und ")));
        }
        if !input.mistakes.is_empty() {
            let kinds: Vec<&str> = input.mistakes.iter().map(|m| m.label()).collect();
            text.push_str(&format!(" ({})", kinds.join(", ")));
        }
        parts.push(text);
    }
    parts.join(" · ")
}

/// Kompakte Prüfzeilen (Bezeichnung, Wert) für die Vorschau vor dem Speichern:
/// Spiel, Spieler, Mitspieler, Ausgang, Schneider/Schwarz, Lauf.
pub fn check_rows(table: &Table, game: &Game) -> Vec<(&'static str, String)> {
    let input = &game.input;
    let name = |seat: Seat| table.players[seat].clone();
    let mut rows = vec![("Spiel", title(game))];
    let declarer_label = if input.kind == GameKind::Ramsch {
        "Verlierer"
    } else {
        "Spieler"
    };
    rows.push((declarer_label, name(input.declarer)));
    if let Some(partner) = input.partner {
        rows.push(("Mitspieler", name(partner)));
    }
    let outcome = match (input.kind, input.won) {
        (GameKind::Ramsch, false) => "verliert",
        (GameKind::Ramsch, true) => "Durchmarsch",
        (_, true) => "gewonnen",
        (_, false) => "verloren",
    };
    rows.push(("Ausgang", outcome.to_owned()));
    if input.kind.has_extras() {
        let sn = if input.schwarz {
            "Schwarz"
        } else if input.schneider {
            "Schneider"
        } else {
            "nein"
        };
        rows.push(("Schneider/Schwarz", sn.to_owned()));
        let lauf = match (input.laufende, input.gegenlauf) {
            (0, _) => "nein".to_owned(),
            (n, true) => format!("{n} (Gegenlauf)"),
            (n, false) => format!("{n}"),
        };
        rows.push(("Lauf", lauf));
    }
    rows
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

/// Wann ein Abend lief: "19:30 bis 23:10 Uhr (3 h 40 min)", noch offen "seit 19:30 Uhr". `None`,
/// wenn der Beginn nicht bekannt ist (alte Dateien).
pub fn evening_times(table: &Table) -> Option<String> {
    let offset = table.utc_offset_min;
    let start = table.created_at?;
    let begin = schafkopf_core::format_clock(start, offset);
    Some(match table.ended_at {
        Some(end) => format!(
            "{begin} bis {} Uhr ({})",
            schafkopf_core::format_clock(end, offset),
            schafkopf_core::format_duration(end - start)
        ),
        None => format!("seit {begin} Uhr"),
    })
}

/// Der Satz zu einem Eintrag im Protokoll: "21:05 · Eva kommt dazu (hinter Anna, nach Spiel 4)".
pub fn event_text(table: &Table, event: &schafkopf_core::TableEvent) -> String {
    let name = |seat: usize| table.players.get(seat).map_or("?", String::as_str);
    let time = event
        .at
        .map(|t| {
            format!(
                "{} · ",
                schafkopf_core::format_clock(t, table.utc_offset_min)
            )
        })
        .unwrap_or_default();
    let when = match event.after_games {
        0 => "vor dem ersten Spiel".to_owned(),
        n => format!("nach Spiel {n}"),
    };
    let behind = event
        .behind
        .map(|b| format!("hinter {}, ", name(b)))
        .unwrap_or_default();
    format!(
        "{time}{} {} ({behind}{when})",
        name(event.seat),
        match event.kind {
            schafkopf_core::EventKind::Joined => "kommt dazu",
            schafkopf_core::EventKind::Left => "geht",
            schafkopf_core::EventKind::Returned => "kommt zurück",
            schafkopf_core::EventKind::Moved => "setzt sich um",
        },
    )
}

/// Der Satz zu einem Vergehen: "21:05 · Cem: Nicht bedient (nach Spiel 4), zahlt 0,50 € an jeden".
pub fn offence_text(table: &Table, offence: &schafkopf_core::Offence) -> String {
    let who = table
        .players
        .get(offence.offender)
        .map_or("?", String::as_str);
    let time = offence
        .at
        .map(|t| {
            format!(
                "{} · ",
                schafkopf_core::format_clock(t, table.utc_offset_min)
            )
        })
        .unwrap_or_default();
    let when = match offence.after_games {
        0 => "vor dem ersten Spiel".to_owned(),
        n => format!("nach Spiel {n}"),
    };
    let cost = if offence.amount > 0 {
        format!(", zahlt {} an jeden", format_euro(offence.amount))
    } else {
        ", ohne Strafe".to_owned()
    };
    format!("{time}{who}: {} ({when}{cost})", offence.kind.label())
}

/// Der Satz zu einer Bewegung am Stock: "21:05 · Keiner spielt: alle legen ein, im Stock 0,80 € (nach Spiel 4)" oder
/// "Stock ausgezahlt: Ben +0,40 €, Dora +0,40 € (nach Spiel 5)".
pub fn stock_text(table: &Table, m: &schafkopf_core::StockMove) -> String {
    let time =
        m.at.map(|t| {
            format!(
                "{} · ",
                schafkopf_core::format_clock(t, table.utc_offset_min)
            )
        })
        .unwrap_or_default();
    let when = match m.after_games {
        0 => "vor dem ersten Spiel".to_owned(),
        n => format!("nach Spiel {n}"),
    };
    let detail = match m.kind {
        schafkopf_core::StockMoveKind::NoGame if m.pot_change == 0 => "zusammengeworfen".to_owned(),
        schafkopf_core::StockMoveKind::NoGame => {
            format!("alle legen ein, im Stock {}", format_euro(m.pot_change))
        }
        _ => table
            .players
            .iter()
            .zip(&m.payouts)
            .filter(|(_, &p)| p != 0)
            .map(|(name, &p)| format!("{name} {}", signed_euro(p)))
            .collect::<Vec<_>>()
            .join(", "),
    };
    format!("{time}{}: {detail} ({when})", m.kind.label())
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
            "Ben mit Dora gewonnen · Schneider · 3 Laufende · 1× geklopft · Kontra, Re · Zu wenig verlangt, falsch verlangt von Ben und Dora"
        );
    }

    #[test]
    fn check_rows_list_player_partner_and_extras() {
        let mut t = table();
        let input = GameInput {
            suit: Some(Suit::Eichel),
            partner: Some(3),
            schneider: true,
            laufende: 3,
            ..GameInput::new(0, GameKind::Sauspiel, 1)
        };
        let game = t.add_game(input).unwrap().clone();
        let rows = check_rows(&t, &game);
        let get = |k: &str| rows.iter().find(|r| r.0 == k).map(|r| r.1.as_str());
        assert_eq!(get("Spiel"), Some("Sauspiel Eichel"));
        assert_eq!(get("Spieler"), Some("Ben"));
        assert_eq!(get("Mitspieler"), Some("Dora"));
        assert_eq!(get("Schneider/Schwarz"), Some("Schneider"));
        assert_eq!(get("Lauf"), Some("3"));
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

    #[test]
    fn shows_when_an_evening_ran() {
        let mut t = table();
        assert_eq!(
            evening_times(&t),
            None,
            "alte Dateien kennen den Beginn nicht"
        );
        t.created_at = Some(1_000_000_000);
        t.utc_offset_min = 120;
        assert_eq!(evening_times(&t).as_deref(), Some("seit 03:46 Uhr"));
        t.finish(1_000_000_000 + 3 * 3600 + 40 * 60);
        assert_eq!(
            evening_times(&t).as_deref(),
            Some("03:46 bis 07:26 Uhr (3 h 40 min)")
        );
    }

    #[test]
    fn describes_what_happened_at_the_table() {
        let mut t = table();
        t.utc_offset_min = 60;
        t.add_player("Eva", Some(0), Some(1_000_000_000)).unwrap();
        t.leave(1, None).unwrap();
        assert_eq!(
            event_text(&t, &t.events[0]),
            "02:46 · Eva kommt dazu (hinter Anna, vor dem ersten Spiel)"
        );
        assert_eq!(
            event_text(&t, &t.events[1]),
            "Ben geht (vor dem ersten Spiel)"
        );
    }

    #[test]
    fn describes_bock_rounds_and_levels() {
        let mut t = table();
        t.bock_mode = schafkopf_core::BockMode::NextGame;
        let input = GameInput {
            suit: Some(Suit::Eichel),
            partner: Some(3),
            bock: true,
            bock_extra: 1,
            starts_bock: true,
            extra_bock_rounds: 1,
            ..GameInput::new(0, schafkopf_core::GameKind::Sauspiel, 1)
        };
        let game = t.add_game(input).unwrap().clone();
        let text = detail(&t, &game);
        assert!(text.contains("Bockspiel (Preis mal 4)"), "{text}");
        assert!(text.contains("löst 2 Bockrunden aus"), "{text}");
    }

    #[test]
    fn describes_too_much_demanded_with_and_without_a_reclaim() {
        let mut t = table();
        let base = GameInput {
            suit: Some(Suit::Eichel),
            partner: Some(3),
            price_override: Some(300),
            ..GameInput::new(0, schafkopf_core::GameKind::Sauspiel, 1)
        };
        let g = t.add_game(base.clone()).unwrap().clone();
        let text = detail(&t, &g);
        assert!(
            text.contains("Zu viel verlangt, nicht aufgefallen, falsch verlangt von Ben und Dora"),
            "{text}"
        );
        let g = t
            .add_game(GameInput {
                reclaimed: true,
                reclaimed_by: vec![2],
                ..base
            })
            .unwrap()
            .clone();
        let text = detail(&t, &g);
        assert!(
            text.contains("Zu viel verlangt, zurückverlangt von Cem"),
            "{text}"
        );
    }
}

#[cfg(test)]
mod offence_tests {
    use super::*;
    use schafkopf_core::{OffenceKind, Tariff};

    #[test]
    fn an_offence_reads_as_one_sentence() {
        let players = ["Anna", "Ben", "Cem", "Dora"].map(String::from).to_vec();
        let mut t = Table::new("T", players, 0, Tariff::default()).unwrap();
        t.add_offence(OffenceKind::NotFollowed, 2, None, None, None)
            .unwrap();
        t.add_offence(OffenceKind::Misdeal, 1, None, None, None)
            .unwrap();
        assert_eq!(
            offence_text(&t, &t.offences[0]),
            "Cem: Nicht bedient (vor dem ersten Spiel, zahlt 0,50\u{a0}€ an jeden)"
        );
        assert_eq!(
            offence_text(&t, &t.offences[1]),
            "Ben: Falsch gegeben (neu gemischt) (vor dem ersten Spiel, ohne Strafe)"
        );
    }
}
