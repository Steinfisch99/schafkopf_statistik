use crate::*;

fn names() -> Vec<String> {
    ["Anna", "Ben", "Cem", "Dora"].map(String::from).to_vec()
}

fn table() -> Table {
    Table::new("Stammtisch", names(), 0, Tariff::default()).unwrap()
}

fn sauspiel(dealer: Seat, declarer: Seat, partner: Seat) -> GameInput {
    GameInput {
        suit: Some(Suit::Eichel),
        partner: Some(partner),
        ..GameInput::new(dealer, GameKind::Sauspiel, declarer)
    }
}

#[test]
fn sauspiel_won_basic() {
    let mut t = table();
    let g = t.add_game(sauspiel(0, 1, 3)).unwrap();
    assert_eq!(g.value, 20);
    assert_eq!(g.payouts, vec![-20, 20, -20, 20]);
}

#[test]
fn sauspiel_lost_with_laufende_schneider_and_knock() {
    let mut t = table();
    let input = GameInput {
        won: false,
        schneider: true,
        laufende: 3,
        knockers: vec![2],
        ..sauspiel(0, 1, 3)
    };
    // (20 + 10 Schneider + 3*10 Laufende) * 2 für Klopfen
    let g = t.add_game(input).unwrap();
    assert_eq!(g.value, 120);
    assert_eq!(g.payouts, vec![120, -120, 120, -120]);
}

#[test]
fn laufende_below_minimum_do_not_count() {
    let t = Tariff::default();
    let sau = GameInput {
        laufende: 2,
        ..sauspiel(0, 1, 3)
    };
    assert_eq!(game_value(&t, &sau), 20);
    let wenz = GameInput {
        laufende: 2,
        ..GameInput::new(0, GameKind::Wenz, 1)
    };
    assert_eq!(game_value(&t, &wenz), 50 + 20);
}

#[test]
fn solo_pays_three_times() {
    let mut t = table();
    let input = GameInput {
        suit: Some(Suit::Gras),
        schneider: true,
        schwarz: true,
        ..GameInput::new(3, GameKind::Farbsolo, 0)
    };
    // 50 + 10 Schneider + 10 Schwarz
    let g = t.add_game(input).unwrap();
    assert_eq!(g.value, 70);
    assert_eq!(g.payouts, vec![210, -70, -70, -70]);
}

#[test]
fn kontra_re_and_tout_double() {
    let t = Tariff::default();
    let input = GameInput {
        kontra_by: Some(2),
        re: true,
        tout: true,
        ..GameInput::new(0, GameKind::Wenz, 1)
    };
    assert_eq!(game_value(&t, &input), 50 * 8);
}

#[test]
fn five_players_dealer_sits_out() {
    let mut players = names();
    players.push("Emil".into());
    let mut t = Table::new("Fünf", players, 4, Tariff::default()).unwrap();
    // Geber 4, davor 3 (hebt ab), danach 0 und 1; Platz 2 setzt aus.
    assert_eq!(t.default_participants(4), vec![0, 1, 3, 4]);
    let g = t.add_game(sauspiel(4, 0, 1)).unwrap();
    assert_eq!(g.payouts, vec![20, 20, 0, -20, -20]);
    assert_eq!(t.next_dealer(), 0);
    // Wer aussetzt, kann nicht spielen: bei Geber 0 setzt Platz 3 aus.
    let err = t.add_game(sauspiel(0, 3, 1)).unwrap_err();
    assert_eq!(err, Error::SeatNotActive(3));
}

#[test]
fn dealer_rotates() {
    let mut t = Table::new("T", names(), 2, Tariff::default()).unwrap();
    assert_eq!(t.next_dealer(), 2);
    t.add_game(sauspiel(2, 0, 1)).unwrap();
    assert_eq!(t.next_dealer(), 3);
    t.add_game(sauspiel(3, 0, 1)).unwrap();
    assert_eq!(t.next_dealer(), 0);
}

#[test]
fn validation_errors() {
    let mut t = table();
    let mut input = sauspiel(0, 1, 1);
    assert_eq!(
        t.add_game(input.clone()).unwrap_err(),
        Error::PartnerIsDeclarer
    );
    input.partner = None;
    assert_eq!(
        t.add_game(input.clone()).unwrap_err(),
        Error::PartnerRequired(GameKind::Sauspiel)
    );
    input.partner = Some(2);
    input.suit = Some(Suit::Herz);
    assert_eq!(
        t.add_game(input.clone()).unwrap_err(),
        Error::HeartsAceCalled
    );
    input.suit = Some(Suit::Gras);
    input.kontra_by = Some(2);
    assert_eq!(
        t.add_game(input.clone()).unwrap_err(),
        Error::KontraFromOwnTeam
    );
    input.kontra_by = None;
    input.re = true;
    assert_eq!(
        t.add_game(input.clone()).unwrap_err(),
        Error::ReWithoutKontra
    );
    input.re = false;
    input.knockers = vec![3, 3];
    assert_eq!(t.add_game(input).unwrap_err(), Error::DuplicateKnock(3));
    assert_eq!(
        t.add_game(GameInput::new(0, GameKind::Farbsolo, 1))
            .unwrap_err(),
        Error::SuitRequired(GameKind::Farbsolo)
    );
    assert_eq!(
        t.add_game(GameInput {
            tout: true,
            ..sauspiel(0, 1, 2)
        })
        .unwrap_err(),
        Error::ToutNotAllowed
    );
    assert!(t.games().is_empty());
}

#[test]
fn table_validation() {
    let t = |p: Vec<&str>| {
        Table::new(
            "x",
            p.into_iter().map(String::from).collect(),
            0,
            Tariff::default(),
        )
    };
    assert_eq!(t(vec!["a"]).unwrap_err(), Error::PlayerCount(1));
    let eleven: Vec<String> = (0..11).map(|i| i.to_string()).collect();
    assert_eq!(
        Table::new("x", eleven, 0, Tariff::default()).unwrap_err(),
        Error::PlayerCount(11)
    );
    assert_eq!(t(vec!["a", "b", "c", " "]).unwrap_err(), Error::EmptyName);
    assert_eq!(
        t(vec!["a", "b", "c", "a"]).unwrap_err(),
        Error::DuplicateName("a".into())
    );
}

#[test]
fn payouts_always_sum_to_zero() {
    let tariff = Tariff::default();
    let active = [0, 1, 2, 3];
    for kind in GameKind::ALL {
        for won in [true, false] {
            for doublings in 0..3 {
                let input = GameInput {
                    won,
                    schneider: true,
                    laufende: 4,
                    knockers: (0..doublings).collect(),
                    participants: active.to_vec(),
                    partner: kind.is_team_game().then_some(2),
                    ..GameInput::new(3, kind, 1)
                };
                let p = payouts(&tariff, &input, 4);
                assert_eq!(p.iter().sum::<i64>(), 0, "{kind:?} won={won}");
            }
        }
    }
}

#[test]
fn points_to_outcome() {
    assert_eq!(
        outcome_from_points(60),
        Outcome {
            won: false,
            schneider: false
        }
    );
    assert_eq!(
        outcome_from_points(61),
        Outcome {
            won: true,
            schneider: false
        }
    );
    assert_eq!(
        outcome_from_points(89),
        Outcome {
            won: true,
            schneider: false
        }
    );
    assert_eq!(
        outcome_from_points(90),
        Outcome {
            won: true,
            schneider: true
        }
    );
    assert_eq!(
        outcome_from_points(31),
        Outcome {
            won: false,
            schneider: false
        }
    );
    assert_eq!(
        outcome_from_points(30),
        Outcome {
            won: false,
            schneider: true
        }
    );
}

#[test]
fn undo_and_json_roundtrip() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 2)).unwrap();
    t.add_game(GameInput {
        note: Some("Ohne Trumpf!".into()),
        ..sauspiel(1, 0, 3)
    })
    .unwrap();
    let back = Table::from_json(&t.to_json().unwrap()).unwrap();
    assert_eq!(back, t);
    assert!(t.undo_last().is_some());
    assert_eq!(t.games().len(), 1);
    assert_eq!(t.balances(), vec![-20, 20, 20, -20]);
    assert!(matches!(Table::from_json("{"), Err(Error::Json(_))));
}

#[test]
fn stats_roles_streaks_and_matrix() {
    let mut t = table();
    // Spiel 1: Ben (Mitspieler Dora) gewinnt, Cem klopft und verliert.
    t.add_game(GameInput {
        knockers: vec![2],
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    // Spiel 2: Ben verliert mit Dora.
    t.add_game(GameInput {
        won: false,
        ..sauspiel(1, 1, 3)
    })
    .unwrap();
    // Spiel 3: Anna spielt Wenz und gewinnt.
    t.add_game(GameInput::new(2, GameKind::Wenz, 0)).unwrap();

    let s = table_stats(&t);
    assert_eq!(s.games, 3);
    assert_eq!(s.knocked_games, 1);
    assert_eq!(s.by_kind[&GameKind::Sauspiel].games, 2);
    assert_eq!(
        s.by_called_ace[&Suit::Eichel],
        RoleStats { games: 2, wins: 1 }
    );
    assert_eq!(s.partner_matrix[1][3], RoleStats { games: 2, wins: 1 });
    assert_eq!(s.partner_matrix[3][1], s.partner_matrix[1][3]);
    assert_eq!(s.most_expensive, Some((2, 50)));

    let ben = &s.players[1];
    assert_eq!(ben.as_declarer, RoleStats { games: 2, wins: 1 });
    assert_eq!(ben.current_streak, -2);
    assert_eq!(ben.longest_loss_streak, 2);
    assert_eq!(ben.longest_win_streak, 1);
    assert_eq!(ben.balance, 40 - 20 + -50);

    let cem = &s.players[2];
    assert_eq!(cem.knocks, RoleStats { games: 1, wins: 0 });
    assert_eq!(cem.as_defender.games, 3);

    assert_eq!(s.players.iter().map(|p| p.balance).sum::<i64>(), 0);
    assert_eq!(
        s.players.iter().map(|p| p.balance).collect::<Vec<_>>(),
        t.balances()
    );
    assert_eq!(s.players[0].games_dealt, 1);
    // Ben saß in Spiel 1 (Geber Anna) auf Platz 0 = Vorhand und hat gewonnen.
    assert_eq!(ben.by_position[0], RoleStats { games: 1, wins: 1 });

    let history = balance_history(&t);
    assert_eq!(history.len(), 4);
    assert_eq!(history[0], vec![0, 0, 0, 0]);
    assert_eq!(history.last().unwrap(), &t.balances());
}

fn table_of(n: usize) -> Table {
    let players = (0..n).map(|i| format!("Spieler {i}")).collect();
    Table::new("T", players, 0, Tariff::default()).unwrap()
}

#[test]
fn every_player_count_from_two_to_ten_works() {
    for n in 2..=10 {
        let mut t = table_of(n);
        // Wenz geht immer; Sauspiel nur mit genau vier Mitspielenden.
        for _ in 0..n * 2 {
            let dealer = t.next_dealer();
            let declarer = t.default_participants(dealer)[0];
            t.add_game(GameInput::new(dealer, GameKind::Wenz, declarer))
                .unwrap();
            let sau = GameInput {
                suit: Some(Suit::Gras),
                partner: t.default_participants(dealer).get(2).copied(),
                ..GameInput::new(dealer, GameKind::Sauspiel, declarer)
            };
            let res = t.add_game(sau).map(|g| g.payouts.clone());
            if n >= 4 {
                assert_eq!(res.unwrap().iter().sum::<i64>(), 0, "n={n}");
            } else {
                assert!(res.is_err(), "n={n}");
            }
        }
        assert_eq!(t.balances().iter().sum::<i64>(), 0, "n={n}");
        let stats = table_stats(&t);
        assert_eq!(stats.players.len(), n);
        assert_eq!(stats.players.iter().map(|p| p.balance).sum::<i64>(), 0);
    }
}

#[test]
fn two_players_solo_is_one_against_one() {
    let mut t = table_of(2);
    let g = t.add_game(GameInput::new(0, GameKind::Wenz, 1)).unwrap();
    assert_eq!(g.input.participants, vec![1, 0]);
    assert_eq!(g.payouts, vec![-50, 50]);
}

#[test]
fn three_players_solo_and_no_team_game() {
    let mut t = table_of(3);
    let g = t.add_game(GameInput::new(1, GameKind::Wenz, 2)).unwrap();
    assert_eq!(g.payouts, vec![-50, -50, 100]);
    let sau = GameInput {
        suit: Some(Suit::Gras),
        partner: Some(0),
        ..GameInput::new(1, GameKind::Sauspiel, 2)
    };
    assert_eq!(
        t.add_game(sau).unwrap_err(),
        Error::TeamGameNeedsFour(GameKind::Sauspiel)
    );
}

#[test]
fn large_tables_rotate_who_sits_out() {
    let mut t = table_of(10);
    assert_eq!(t.default_participants(0), vec![1, 2, 9, 0]);
    assert_eq!(t.default_participants(8), vec![9, 0, 7, 8]);
    let mut played = [0u32; 10];
    for _ in 0..10 {
        let dealer = t.next_dealer();
        let g = t
            .add_game(GameInput::new(dealer, GameKind::Wenz, (dealer + 1) % 10))
            .unwrap();
        for &s in &g.input.participants {
            played[s] += 1;
        }
        let mut seats = g.input.participants.clone();
        seats.sort();
        let mut expected = vec![
            dealer,
            (dealer + 1) % 10,
            (dealer + 2) % 10,
            (dealer + 9) % 10,
        ];
        expected.sort();
        assert_eq!(seats, expected);
    }
    assert!(played.iter().all(|&c| c == 4));
    let stats = table_stats(&t);
    assert!(stats
        .players
        .iter()
        .all(|p| p.games_played == 4 && p.games_sat_out == 6));
}

#[test]
fn explicit_participants_are_validated() {
    let mut t = table_of(6);
    let with = |participants: Vec<Seat>| GameInput {
        participants,
        ..GameInput::new(0, GameKind::Wenz, 1)
    };
    assert_eq!(
        t.add_game(with(vec![1, 2, 3, 4, 5])).unwrap_err(),
        Error::BadParticipantCount(5)
    );
    assert_eq!(
        t.add_game(with(vec![1, 2, 2])).unwrap_err(),
        Error::DuplicateParticipant(2)
    );
    assert_eq!(
        t.add_game(with(vec![1, 2, 9])).unwrap_err(),
        Error::UnknownSeat(9)
    );
    assert!(t.add_game(with(vec![1, 3, 5])).is_ok());
}

#[test]
fn demanded_price_replaces_the_calculation() {
    let mut t = table();
    let g = t
        .add_game(GameInput {
            price_override: Some(30),
            ..sauspiel(0, 1, 3)
        })
        .unwrap();
    assert_eq!(g.value, 30);
    assert_eq!(g.calculated_value, 20);
    assert_eq!(g.payouts, vec![-30, 30, -30, 30]);
    assert_eq!(table_stats(&t).corrected_games, 1);
    assert_eq!(
        t.add_game(GameInput {
            price_override: Some(-1),
            ..sauspiel(1, 1, 3)
        })
        .unwrap_err(),
        Error::NegativePrice(-1)
    );
}

#[test]
fn changing_the_tariff_keeps_old_games() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.set_tariff(Tariff {
        sauspiel: 50,
        ..Tariff::default()
    });
    t.add_game(sauspiel(1, 1, 3)).unwrap();
    assert_eq!(t.games()[0].value, 20);
    assert_eq!(t.games()[1].value, 50);
    assert_eq!(t.balances(), vec![-70, 70, -70, 70]);
    let back = Table::from_json(&t.to_json().unwrap()).unwrap();
    assert_eq!(back.tariff.sauspiel, 50);
}

#[test]
fn dealer_and_the_player_before_play_with_the_two_after() {
    let t = table();
    assert_eq!(t.default_participants(2), vec![3, 0, 1, 2]);
}

#[test]
fn csv_has_header_and_one_row_per_game() {
    let mut t = Table::new(
        "T",
        vec![
            "Anna".into(),
            "Ben, der Alte".into(),
            "Cem".into(),
            "Dora".into(),
        ],
        0,
        Tariff::default(),
    )
    .unwrap();
    t.add_game(GameInput {
        knockers: vec![2],
        note: Some("sagt \"Wenz\"".into()),
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    t.add_game(GameInput {
        won: false,
        ..sauspiel(1, 1, 3)
    })
    .unwrap();
    let csv = t.to_csv();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 3);
    assert!(lines[0].starts_with("Nr,Zeit,Geber,Spielart,"));
    assert!(lines[0].ends_with("\"Ben, der Alte Gewinn_Cent\",\"Ben, der Alte Stand_Cent\",Cem Gewinn_Cent,Cem Stand_Cent,Dora Gewinn_Cent,Dora Stand_Cent"));
    assert_eq!(t.csv_row(0).unwrap(), lines[1]);
    assert!(t.csv_row(2).is_none());
    // Spiel 1: Klopfen verdoppelt 20 auf 40; Ben und Dora gewinnen.
    assert!(lines[1].starts_with("1,,Anna,Sauspiel,Eichel,\"Ben, der Alte\",Dora,"));
    assert!(lines[1].contains("\"sagt \"\"Wenz\"\"\""));
    assert!(lines[1].ends_with("-40,-40,40,40,-40,-40,40,40"));
    // Spiel 2 verloren: Ben und Dora zahlen je 20, Stand danach -20, +20, -20, +20.
    assert!(lines[2].ends_with("20,-20,-20,20,20,-20,-20,20"));
}

#[test]
fn preview_does_not_save() {
    let mut t = table();
    let preview = t.preview_game(sauspiel(0, 1, 3)).unwrap();
    assert_eq!(preview.value, 20);
    assert_eq!(preview.payouts, vec![-20, 20, -20, 20]);
    assert!(t.games().is_empty());
    assert!(t.preview_game(sauspiel(0, 1, 1)).is_err());
    assert_eq!(
        t.add_game(sauspiel(0, 1, 3)).unwrap().payouts,
        preview.payouts
    );
    assert_eq!(t.games().len(), 1);
}

#[test]
fn csv_with_semicolon_for_german_excel() {
    let mut t = Table::new(
        "T",
        vec![
            "Anna".into(),
            "Ben; Jr".into(),
            "Cem, Sr".into(),
            "Dora".into(),
        ],
        0,
        Tariff::default(),
    )
    .unwrap();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    let csv = t.to_csv_with(';');
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].starts_with("Nr;Zeit;Geber;Spielart;"));
    // Semikolon im Namen und die Mitspielenden-Liste werden maskiert, das Komma im Namen nicht.
    assert!(
        lines[0].contains("\"Ben; Jr Gewinn_Cent\";\"Ben; Jr Stand_Cent\";Cem, Sr Gewinn_Cent;")
    );
    assert!(lines[1].starts_with("1;;Anna;Sauspiel;Eichel;\"Ben; Jr\";Dora;\""));
    // Mit Komma als Trennzeichen bleibt alles wie bisher.
    assert_eq!(t.to_csv(), t.to_csv_with(','));
    assert!(t
        .to_csv()
        .lines()
        .next()
        .unwrap()
        .contains("\"Cem, Sr Gewinn_Cent\""));
    assert_eq!(t.csv_row(0), t.csv_row_with(0, ','));
}

#[test]
fn geier_counts_laufende_from_two_like_wenz() {
    let t = Tariff::default();
    for kind in [GameKind::Wenz, GameKind::Geier] {
        let two = GameInput {
            laufende: 2,
            ..GameInput::new(0, kind, 1)
        };
        let one = GameInput {
            laufende: 1,
            ..GameInput::new(0, kind, 1)
        };
        assert_eq!(game_value(&t, &two), 50 + 2 * 10, "{kind:?}");
        assert_eq!(game_value(&t, &one), 50, "{kind:?}");
    }
    let farbgeier = GameInput {
        suit: Some(Suit::Gras),
        laufende: 2,
        ..GameInput::new(0, GameKind::Farbgeier, 1)
    };
    assert_eq!(game_value(&t, &farbgeier), 70);
    // Farbsolo zählt weiter erst ab drei.
    let solo = GameInput {
        suit: Some(Suit::Gras),
        laufende: 2,
        ..GameInput::new(0, GameKind::Farbsolo, 1)
    };
    assert_eq!(game_value(&t, &solo), 50);
}

#[test]
fn tariff_saved_with_the_old_field_name_still_loads() {
    let mut json = serde_json::to_value(Tariff::default()).unwrap();
    let object = json.as_object_mut().unwrap();
    let value = object.remove("laufende_min_wenz_geier").unwrap();
    object.insert("laufende_min_wenz".into(), value);
    let loaded: Tariff = serde_json::from_value(json).unwrap();
    assert_eq!(loaded, Tariff::default());
}

#[test]
fn edited_game_keeps_the_tariff_it_was_played_with() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.set_tariff(Tariff {
        sauspiel: 50,
        ..Tariff::default()
    });
    t.add_game(sauspiel(1, 1, 3)).unwrap();
    assert_eq!(t.balances(), vec![-70, 70, -70, 70]);

    // Das erste Spiel bekommt Schneider: 20 + 10 mit dem Tarif von damals, nicht 50 + 10.
    let mut input = t.games()[0].input.clone();
    input.schneider = true;
    let edited = t.replace_game(0, input).unwrap();
    assert_eq!(edited.value, 30);
    assert_eq!(edited.calculated_value, 30);
    assert_eq!(edited.tariff.as_ref().unwrap().sauspiel, 20);
    assert_eq!(t.games()[1].value, 50, "das zweite Spiel bleibt unberührt");
    assert_eq!(t.balances(), vec![-80, 80, -80, 80]);
}

#[test]
fn edit_can_change_winner_players_and_price() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    let mut input = t.games()[0].input.clone();
    input.won = false;
    input.declarer = 2;
    input.partner = Some(0);
    input.price_override = Some(100);
    let preview = t.preview_edit(0, input.clone()).unwrap();
    assert_eq!(
        t.games()[0].payouts,
        vec![-20, 20, -20, 20],
        "Vorschau ändert nichts"
    );
    assert_eq!(preview.payouts, vec![-100, 100, -100, 100]);
    t.replace_game(0, input).unwrap();
    assert_eq!(t.balances(), vec![-100, 100, -100, 100]);
}

#[test]
fn invalid_edits_and_unknown_games_are_rejected() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    let mut bad = t.games()[0].input.clone();
    bad.partner = Some(bad.declarer);
    assert_eq!(
        t.replace_game(0, bad).unwrap_err(),
        Error::PartnerIsDeclarer
    );
    assert_eq!(t.games()[0].payouts, vec![-20, 20, -20, 20]);
    let ok = t.games()[0].input.clone();
    assert_eq!(
        t.replace_game(5, ok.clone()).unwrap_err(),
        Error::UnknownGame(5)
    );
    assert_eq!(t.preview_edit(1, ok).unwrap_err(), Error::UnknownGame(1));
    assert!(t.remove_game(3).is_none());
}

#[test]
fn removing_a_game_from_the_middle() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.add_game(GameInput::new(1, GameKind::Wenz, 2)).unwrap();
    t.add_game(sauspiel(2, 0, 1)).unwrap();
    let removed = t.remove_game(1).unwrap();
    assert_eq!(removed.input.kind, GameKind::Wenz);
    assert_eq!(t.games().len(), 2);
    assert_eq!(t.balances(), vec![0, 40, -40, 0]);
    assert_eq!(
        t.next_dealer(),
        3,
        "der Geber richtet sich nach dem letzten Spiel"
    );
}

#[test]
fn games_saved_without_a_tariff_are_repriced_with_the_current_one_when_edited() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    let mut json = serde_json::to_value(&t).unwrap();
    json["games"][0].as_object_mut().unwrap().remove("tariff");
    let mut old: Table = serde_json::from_value(json).unwrap();
    assert!(old.games()[0].tariff.is_none());
    old.set_tariff(Tariff {
        sauspiel: 40,
        ..Tariff::default()
    });
    let input = old.games()[0].input.clone();
    assert_eq!(old.replace_game(0, input).unwrap().value, 40);
    assert!(old.games()[0].tariff.is_some());
}

fn kinds_of(library: &Library, name: &str) -> Vec<GameKind> {
    library.rule_set(name).unwrap().kinds.clone()
}

#[test]
fn library_starts_with_usable_defaults() {
    let lib = Library::with_defaults();
    assert_eq!(lib.default_rule_set().name, "Unsere Runde");
    assert_eq!(
        kinds_of(&lib, "Unsere Runde"),
        [
            GameKind::Sauspiel,
            GameKind::Wenz,
            GameKind::Geier,
            GameKind::Farbsolo,
            GameKind::Ramsch
        ]
    );
    assert_eq!(kinds_of(&lib, "Alle Spielarten").len(), GameKind::ALL.len());
    assert_eq!(lib.default_tariff().name, "20/50");
    assert_eq!(lib.default_tariff().tariff, Tariff::default());
    assert!(lib.is_default_rule_set("Unsere Runde") && !lib.is_default_rule_set("Alle Spielarten"));
}

#[test]
fn rule_sets_can_be_created_edited_and_renamed() {
    let mut lib = Library::with_defaults();
    let name = lib
        .save_rule_set(
            None,
            "  Kneipe  ",
            &[GameKind::Wenz, GameKind::Sauspiel, GameKind::Wenz],
        )
        .unwrap();
    assert_eq!(name, "Kneipe");
    assert_eq!(
        kinds_of(&lib, "Kneipe"),
        [GameKind::Sauspiel, GameKind::Wenz],
        "ohne Doppelte, in üblicher Reihenfolge"
    );
    lib.save_rule_set(Some("Kneipe"), "Kneipe 2", &[GameKind::Bettel])
        .unwrap();
    assert!(lib.rule_set("Kneipe").is_none());
    assert_eq!(kinds_of(&lib, "Kneipe 2"), [GameKind::Bettel]);
    // Umbenennen auf den eigenen Namen in anderer Schreibweise ist erlaubt.
    lib.save_rule_set(Some("Kneipe 2"), "KNEIPE 2", &[GameKind::Bettel])
        .unwrap();
    assert_eq!(lib.rule_sets().len(), 3);
}

#[test]
fn names_must_be_filled_unique_and_kinds_not_empty() {
    let mut lib = Library::with_defaults();
    assert_eq!(
        lib.save_rule_set(None, "  ", &[GameKind::Wenz])
            .unwrap_err(),
        Error::NameEmpty
    );
    assert_eq!(
        lib.save_rule_set(None, "unsere runde", &[GameKind::Wenz])
            .unwrap_err(),
        Error::NameTaken("unsere runde".into())
    );
    assert_eq!(
        lib.save_rule_set(None, "Neu", &[]).unwrap_err(),
        Error::NoKinds
    );
    assert_eq!(
        lib.save_rule_set(Some("Gibt es nicht"), "X", &[GameKind::Wenz])
            .unwrap_err(),
        Error::UnknownEntry("Gibt es nicht".into())
    );
    assert_eq!(
        lib.save_rule_set(Some("Alle Spielarten"), "Unsere Runde", &[GameKind::Wenz])
            .unwrap_err(),
        Error::NameTaken("Unsere Runde".into())
    );
    assert_eq!(
        lib.save_tariff(None, "20/50", Tariff::default())
            .unwrap_err(),
        Error::NameTaken("20/50".into())
    );
    assert_eq!(lib.rule_sets().len(), 2, "nichts davon hat etwas verändert");
}

#[test]
fn default_follows_renames_and_deletes() {
    let mut lib = Library::with_defaults();
    lib.save_rule_set(Some("Unsere Runde"), "Stammtisch", &[GameKind::Wenz])
        .unwrap();
    assert_eq!(lib.default_rule_set().name, "Stammtisch");
    lib.set_default_rule_set("Alle Spielarten").unwrap();
    assert!(lib.is_default_rule_set("Alle Spielarten"));
    lib.delete_rule_set("Alle Spielarten").unwrap();
    assert_eq!(
        lib.default_rule_set().name,
        "Stammtisch",
        "das erste übrige wird Standard"
    );
    assert_eq!(
        lib.delete_rule_set("Stammtisch").unwrap_err(),
        Error::LastEntry
    );
    assert_eq!(
        lib.delete_rule_set("Nope").unwrap_err(),
        Error::UnknownEntry("Nope".into())
    );
    assert_eq!(
        lib.set_default_rule_set("Nope").unwrap_err(),
        Error::UnknownEntry("Nope".into())
    );
}

#[test]
fn tariffs_work_like_rule_sets() {
    let mut lib = Library::with_defaults();
    let cheap = Tariff {
        sauspiel: 10,
        solo: 20,
        ..Tariff::default()
    };
    lib.save_tariff(None, "Günstig", cheap.clone()).unwrap();
    assert_eq!(lib.tariff("Günstig").unwrap().tariff, cheap);
    lib.set_default_tariff("Günstig").unwrap();
    assert_eq!(lib.default_tariff().tariff, cheap);
    lib.save_tariff(Some("Günstig"), "Billig", cheap.clone())
        .unwrap();
    assert!(
        lib.is_default_tariff("Billig"),
        "der Standard zieht beim Umbenennen mit"
    );
    lib.delete_tariff("Billig").unwrap();
    assert_eq!(lib.default_tariff().name, "20/50");
    assert_eq!(lib.delete_tariff("20/50").unwrap_err(), Error::LastEntry);
}

#[test]
fn library_json_roundtrip_and_validation() {
    let mut lib = Library::with_defaults();
    lib.save_tariff(
        None,
        "Zweiter",
        Tariff {
            sauspiel: 30,
            ..Tariff::default()
        },
    )
    .unwrap();
    assert_eq!(Library::from_json(&lib.to_json().unwrap()).unwrap(), lib);
    assert!(matches!(Library::from_json("{"), Err(Error::Json(_))));
    let mut broken = serde_json::to_value(&lib).unwrap();
    broken["tariffs"] = serde_json::json!([]);
    assert!(matches!(
        Library::from_json(&broken.to_string()),
        Err(Error::Json(_))
    ));
    let mut future = serde_json::to_value(&lib).unwrap();
    future["version"] = serde_json::json!(99);
    assert_eq!(
        Library::from_json(&future.to_string()).unwrap_err(),
        Error::UnsupportedVersion(99)
    );
    // Ein Standard, den es nicht mehr gibt, fällt auf den ersten Eintrag zurück.
    let mut dangling = serde_json::to_value(&lib).unwrap();
    dangling["default_tariff"] = serde_json::json!("weg");
    let repaired = Library::from_json(&dangling.to_string()).unwrap();
    assert_eq!(repaired.default_tariff().name, "20/50");
}

#[test]
fn table_remembers_enabled_kinds_and_names() {
    let mut t = table();
    assert!(
        GameKind::ALL.iter().all(|&k| t.is_enabled(k)),
        "neue Abende: alle Spielarten"
    );
    t.use_rules(Some("Kneipe".into()), &[GameKind::Wenz, GameKind::Sauspiel])
        .unwrap();
    assert!(t.is_enabled(GameKind::Wenz) && !t.is_enabled(GameKind::Geier));
    assert_eq!(t.rule_set_name.as_deref(), Some("Kneipe"));
    assert_eq!(t.use_rules(None, &[]).unwrap_err(), Error::NoKinds);
    assert!(t.is_enabled(GameKind::Wenz), "der Fehler ändert nichts");
    t.use_tariff(
        Some("Günstig".into()),
        Tariff {
            sauspiel: 10,
            ..Tariff::default()
        },
    );
    assert_eq!(t.tariff_name.as_deref(), Some("Günstig"));
    t.set_tariff(Tariff::default());
    assert_eq!(t.tariff_name, None);
    // Ein bereits gespieltes Spiel bleibt, auch wenn seine Spielart danach nicht mehr aktiv ist.
    t.add_game(GameInput::new(0, GameKind::Wenz, 1)).unwrap();
    t.use_rules(None, &[GameKind::Sauspiel]).unwrap();
    assert_eq!(t.games().len(), 1);
    assert_eq!(Table::from_json(&t.to_json().unwrap()).unwrap(), t);
}

#[test]
fn old_table_files_without_rules_load_with_all_kinds() {
    let t = table();
    let mut json = serde_json::to_value(&t).unwrap();
    let object = json.as_object_mut().unwrap();
    for key in ["enabled_kinds", "rule_set_name", "tariff_name"] {
        object.remove(key);
    }
    let old = Table::from_json(&json.to_string()).unwrap();
    assert_eq!(old.enabled_kinds, GameKind::ALL.to_vec());
    assert_eq!((old.rule_set_name, old.tariff_name), (None, None));
}

#[test]
fn max_laufende_per_game_kind_and_side() {
    // (Spielart, Lauf der Spielerpartei, Gegenlauf der Gegner)
    let expected = [
        (GameKind::Sauspiel, 14, 14),
        (GameKind::Hochzeit, 14, 14),
        (GameKind::Farbsolo, 8, 14),
        (GameKind::Wenz, 4, 4),
        (GameKind::Geier, 4, 4),
        (GameKind::Farbwenz, 8, 11),
        (GameKind::Farbgeier, 8, 11),
        (GameKind::Bettel, 0, 0),
        (GameKind::Ramsch, 0, 0),
    ];
    assert_eq!(expected.len(), GameKind::ALL.len());
    for (kind, lauf, gegenlauf) in expected {
        assert_eq!(kind.max_laufende(false), lauf, "{kind:?} Lauf");
        assert_eq!(kind.max_laufende(true), gegenlauf, "{kind:?} Gegenlauf");
    }
}

#[test]
fn a_solo_player_has_only_eight_cards_but_the_opponents_may_hold_more() {
    let mut t = table();
    let solo = |laufende: u8, gegenlauf: bool, dealer: Seat| GameInput {
        suit: Some(Suit::Gras),
        laufende,
        gegenlauf,
        ..GameInput::new(dealer, GameKind::Farbsolo, 1)
    };
    assert!(t.add_game(solo(8, false, 0)).is_ok());
    let err = t.add_game(solo(9, false, 1)).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidLaufende {
            kind: GameKind::Farbsolo,
            got: 9,
            max: 8
        }
    );
    assert_eq!(
        err.to_string(),
        "Beim Farbsolo gibt es höchstens 8 Laufende, nicht 9"
    );
    assert!(
        t.add_game(solo(12, true, 1)).is_ok(),
        "ein Gegenlauf darf länger sein"
    );
    assert!(t.add_game(solo(14, true, 2)).is_ok());
    assert!(t.add_game(solo(15, true, 3)).is_err());
    // Beim Sauspiel haben beide Parteien 16 Karten.
    assert!(t
        .add_game(GameInput {
            laufende: 14,
            ..sauspiel(0, 1, 3)
        })
        .is_ok());
}

#[test]
fn laufende_above_eight_are_possible_in_a_sauspiel() {
    let mut t = table();
    let ten = GameInput {
        laufende: 10,
        ..sauspiel(0, 1, 3)
    };
    // 20 + 10 * 10 Laufende
    assert_eq!(t.add_game(ten).unwrap().value, 120);
    let all = GameInput {
        laufende: 14,
        ..sauspiel(1, 1, 3)
    };
    assert_eq!(t.add_game(all).unwrap().value, 20 + 140);
    let too_many = GameInput {
        laufende: 15,
        ..sauspiel(2, 1, 3)
    };
    let err = t.add_game(too_many).unwrap_err();
    assert_eq!(
        err,
        Error::InvalidLaufende {
            kind: GameKind::Sauspiel,
            got: 15,
            max: 14
        }
    );
    assert_eq!(
        err.to_string(),
        "Beim Sauspiel gibt es höchstens 14 Laufende, nicht 15"
    );
}

#[test]
fn wenz_and_geier_have_at_most_four_laufende() {
    let mut t = table();
    for kind in [GameKind::Wenz, GameKind::Geier] {
        let ok = GameInput {
            laufende: 4,
            ..GameInput::new(0, kind, 1)
        };
        assert!(t.add_game(ok).is_ok(), "{kind:?}");
        let bad = GameInput {
            laufende: 5,
            ..GameInput::new(0, kind, 1)
        };
        assert!(
            matches!(t.add_game(bad), Err(Error::InvalidLaufende { max: 4, .. })),
            "{kind:?}"
        );
    }
}

#[test]
fn players_are_stored_once_and_sorted_for_lists() {
    let mut lib = Library::with_defaults();
    assert!(lib.players().is_empty());
    assert_eq!(lib.add_player("  Zoë ").unwrap(), "Zoë");
    lib.add_player("anna").unwrap();
    lib.add_player("Ben").unwrap();
    assert_eq!(
        lib.players(),
        ["Zoë", "anna", "Ben"],
        "Reihenfolge des Anlegens"
    );
    assert_eq!(
        lib.players_sorted(),
        ["anna", "Ben", "Zoë"],
        "alphabetisch ohne Schreibweise"
    );
    assert_eq!(
        lib.add_player("ANNA").unwrap_err(),
        Error::NameTaken("ANNA".into())
    );
    assert_eq!(lib.add_player("   ").unwrap_err(), Error::NameEmpty);
    assert_eq!(lib.players().len(), 3);
}

#[test]
fn ensure_players_adds_only_what_is_missing() {
    let mut lib = Library::with_defaults();
    lib.add_player("Anna").unwrap();
    let names = ["anna", "Ben", "", "  ", "Ben", "Cem"]
        .map(String::from)
        .to_vec();
    assert_eq!(lib.ensure_players(&names), 2);
    assert_eq!(lib.players(), ["Anna", "Ben", "Cem"]);
    assert_eq!(
        lib.ensure_players(&names),
        0,
        "ein zweites Mal ändert nichts"
    );
}

#[test]
fn players_can_be_renamed_and_removed() {
    let mut lib = Library::with_defaults();
    lib.ensure_players(&["Anna".into(), "Ben".into()]);
    assert_eq!(lib.rename_player("Ben", "Benedikt").unwrap(), "Benedikt");
    assert_eq!(lib.players(), ["Anna", "Benedikt"]);
    // Nur die Schreibweise des eigenen Namens zu ändern ist erlaubt.
    assert_eq!(lib.rename_player("Anna", "ANNA").unwrap(), "ANNA");
    assert_eq!(
        lib.rename_player("ANNA", "benedikt").unwrap_err(),
        Error::NameTaken("benedikt".into())
    );
    assert_eq!(
        lib.rename_player("ANNA", " ").unwrap_err(),
        Error::NameEmpty
    );
    assert_eq!(
        lib.rename_player("Nobody", "X").unwrap_err(),
        Error::UnknownEntry("Nobody".into())
    );
    lib.remove_player("ANNA").unwrap();
    assert_eq!(lib.players(), ["Benedikt"]);
    assert_eq!(
        lib.remove_player("ANNA").unwrap_err(),
        Error::UnknownEntry("ANNA".into())
    );
    lib.remove_player("Benedikt").unwrap();
    assert!(
        lib.players().is_empty(),
        "anders als bei Regelwerken darf die Liste leer werden"
    );
}

#[test]
fn player_list_roundtrips_and_old_library_files_still_load() {
    let mut lib = Library::with_defaults();
    lib.ensure_players(&["Anna".into(), "Ben".into()]);
    assert_eq!(Library::from_json(&lib.to_json().unwrap()).unwrap(), lib);
    let mut json = serde_json::to_value(&lib).unwrap();
    json.as_object_mut().unwrap().remove("players");
    let old = Library::from_json(&json.to_string()).unwrap();
    assert!(old.players().is_empty());
    assert_eq!(old.rule_sets(), lib.rule_sets());
}

#[test]
fn gegenlauf_needs_laufende_and_does_not_change_the_price() {
    let mut t = table();
    let lauf = GameInput {
        laufende: 4,
        ..sauspiel(0, 1, 3)
    };
    let gegen = GameInput {
        laufende: 4,
        gegenlauf: true,
        ..sauspiel(1, 1, 3)
    };
    let a = t.add_game(lauf).unwrap().clone();
    let b = t.add_game(gegen).unwrap().clone();
    assert_eq!(
        (a.value, a.payouts.clone()),
        (b.value, b.payouts.clone()),
        "Geld ist gleich"
    );
    let none = GameInput {
        gegenlauf: true,
        ..sauspiel(2, 1, 3)
    };
    assert_eq!(
        t.add_game(none).unwrap_err(),
        Error::GegenlaufWithoutLaufende
    );
    assert_eq!(t.games().len(), 2);
}

#[test]
fn gegenlauf_roundtrips_and_old_games_default_to_lauf() {
    let mut t = table();
    t.add_game(GameInput {
        laufende: 3,
        gegenlauf: true,
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    assert_eq!(Table::from_json(&t.to_json().unwrap()).unwrap(), t);
    let mut json = serde_json::to_value(&t).unwrap();
    json["games"][0]["input"]
        .as_object_mut()
        .unwrap()
        .remove("gegenlauf");
    let old = Table::from_json(&json.to_string()).unwrap();
    assert!(!old.games()[0].input.gegenlauf);
}

#[test]
fn csv_has_a_gegenlauf_column() {
    let mut t = table();
    t.add_game(GameInput {
        laufende: 3,
        gegenlauf: true,
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    let header: Vec<String> = t.csv_header().split(',').map(String::from).collect();
    let row: Vec<String> = t.csv_row(0).unwrap().split(',').map(String::from).collect();
    let at = header.iter().position(|h| h == "Laufende").unwrap();
    assert_eq!(header[at + 1], "Gegenlauf");
    assert_eq!((row[at].as_str(), row[at + 1].as_str()), ("3", "ja"));
}

fn lauf_stats(t: &Table, kind: GameKind) -> LaufendeStats {
    table_stats(t).laufende_by_kind[&kind]
}

#[test]
fn lauf_and_gegenlauf_are_counted_per_kind_above_the_minimum() {
    let mut t = table();
    // Sauspiel zählt ab 3: Lauf mit 4, Gegenlauf mit 3, 2 Laufende zählen nicht, eines ohne.
    t.add_game(GameInput {
        laufende: 4,
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    t.add_game(GameInput {
        laufende: 3,
        gegenlauf: true,
        ..sauspiel(1, 1, 3)
    })
    .unwrap();
    t.add_game(GameInput {
        laufende: 2,
        ..sauspiel(2, 1, 3)
    })
    .unwrap();
    t.add_game(sauspiel(3, 1, 3)).unwrap();
    let s = lauf_stats(&t, GameKind::Sauspiel);
    assert_eq!((s.games, s.lauf, s.gegenlauf, s.without()), (4, 1, 1, 2));
    assert_eq!((s.lauf_sum, s.gegenlauf_sum), (4, 3));
    assert_eq!(
        (s.average_lauf(), s.average_gegenlauf()),
        (Some(4.0), Some(3.0))
    );

    // Wenz zählt ab 2.
    t.add_game(GameInput {
        laufende: 2,
        ..GameInput::new(0, GameKind::Wenz, 1)
    })
    .unwrap();
    t.add_game(GameInput {
        laufende: 1,
        ..GameInput::new(1, GameKind::Wenz, 1)
    })
    .unwrap();
    let w = lauf_stats(&t, GameKind::Wenz);
    assert_eq!((w.games, w.lauf, w.gegenlauf), (2, 1, 0));
    assert_eq!(w.average_gegenlauf(), None);
}

#[test]
fn kinds_without_extras_have_no_lauf_stats() {
    let mut t = table();
    t.add_game(GameInput::new(0, GameKind::Ramsch, 1)).unwrap();
    t.add_game(GameInput::new(1, GameKind::Bettel, 1)).unwrap();
    assert!(table_stats(&t).laufende_by_kind.is_empty());
}

#[test]
fn the_games_own_tariff_decides_what_counts_as_a_lauf() {
    let mut t = table();
    t.set_tariff(Tariff {
        laufende_min_sauspiel: 2,
        ..Tariff::default()
    });
    t.add_game(GameInput {
        laufende: 2,
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    t.set_tariff(Tariff {
        laufende_min_sauspiel: 5,
        ..Tariff::default()
    });
    t.add_game(GameInput {
        laufende: 2,
        ..sauspiel(1, 1, 3)
    })
    .unwrap();
    let s = lauf_stats(&t, GameKind::Sauspiel);
    assert_eq!(
        (s.games, s.lauf),
        (2, 1),
        "nur das Spiel nach dem Tarif mit Grenze 2 zählt"
    );
}

#[test]
fn players_know_which_side_had_the_laufende() {
    let mut t = table();
    // Spiel 1: Ben und Dora spielen, sie haben den Lauf. Spiel 2: Gegenlauf, also Anna und Cem.
    t.add_game(GameInput {
        laufende: 3,
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    t.add_game(GameInput {
        laufende: 3,
        gegenlauf: true,
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    let stats = table_stats(&t);
    let own = |seat: usize| {
        (
            stats.players[seat].lauf_own_side,
            stats.players[seat].lauf_other_side,
        )
    };
    assert_eq!(
        own(1),
        (1, 1),
        "Ben: im ersten Spiel Lauf, im zweiten Gegenlauf gegen sich"
    );
    assert_eq!(own(3), (1, 1));
    assert_eq!(
        own(0),
        (1, 1),
        "Anna: im ersten Spiel hatten ihre Gegner den Lauf, im zweiten sie selbst"
    );
}

fn ramsch_with_knocks(dealer: Seat, knockers: Vec<Seat>) -> GameInput {
    GameInput {
        won: false,
        knockers,
        ..GameInput::new(dealer, GameKind::Ramsch, 2)
    }
}

#[test]
fn knocking_in_a_ramsch_doubles_by_default() {
    let mut t = table();
    let g = t.add_game(ramsch_with_knocks(0, vec![0, 1])).unwrap();
    assert_eq!(g.value, 20 << 2, "wie bisher: zweimal Klopfen = vierfach");
}

#[test]
fn knocking_in_a_ramsch_can_be_left_out_of_the_price_but_is_still_recorded() {
    let mut t = table();
    t.set_tariff(Tariff {
        ramsch_knock_doubles: false,
        ..Tariff::default()
    });
    let g = t
        .add_game(ramsch_with_knocks(0, vec![0, 1]))
        .unwrap()
        .clone();
    assert_eq!(
        (g.value, g.calculated_value),
        (20, 20),
        "der Preis bleibt einfach"
    );
    assert_eq!(
        g.input.knockers,
        vec![0, 1],
        "das Klopfen ist trotzdem erfasst"
    );
    assert_eq!(g.payouts, vec![20, 20, -60, 20]);
    let stats = table_stats(&t);
    assert_eq!(stats.knocked_games, 1);
    assert_eq!(stats.players[0].knocks.games, 1);
    assert_eq!(stats.players[1].knocks.games, 1);
}

#[test]
fn the_ramsch_option_leaves_other_games_alone() {
    let tariff = Tariff {
        ramsch_knock_doubles: false,
        ..Tariff::default()
    };
    let sau = GameInput {
        knockers: vec![0, 1],
        ..sauspiel(0, 1, 3)
    };
    assert_eq!(
        game_value(&tariff, &sau),
        20 << 2,
        "beim Sauspiel zählt das Klopfen weiter"
    );
    let wenz = GameInput {
        knockers: vec![0],
        ..GameInput::new(0, GameKind::Wenz, 1)
    };
    assert_eq!(game_value(&tariff, &wenz), 50 << 1);
    // Ein verlangter Preis ersetzt die Rechnung wie immer.
    let demanded = GameInput {
        price_override: Some(70),
        ..ramsch_with_knocks(0, vec![0])
    };
    assert_eq!(game_value(&tariff, &demanded), 70);
}

#[test]
fn tariffs_saved_before_the_ramsch_option_still_double() {
    let mut json = serde_json::to_value(Tariff::default()).unwrap();
    json.as_object_mut().unwrap().remove("ramsch_knock_doubles");
    let old: Tariff = serde_json::from_value(json).unwrap();
    assert!(old.ramsch_knock_doubles);
    assert_eq!(old, Tariff::default());
}

fn bock_table(mode: BockMode) -> Table {
    let mut t = table();
    t.bock_mode = mode;
    t
}

fn game_with(dealer: Seat, bock: bool, starts_bock: bool) -> GameInput {
    GameInput {
        bock,
        starts_bock,
        ..sauspiel(dealer, 1, 3)
    }
}

#[test]
fn a_bock_game_doubles_the_price_and_stacks_with_knocking() {
    let mut t = bock_table(BockMode::NextGame);
    assert_eq!(t.add_game(game_with(0, false, false)).unwrap().value, 20);
    assert_eq!(t.add_game(game_with(1, true, false)).unwrap().value, 40);
    let knocked = GameInput {
        knockers: vec![2],
        ..game_with(2, true, false)
    };
    assert_eq!(
        t.add_game(knocked).unwrap().value,
        20 << 2,
        "Klopfen und Bock verdoppeln beide"
    );
}

#[test]
fn bock_for_the_next_game_only() {
    let mut t = bock_table(BockMode::NextGame);
    assert_eq!(t.bock_pending(), 0);
    t.add_game(game_with(0, false, true)).unwrap();
    assert_eq!(t.bock_pending(), 1, "das nächste Spiel ist ein Bockspiel");
    t.add_game(game_with(1, true, false)).unwrap();
    assert_eq!(t.bock_pending(), 0, "danach ist der Bock verbraucht");
    t.add_game(game_with(2, false, false)).unwrap();
    assert_eq!(t.bock_pending(), 0);
}

#[test]
fn bock_for_a_whole_round_lasts_as_many_games_as_players() {
    let mut t = bock_table(BockMode::NextRound);
    t.add_game(game_with(0, false, true)).unwrap();
    assert_eq!(t.bock_pending(), 4);
    for dealer in 1..=3 {
        t.add_game(game_with(dealer, true, false)).unwrap();
    }
    assert_eq!(t.bock_pending(), 1);
    t.add_game(game_with(0, true, false)).unwrap();
    assert_eq!(t.bock_pending(), 0);
}

#[test]
fn a_new_trigger_during_a_bock_round_adds_to_it() {
    let mut t = bock_table(BockMode::NextRound);
    t.add_game(game_with(0, false, true)).unwrap(); // 4 offen
    t.add_game(game_with(1, true, true)).unwrap(); // eines verbraucht, vier neue
    assert_eq!(t.bock_pending(), 4 - 1 + 4);
    let mut single = bock_table(BockMode::NextGame);
    single.add_game(game_with(0, false, true)).unwrap();
    single.add_game(game_with(1, true, true)).unwrap();
    assert_eq!(
        single.bock_pending(),
        1,
        "das Bockspiel löst wieder eines aus"
    );
}

#[test]
fn without_a_bock_mode_a_trigger_does_nothing() {
    let mut t = bock_table(BockMode::Off);
    t.add_game(game_with(0, false, true)).unwrap();
    assert_eq!(t.bock_pending(), 0);
}

#[test]
fn bock_follows_undo_edit_and_removal_of_games() {
    let mut t = bock_table(BockMode::NextGame);
    t.add_game(game_with(0, false, true)).unwrap();
    assert_eq!(t.bock_pending(), 1);
    // Der Auslöser wird zurückgenommen.
    let mut input = t.games()[0].input.clone();
    input.starts_bock = false;
    t.replace_game(0, input).unwrap();
    assert_eq!(t.bock_pending(), 0);
    t.add_game(game_with(1, false, true)).unwrap();
    assert_eq!(t.bock_pending(), 1);
    t.undo_last();
    assert_eq!(t.bock_pending(), 0);
    t.add_game(game_with(1, false, true)).unwrap();
    t.remove_game(0).unwrap();
    assert_eq!(t.bock_pending(), 1, "der zweite Auslöser zählt weiter");
    assert_eq!(t.bock_pending_before(0), 0);
}

#[test]
fn bock_fields_roundtrip_and_old_files_play_without_bock() {
    let mut t = bock_table(BockMode::NextRound);
    t.add_game(game_with(0, false, true)).unwrap();
    assert_eq!(Table::from_json(&t.to_json().unwrap()).unwrap(), t);
    let mut json = serde_json::to_value(&t).unwrap();
    json.as_object_mut().unwrap().remove("bock_mode");
    let input = json["games"][0]["input"].as_object_mut().unwrap();
    input.remove("bock");
    input.remove("starts_bock");
    let old = Table::from_json(&json.to_string()).unwrap();
    assert_eq!(old.bock_mode, BockMode::Off);
    assert!(!old.games()[0].input.bock && !old.games()[0].input.starts_bock);
    assert_eq!(old.bock_pending(), 0);
}

#[test]
fn rule_sets_carry_the_bock_mode() {
    let lib = Library::with_defaults();
    assert_eq!(
        lib.rule_set("Unsere Runde").unwrap().bock,
        BockMode::NextGame
    );
    assert_eq!(lib.rule_set("Alle Spielarten").unwrap().bock, BockMode::Off);
    let mut lib = lib;
    lib.save_rule_set_with(None, "Kneipe", &[GameKind::Wenz], BockMode::NextRound)
        .unwrap();
    assert_eq!(lib.rule_set("Kneipe").unwrap().bock, BockMode::NextRound);
    // Die kurze Fassung lässt den Bock-Modus, wie er ist.
    lib.save_rule_set(
        Some("Kneipe"),
        "Kneipe 2",
        &[GameKind::Wenz, GameKind::Geier],
    )
    .unwrap();
    assert_eq!(lib.rule_set("Kneipe 2").unwrap().bock, BockMode::NextRound);
    let mut t = table();
    t.use_rule_set(lib.rule_set("Kneipe 2").unwrap()).unwrap();
    assert_eq!(t.bock_mode, BockMode::NextRound);
    assert_eq!(t.rule_set_name.as_deref(), Some("Kneipe 2"));
    assert!(t.is_enabled(GameKind::Geier) && !t.is_enabled(GameKind::Sauspiel));
    // Alte Bibliotheksdateien ohne Bock-Modus.
    let mut json = serde_json::to_value(&lib).unwrap();
    for rule in json["rule_sets"].as_array_mut().unwrap() {
        rule.as_object_mut().unwrap().remove("bock");
    }
    let old = Library::from_json(&json.to_string()).unwrap();
    assert!(old.rule_sets().iter().all(|r| r.bock == BockMode::Off));
}

#[test]
fn bock_is_counted_in_the_statistics_and_the_csv() {
    let mut t = bock_table(BockMode::NextGame);
    t.add_game(game_with(0, false, true)).unwrap();
    t.add_game(game_with(1, true, false)).unwrap();
    let stats = table_stats(&t);
    assert_eq!((stats.bock_games, stats.bock_started), (1, 1));
    let header: Vec<String> = t.csv_header().split(',').map(String::from).collect();
    let at = header.iter().position(|h| h == "Bockspiel").unwrap();
    assert_eq!(header[at + 1], "Bockrunde_ausgeloest");
    let row0: Vec<String> = t.csv_row(0).unwrap().split(',').map(String::from).collect();
    let row1: Vec<String> = t.csv_row(1).unwrap().split(',').map(String::from).collect();
    assert_eq!((row0[at].as_str(), row0[at + 1].as_str()), ("nein", "ja"));
    assert_eq!((row1[at].as_str(), row1[at + 1].as_str()), ("ja", "nein"));
}

#[test]
fn a_late_player_sits_where_chosen_and_owes_nothing_for_earlier_games() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    // Eva setzt sich zwischen Ben (1) und Cem (2).
    let eva = t.add_player("Eva", Some(1), None).unwrap();
    assert_eq!(eva, 4);
    assert_eq!(t.players.len(), 5);
    assert_eq!(t.ring(), vec![0, 1, 4, 2, 3]);
    assert_eq!(t.balances(), vec![-20, 20, -20, 20, 0]);
    assert_eq!(t.games()[0].payouts, vec![-20, 20, -20, 20, 0]);
    // Der Geber wandert in der neuen Reihenfolge weiter: nach Anna (0) kommt Ben (1), dann Eva.
    assert_eq!(t.next_dealer(), 1);
    t.add_game(sauspiel(1, 0, 2)).unwrap();
    assert_eq!(t.next_dealer(), 4);
    assert_eq!(t.default_participants(4), vec![2, 3, 1, 4]);
    assert_eq!(t.default_participants(3), vec![0, 1, 2, 3]);
}

#[test]
fn a_new_player_needs_a_fresh_name_and_a_free_seat() {
    let mut t = table();
    assert_eq!(
        t.add_player("Anna", None, None),
        Err(Error::DuplicateName("Anna".into()))
    );
    assert_eq!(t.add_player("  ", None, None), Err(Error::EmptyName));
    assert_eq!(
        t.add_player("Eva", Some(9), None),
        Err(Error::UnknownSeat(9))
    );
    for name in ["E1", "E2", "E3", "E4", "E5", "E6"] {
        t.add_player(name, None, None).unwrap();
    }
    assert_eq!(t.player_count(), 10);
    assert_eq!(
        t.add_player("Zu viel", None, None),
        Err(Error::PlayerCount(11))
    );
}

#[test]
fn a_player_who_leaves_is_skipped_and_can_return() {
    let mut t = table();
    t.add_player("Eva", None, None).unwrap();
    t.add_game(sauspiel(0, 1, 2)).unwrap();
    t.leave(1, None).unwrap();
    assert_eq!(t.player_count(), 4);
    assert_eq!(t.present_seats(), vec![0, 2, 3, 4]);
    assert!(!t.is_present(1));
    // Ben fehlt: nach Anna gibt Cem.
    assert_eq!(t.next_dealer(), 2);
    assert_eq!(t.default_participants(2), vec![3, 4, 0, 2]);
    // Sein Stand bleibt stehen, die Spiele auch.
    assert_eq!(t.balances()[1], 20);
    assert_eq!(t.games().len(), 1);
    t.rejoin(1, None).unwrap();
    assert_eq!(t.present_seats(), vec![0, 1, 2, 3, 4]);
    assert_eq!(t.next_dealer(), 1);
}

#[test]
fn nobody_can_leave_below_two_and_only_present_players_can_play() {
    let mut t = Table::new("Zu zweit", names()[..3].to_vec(), 0, Tariff::default()).unwrap();
    t.leave(2, None).unwrap();
    assert_eq!(t.leave(1, None), Err(Error::PlayerCount(1)));
    assert_eq!(t.leave(2, None), Err(Error::UnknownSeat(2)));
    let solo = GameInput::new(0, GameKind::Wenz, 1);
    t.add_game(solo.clone()).unwrap();
    let with_absent = GameInput {
        participants: vec![0, 1, 2],
        ..solo
    };
    assert_eq!(t.preview_game(with_absent), Err(Error::SeatNotAtTable(2)));
}

#[test]
fn old_games_stay_editable_and_keep_who_sat_there() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.add_player("Eva", None, None).unwrap();
    t.leave(3, None).unwrap();
    let edited = GameInput {
        won: false,
        participants: vec![1, 2, 3, 0],
        ..sauspiel(0, 1, 3)
    };
    t.replace_game(0, edited).unwrap();
    assert_eq!(t.games()[0].payouts, vec![20, -20, 20, -20, 0]);
    // Eva war beim ersten Spiel nicht dabei, Dora schon.
    assert!(!t.was_at_table(&t.games()[0], 4));
    assert!(t.was_at_table(&t.games()[0], 3));
}

#[test]
fn sitting_out_only_counts_when_the_player_was_at_the_table() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.add_player("Eva", Some(3), None).unwrap();
    t.add_game(sauspiel(1, 0, 2)).unwrap();
    let stats = table_stats(&t);
    assert_eq!(stats.players[4].games_sat_out, 1);
    assert_eq!(stats.players[4].games_played, 0);
    assert_eq!(stats.players[0].games_sat_out, 0);
}

#[test]
fn bock_round_length_follows_the_players_at_the_table() {
    let mut t = bock_table(BockMode::NextRound);
    t.add_player("Eva", None, None).unwrap();
    t.add_game(GameInput {
        starts_bock: true,
        ..sauspiel(0, 1, 2)
    })
    .unwrap();
    assert_eq!(t.bock_pending(), 5);
    t.leave(2, None).unwrap();
    // Das Spiel zählt mit den fünf, die dabei saßen.
    assert_eq!(t.bock_pending(), 5);
    t.add_game(GameInput {
        starts_bock: true,
        ..sauspiel(1, 0, 3)
    })
    .unwrap();
    assert_eq!(t.bock_pending(), 5 + 4);
}

#[test]
fn tables_with_players_coming_and_going_survive_json_and_old_files_load() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.add_player("Eva", Some(0), None).unwrap();
    t.leave(2, None).unwrap();
    let back = Table::from_json(&t.to_json().unwrap()).unwrap();
    assert_eq!(back, t);
    assert_eq!(back.ring(), vec![0, 4, 1, 2, 3]);

    let old = table().to_json().unwrap();
    let old: String = old
        .lines()
        .filter(|l| !l.contains("\"seating\"") && !l.contains("\"absent\""))
        .collect::<Vec<_>>()
        .join("\n");
    let t = Table::from_json(&old).unwrap();
    assert_eq!(t.present_seats(), vec![0, 1, 2, 3]);
    assert_eq!(t.next_dealer(), 0);
}

fn bock_table_with(mode: BockMode, stacking: BockStacking) -> Table {
    let mut t = table();
    t.bock_mode = mode;
    t.bock_stacking = stacking;
    t
}

fn trigger(dealer: Seat, declarer: Seat, partner: Seat, extra: u8) -> GameInput {
    GameInput {
        starts_bock: true,
        extra_bock_rounds: extra,
        ..sauspiel(dealer, declarer, partner)
    }
}

#[test]
fn two_bock_rounds_in_a_row_make_the_next_two_games_bock() {
    let mut t = bock_table_with(BockMode::NextGame, BockStacking::Sequential);
    t.add_game(trigger(0, 1, 3, 1)).unwrap();
    assert_eq!(t.bock_pending(), 2);
    assert_eq!(t.bock_level(), 1);
    for dealer in [1, 2] {
        let g = GameInput {
            bock: true,
            ..sauspiel(dealer, 0, (dealer + 1) % 4)
        };
        let g = if dealer == 1 { sauspiel(1, 2, 0) } else { g };
        t.add_game(GameInput { bock: true, ..g }).unwrap();
    }
    assert_eq!(t.bock_pending(), 0);
    assert_eq!(table_stats(&t).bock_started, 2);
}

#[test]
fn stacked_bock_rounds_multiply_the_price_while_they_overlap() {
    let mut t = bock_table_with(BockMode::NextRound, BockStacking::Stacked);
    t.add_game(trigger(0, 1, 3, 1)).unwrap();
    // Zwei Runden zu je vier Spielen laufen zugleich.
    assert_eq!(t.bock_pending(), 4);
    assert_eq!(t.bock_level(), 2);
    let game = GameInput {
        bock: true,
        bock_extra: 1,
        ..sauspiel(1, 2, 0)
    };
    let g = t.add_game(game).unwrap();
    assert_eq!(g.value, 80, "20 mal 4");
    assert_eq!(t.bock_pending(), 3);
    assert_eq!(t.bock_level(), 2);
}

#[test]
fn a_new_trigger_during_a_running_round_stacks_or_queues_by_the_rule() {
    for (stacking, pending, level) in [
        (BockStacking::Sequential, 4 + 3, 1),
        (BockStacking::Stacked, 4, 2),
    ] {
        let mut t = bock_table_with(BockMode::NextRound, stacking);
        t.add_game(trigger(0, 1, 3, 0)).unwrap();
        t.add_game(GameInput {
            bock: true,
            starts_bock: true,
            ..sauspiel(1, 2, 0)
        })
        .unwrap();
        // Erste Runde: 4 - 1 = 3 übrig. Zweite Runde: 4 dazu.
        assert_eq!(
            t.bock_pending(),
            if stacking == BockStacking::Stacked {
                4
            } else {
                pending
            },
            "{stacking:?}"
        );
        assert_eq!(t.bock_level(), level, "{stacking:?}");
    }
}

#[test]
fn too_many_bock_rounds_are_rejected() {
    let mut t = bock_table_with(BockMode::NextGame, BockStacking::Sequential);
    assert_eq!(
        t.add_game(trigger(0, 1, 3, MAX_BOCK)).unwrap_err(),
        Error::TooManyBock(MAX_BOCK + 1)
    );
}

#[test]
fn rule_sets_keep_their_stacking() {
    let mut lib = Library::with_defaults();
    lib.save_rule_set_full(
        None,
        "Kneipe",
        &[GameKind::Wenz],
        BockMode::NextRound,
        BockStacking::Stacked,
    )
    .unwrap();
    assert_eq!(
        lib.rule_set("Kneipe").unwrap().stacking,
        BockStacking::Stacked
    );
    lib.save_rule_set_with(
        Some("Kneipe"),
        "Kneipe",
        &[GameKind::Wenz],
        BockMode::NextGame,
    )
    .unwrap();
    assert_eq!(
        lib.rule_set("Kneipe").unwrap().stacking,
        BockStacking::Stacked
    );
    let mut t = table();
    t.use_rule_set(lib.rule_set("Kneipe").unwrap()).unwrap();
    assert_eq!(t.bock_stacking, BockStacking::Stacked);
}

#[test]
fn the_evening_log_records_who_came_left_and_moved() {
    let mut t = table();
    t.created_at = Some(1_000_000_000);
    t.utc_offset_min = 120;
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    let eva = t.add_player("Eva", Some(0), Some(1_000_000_600)).unwrap();
    t.leave(2, Some(1_000_001_200)).unwrap();
    t.move_player(eva, 3, Some(1_000_001_800)).unwrap();
    t.rejoin(2, Some(1_000_002_400)).unwrap();
    t.finish(1_000_003_000);
    let kinds: Vec<_> = t
        .events
        .iter()
        .map(|e| (e.kind, e.seat, e.after_games))
        .collect();
    assert_eq!(
        kinds,
        vec![
            (EventKind::Joined, 4, 1),
            (EventKind::Left, 2, 1),
            (EventKind::Moved, 4, 1),
            (EventKind::Returned, 2, 1),
        ]
    );
    assert_eq!(t.events[0].behind, Some(0));
    assert_eq!(t.ring(), vec![0, 1, 2, 3, 4]);
    assert_eq!(t.last_activity(), Some(1_000_002_400));
    let csv = t.events_csv();
    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(
        lines[0],
        "Zeit,Zeit_Unix,Ereignis,Spieler,Hinter,Nach_Spiel_Nr,Strafe_je_Mitspieler_Cent"
    );
    assert_eq!(lines[1], "2001-09-09 03:46,1000000000,Abend beginnt,,,0,");
    assert_eq!(
        lines[2],
        "2001-09-09 03:56,1000000600,Kommt dazu,Eva,Anna,1,"
    );
    assert_eq!(lines[3], "2001-09-09 04:06,1000001200,Geht,Cem,,1,");
    assert!(lines[4].ends_with("Setzt sich um,Eva,Dora,1,"));
    assert!(lines[6].contains("Abend endet"));
    assert_eq!(lines.len(), 7);
}

#[test]
fn moving_a_player_changes_the_dealing_order_and_a_no_change_is_not_logged() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    assert_eq!(t.next_dealer(), 1);
    // Ben setzt sich hinter Cem: Anna, Cem, Ben, Dora.
    t.move_player(1, 2, None).unwrap();
    assert_eq!(t.ring(), vec![0, 2, 1, 3]);
    assert_eq!(t.next_dealer(), 2);
    let logged = t.events.len();
    t.move_player(1, 2, None).unwrap();
    assert_eq!(t.events.len(), logged);
    assert_eq!(t.move_player(1, 1, None), Err(Error::UnknownSeat(1)));
    assert_eq!(t.move_player(1, 9, None), Err(Error::UnknownSeat(9)));
}

#[test]
fn clock_formats_local_times_and_durations() {
    assert_eq!(format_local(0, 0), "1970-01-01 00:00");
    assert_eq!(format_local(1_000_000_000, 120), "2001-09-09 03:46");
    assert_eq!(format_clock(1_000_000_000, -60), "00:46");
    assert_eq!(format_duration(25 * 60), "25 min");
    assert_eq!(format_duration(3 * 3600 + 40 * 60), "3 h 40 min");
}

#[test]
fn evenings_saved_before_the_log_still_load() {
    let json = table().to_json().unwrap();
    let old: String = json
        .lines()
        .filter(|l| {
            !["bock_stacking", "utc_offset_min", "ended_at", "\"events\""]
                .iter()
                .any(|k| l.contains(k))
        })
        .collect::<Vec<_>>()
        .join("\n");
    let t = Table::from_json(&old).unwrap();
    assert!(t.events.is_empty() && t.ended_at.is_none());
    assert_eq!(t.bock_stacking, BockStacking::Sequential);
}

fn ramsch(dealer: Seat, loser: Seat) -> GameInput {
    GameInput {
        won: false,
        ..GameInput::new(dealer, GameKind::Ramsch, loser)
    }
}

#[test]
fn bock_can_leave_the_ramsch_price_alone_but_is_still_recorded() {
    let tariff = Tariff {
        ramsch_bock_doubles: false,
        ..Tariff::default()
    };
    let input = GameInput {
        bock: true,
        ..ramsch(0, 1)
    };
    assert_eq!(calculated_value(&tariff, &input), 20);
    assert_eq!(calculated_value(&Tariff::default(), &input), 40);
    // Andere Spiele bleiben beim Bock doppelt.
    let sau = GameInput {
        bock: true,
        ..sauspiel(0, 1, 3)
    };
    assert_eq!(calculated_value(&tariff, &sau), 40);
    // Mit Überlagerung zählen auch die Zusatz-Verdopplungen nicht.
    let stacked = GameInput {
        bock_extra: 2,
        ..input
    };
    assert_eq!(calculated_value(&tariff, &stacked), 20);
    assert_eq!(stacked.bock_doublings(), 3);
}

#[test]
fn old_tariffs_still_double_the_ramsch_for_bock() {
    let mut json: serde_json::Value = serde_json::to_value(Tariff::default()).unwrap();
    json.as_object_mut().unwrap().remove("ramsch_bock_doubles");
    let tariff: Tariff = serde_json::from_value(json).unwrap();
    assert!(tariff.ramsch_bock_doubles);
}

fn ramsch_after_trigger(policy: BockAtRamsch, ramsch_is_bock: bool) -> Table {
    let mut t = bock_table_with(BockMode::NextRound, BockStacking::Sequential);
    t.bock_at_ramsch = policy;
    t.add_game(trigger(0, 1, 3, 0)).unwrap();
    assert_eq!(t.bock_pending(), 4);
    t.add_game(GameInput {
        bock: ramsch_is_bock,
        ..ramsch(1, 2)
    })
    .unwrap();
    t
}

#[test]
fn what_a_ramsch_does_to_a_bock_round_follows_the_rule() {
    // Zählt: Der Ramsch verbraucht ein Bockspiel.
    assert_eq!(
        ramsch_after_trigger(BockAtRamsch::Counts, true).bock_pending(),
        3
    );
    // Wartet: Auch als Bockspiel eingetragen verbraucht er nichts.
    assert_eq!(
        ramsch_after_trigger(BockAtRamsch::Waits, true).bock_pending(),
        4
    );
    assert_eq!(
        ramsch_after_trigger(BockAtRamsch::Waits, false).bock_pending(),
        4
    );
    // Verfällt: Die Bockrunde ist weg.
    assert_eq!(
        ramsch_after_trigger(BockAtRamsch::Expires, false).bock_pending(),
        0
    );
    assert_eq!(
        ramsch_after_trigger(BockAtRamsch::Expires, true).bock_pending(),
        0
    );
}

#[test]
fn a_ramsch_that_expires_the_bock_does_not_stop_a_later_trigger() {
    let mut t = ramsch_after_trigger(BockAtRamsch::Expires, false);
    t.add_game(trigger(2, 3, 0, 0)).unwrap();
    assert_eq!(t.bock_pending(), 4);
}

#[test]
fn rule_sets_keep_what_happens_at_ramsch() {
    let mut lib = Library::with_defaults();
    lib.save_rule_set_all(
        None,
        "Kneipe",
        &[GameKind::Ramsch],
        BockMode::NextRound,
        BockStacking::Sequential,
        BockAtRamsch::Expires,
    )
    .unwrap();
    lib.save_rule_set_full(
        Some("Kneipe"),
        "Kneipe",
        &[GameKind::Ramsch],
        BockMode::NextGame,
        BockStacking::Stacked,
    )
    .unwrap();
    assert_eq!(
        lib.rule_set("Kneipe").unwrap().bock_at_ramsch,
        BockAtRamsch::Expires
    );
    let mut t = table();
    t.use_rule_set(lib.rule_set("Kneipe").unwrap()).unwrap();
    assert_eq!(t.bock_at_ramsch, BockAtRamsch::Expires);
}

#[test]
fn mistakes_are_kept_only_for_a_corrected_price() {
    let mut t = table();
    let wrong = GameInput {
        price_override: Some(30),
        mistakes: vec![
            PriceMistake::Miscalculated,
            PriceMistake::BockForgotten,
            PriceMistake::Miscalculated,
        ],
        ..sauspiel(0, 1, 3)
    };
    let g = t.add_game(wrong).unwrap();
    assert_eq!(
        g.input.mistakes,
        vec![PriceMistake::BockForgotten, PriceMistake::Miscalculated],
        "sortiert, ohne Doppelte"
    );
    assert_eq!(
        g.input.price_error_by,
        vec![1, 3],
        "ohne Angabe beide Gewinner"
    );

    // Der verlangte Preis stimmt: Es gab nichts, was falsch war.
    let right = GameInput {
        price_override: Some(20),
        mistakes: vec![PriceMistake::Other],
        price_error_by: vec![2],
        ..sauspiel(1, 0, 2)
    };
    let g = t.add_game(right).unwrap();
    assert!(g.input.mistakes.is_empty() && g.input.price_error_by.is_empty());
    let none = t
        .add_game(GameInput {
            mistakes: vec![PriceMistake::Other],
            ..sauspiel(2, 0, 1)
        })
        .unwrap();
    assert!(none.input.mistakes.is_empty());
}

#[test]
fn only_a_participant_can_have_demanded_wrong() {
    let t = table();
    let input = GameInput {
        price_override: Some(30),
        price_error_by: vec![9],
        ..sauspiel(0, 1, 3)
    };
    assert!(t.preview_game(input).is_err());
}

#[test]
fn statistics_count_wrong_demands_per_player_and_kind() {
    let mut t = table();
    for (by, kinds) in [
        (1, vec![PriceMistake::Miscalculated]),
        (
            1,
            vec![PriceMistake::BockForgotten, PriceMistake::WrongLaufende],
        ),
        (3, vec![PriceMistake::Miscalculated]),
    ] {
        t.add_game(GameInput {
            price_override: Some(30),
            mistakes: kinds,
            price_error_by: vec![by],
            ..sauspiel(0, 1, 3)
        })
        .unwrap();
    }
    t.add_game(sauspiel(1, 0, 2)).unwrap();
    let s = table_stats(&t);
    assert_eq!(s.corrected_games, 3);
    assert_eq!(s.mistake_counts[&PriceMistake::Miscalculated], 2);
    assert_eq!(s.mistake_counts[&PriceMistake::BockForgotten], 1);
    assert_eq!(s.players[1].wrong_demands, 2);
    assert_eq!(s.players[3].wrong_demands, 1);
    assert_eq!(s.players[0].wrong_demands, 0);
    assert_eq!(s.players[1].wrong_by_kind[&PriceMistake::Miscalculated], 1);
    let csv = t.to_csv();
    let header: Vec<&str> = csv.lines().next().unwrap().split(',').collect();
    assert!(header.contains(&"Falsch_verlangt_von") && header.contains(&"Fehlerarten"));
    assert!(csv.contains("Bockrunde vergessen; Laufende nicht alle oder keine"));
}

fn evening(players: &[&str], games: Vec<GameInput>) -> Table {
    let names = players.iter().map(|s| s.to_string()).collect();
    let mut t = Table::new("Abend", names, 0, Tariff::default()).unwrap();
    for g in games {
        t.add_game(g).unwrap();
    }
    t
}

#[test]
fn overall_stats_merge_players_by_name_across_evenings() {
    let a = evening(
        &["Anna", "Ben", "Cem", "Dora"],
        vec![sauspiel(0, 1, 3), sauspiel(1, 0, 2)],
    );
    // Am zweiten Abend sitzen andere Sitzplätze: Ben ist jetzt Platz 0.
    let b = evening(
        &["Ben", "Eva", "Anna", "Dora"],
        vec![GameInput {
            knockers: vec![2],
            ..sauspiel(0, 0, 3)
        }],
    );
    let all = overall_stats(&[&a, &b]);
    assert_eq!((all.evenings, all.games), (2, 3));
    let by_name = |n: &str| all.players.iter().find(|p| p.name == n).unwrap();
    assert_eq!(all.players.len(), 5);
    assert_eq!(by_name("Anna").evenings, 2);
    assert_eq!(by_name("Eva").evenings, 1);
    let ta = table_stats(&a);
    let tb = table_stats(&b);
    assert_eq!(
        by_name("Ben").balance,
        ta.players[1].balance + tb.players[0].balance
    );
    assert_eq!(
        by_name("Ben").evening_balances,
        vec![ta.players[1].balance, tb.players[0].balance]
    );
    assert_eq!(
        by_name("Anna").knocks.games,
        1,
        "Anna hat am zweiten Abend geklopft"
    );
    assert_eq!(by_name("Anna").games_played, 3);
    assert_eq!(all.knocked_games, 1);
    // Das Geld bleibt über alle Abende null.
    assert_eq!(all.players.iter().map(|p| p.balance).sum::<Cents>(), 0);
}

#[test]
fn overall_stats_count_how_often_prices_were_doubled() {
    let t = evening(
        &["Anna", "Ben", "Cem", "Dora"],
        vec![
            sauspiel(0, 1, 3),
            GameInput {
                knockers: vec![2],
                ..sauspiel(1, 0, 2)
            },
            GameInput {
                knockers: vec![2, 3],
                ..sauspiel(2, 1, 3)
            },
            GameInput {
                knockers: vec![2, 3],
                bock: true,
                ..sauspiel(3, 0, 1)
            },
        ],
    );
    let all = overall_stats(&[&t]);
    assert_eq!(all.by_doublings[&0], 1);
    assert_eq!(all.by_doublings[&1], 1);
    assert_eq!(all.by_doublings[&2], 1);
    assert_eq!(all.by_doublings[&3], 1);
    assert_eq!(all.doubled_at_least(1), 3);
    assert_eq!(all.doubled_at_least(2), 2);
    let anna = &all.players.iter().find(|p| p.name == "Anna").unwrap();
    assert_eq!(anna.doubled_games, 3);
    assert_eq!(anna.quadrupled_games, 2);
    assert_eq!(overall_stats(&[]).games, 0);
}

#[test]
fn a_ramsch_that_ignores_bock_does_not_count_as_doubled() {
    let mut t = table();
    t.tariff.ramsch_bock_doubles = false;
    t.bock_mode = BockMode::NextGame;
    t.add_game(GameInput {
        bock: true,
        ..ramsch(0, 1)
    })
    .unwrap();
    assert_eq!(overall_stats(&[&t]).doubled_at_least(1), 0);
    assert_eq!(overall_stats(&[&t]).bock_games, 1);
}

#[test]
fn debug_evenings_are_a_flag_that_old_files_do_not_have() {
    let mut t = table();
    assert!(!t.debug);
    t.debug = true;
    assert!(Table::from_json(&t.to_json().unwrap()).unwrap().debug);
    let old: String = t
        .to_json()
        .unwrap()
        .lines()
        .filter(|l| !l.contains("\"debug\""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!Table::from_json(&old).unwrap().debug);
}

#[test]
fn schneider_and_schwarz_do_not_add_to_a_tout() {
    let t = Tariff::default();
    let solo = |tout, schneider, schwarz| GameInput {
        tout,
        schneider,
        schwarz,
        laufende: 3,
        ..GameInput::new(0, GameKind::Wenz, 1)
    };
    // Ohne Tout: 50 + 10 Schneider + 10 Schwarz + 3 mal 10 Laufende.
    assert_eq!(calculated_value(&t, &solo(false, true, true)), 100);
    // Mit Tout bleiben nur Grundpreis und Laufende, danach mal 2.
    assert_eq!(
        calculated_value(&t, &solo(true, false, false)),
        (50 + 30) * 2
    );
    assert_eq!(
        calculated_value(&t, &solo(true, true, false)),
        (50 + 30) * 2
    );
    assert_eq!(calculated_value(&t, &solo(true, true, true)), (50 + 30) * 2);
}

#[test]
fn a_tout_still_records_schneider_and_schwarz_for_the_statistics() {
    let mut t = table();
    t.add_game(GameInput {
        tout: true,
        schneider: true,
        schwarz: true,
        ..GameInput::new(0, GameKind::Wenz, 1)
    })
    .unwrap();
    let s = table_stats(&t);
    assert_eq!((s.schneider_games, s.schwarz_games), (1, 1));
    assert_eq!(t.games()[0].value, 100);
}

#[test]
fn schneider_and_schwarz_can_be_forgotten_separately() {
    assert_ne!(
        PriceMistake::SchneiderMissed.label(),
        PriceMistake::SchwarzMissed.label()
    );
    assert!(PriceMistake::ALL.contains(&PriceMistake::SchwarzMissed));
    assert_eq!(PriceMistake::ALL.len(), 12);
}

fn sample_files() -> Vec<(String, Vec<u8>)> {
    let lib = Library::with_defaults();
    let mut running = table();
    running.add_game(sauspiel(0, 1, 3)).unwrap();
    let mut past = table();
    past.add_game(sauspiel(0, 2, 1)).unwrap();
    vec![
        (LIBRARY_FILE.into(), lib.to_json().unwrap().into_bytes()),
        (CURRENT_FILE.into(), running.to_json().unwrap().into_bytes()),
        (
            "archiv/schafkopf-tisch-1000.json".into(),
            past.to_json().unwrap().into_bytes(),
        ),
        (
            "archiv/schafkopf-spiele-1000.csv".into(),
            past.to_csv().into_bytes(),
        ),
        ("INFO.txt".into(), "Hallo".as_bytes().to_vec()),
    ]
}

#[test]
fn a_backup_roundtrips_through_a_zip() {
    let files = sample_files();
    let zip = pack(&files).unwrap();
    assert_eq!(&zip[..2], b"PK");
    let back = unpack(&zip).unwrap();
    assert_eq!(back, files);
    let backup = read_backup(&back);
    assert_eq!(backup.library, Some(Library::with_defaults()));
    assert_eq!(backup.current.as_ref().unwrap().games().len(), 1);
    assert_eq!(backup.past.len(), 1);
    assert_eq!(backup.past[0].0, 1000);
    assert!(backup.unreadable.is_empty());
}

#[test]
fn a_zip_repacked_in_a_folder_still_reads() {
    let files: Vec<_> = sample_files()
        .into_iter()
        .map(|(n, d)| (format!("meine-sicherung/{n}"), d))
        .collect();
    let backup = read_backup(&unpack(&pack(&files).unwrap()).unwrap());
    assert!(backup.library.is_some() && backup.current.is_some());
    assert_eq!(backup.past.len(), 1);
}

#[test]
fn broken_and_unsafe_zip_content_is_not_trusted() {
    assert!(matches!(
        unpack(b"das ist keine zip"),
        Err(Error::Backup(_))
    ));
    let evil = pack(&[
        ("../../etc/passwd".into(), b"x".to_vec()),
        ("/abs/pfad.json".into(), b"x".to_vec()),
        ("a\\b.json".into(), b"x".to_vec()),
        ("ok.txt".into(), b"x".to_vec()),
    ])
    .unwrap();
    let files = unpack(&evil).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].0, "ok.txt");

    let broken = vec![
        (LIBRARY_FILE.to_string(), b"{kaputt".to_vec()),
        (CURRENT_FILE.to_string(), b"[]".to_vec()),
        (
            "archiv/schafkopf-tisch-5.json".to_string(),
            vec![0xff, 0xfe],
        ),
    ];
    let backup = read_backup(&broken);
    assert!(backup.library.is_none() && backup.current.is_none());
    assert_eq!(
        backup.unreadable.len(),
        2,
        "die Datei mit ungültigem UTF-8 zählt nicht dazu"
    );
}

#[test]
fn libraries_merge_without_touching_what_exists() {
    let mut mine = Library::with_defaults();
    mine.add_player("Anna").unwrap();
    let mut theirs = Library::with_defaults();
    theirs.add_player("anna").unwrap();
    theirs.add_player("Ben").unwrap();
    theirs
        .save_rule_set_with(None, "Kneipe", &[GameKind::Wenz], BockMode::NextRound)
        .unwrap();
    theirs
        .save_tariff(
            None,
            "10/20",
            Tariff {
                sauspiel: 10,
                ..Tariff::default()
            },
        )
        .unwrap();
    let before_default = mine.default_rule_set().name.clone();

    let report = mine.merge_from(&theirs, true, true, true);
    assert_eq!(
        (report.players, report.rule_sets, report.tariffs),
        (1, 1, 1)
    );
    assert!(
        report.kept >= 3,
        "Anna, die zwei Standard-Regelwerke und der Standardtarif waren schon da"
    );
    assert!(mine.players().contains(&"Ben".to_string()));
    assert!(mine.rule_set("Kneipe").is_some() && mine.tariff("10/20").is_some());
    assert_eq!(mine.default_rule_set().name, before_default);
    assert_eq!(
        mine.players()
            .iter()
            .filter(|p| p.to_lowercase() == "anna")
            .count(),
        1
    );

    // Nur gewünschte Teile.
    let mut only_players = Library::with_defaults();
    let report = only_players.merge_from(&theirs, true, false, false);
    assert_eq!(
        (report.players, report.rule_sets, report.tariffs),
        (2, 0, 0)
    );
    assert!(only_players.rule_set("Kneipe").is_none());
}

fn wrong_price(input: GameInput) -> GameInput {
    GameInput {
        price_override: Some(input.price_override.unwrap_or(999)),
        ..input
    }
}

#[test]
fn only_the_winners_can_have_demanded_wrong() {
    let mut t = table();
    // Sauspiel gewonnen: Spieler 1 und Partner 3 haben falsch verlangt.
    let g = t.add_game(wrong_price(sauspiel(0, 1, 3))).unwrap();
    assert_eq!(g.input.winners(), vec![1, 3]);
    assert_eq!(g.input.price_error_by, vec![1, 3]);

    // Verloren: Es verlangen die Gegner, nicht der Spieler und sein Partner.
    let lost = GameInput {
        won: false,
        ..sauspiel(1, 0, 2)
    };
    let g = t.add_game(wrong_price(lost)).unwrap();
    assert_eq!(g.input.winners(), vec![3, 1]);
    assert_eq!(g.input.price_error_by, vec![3, 1]);

    // Eine Angabe zu einem Verlierer wird verworfen, die zum Gewinner bleibt.
    let picked = GameInput {
        price_error_by: vec![0, 2],
        ..wrong_price(GameInput {
            won: false,
            ..sauspiel(2, 3, 1)
        })
    };
    let g = t.add_game(picked).unwrap();
    assert_eq!(g.input.winners(), vec![0, 2]);
    assert_eq!(g.input.price_error_by, vec![0, 2]);
    let only_one = GameInput {
        price_error_by: vec![0, 3],
        ..wrong_price(GameInput {
            won: false,
            ..sauspiel(3, 2, 0)
        })
    };
    let g = t.add_game(only_one).unwrap();
    assert_eq!(
        g.input.price_error_by,
        vec![3],
        "der Verlierer 0 wird verworfen"
    );
}

#[test]
fn a_lost_solo_is_demanded_by_all_three_opponents_and_a_won_one_by_the_player() {
    let mut t = table();
    let lost = GameInput {
        won: false,
        ..GameInput::new(0, GameKind::Wenz, 1)
    };
    let g = t.add_game(wrong_price(lost)).unwrap();
    assert_eq!(g.input.price_error_by.len(), 3);
    assert!(!g.input.price_error_by.contains(&1));
    let won = GameInput::new(1, GameKind::Wenz, 2);
    let g = t.add_game(wrong_price(won)).unwrap();
    assert_eq!(g.input.price_error_by, vec![2]);
}

#[test]
fn in_a_ramsch_the_players_who_did_not_lose_are_the_winners() {
    let mut t = table();
    let g = t.add_game(wrong_price(ramsch(0, 1))).unwrap();
    assert_eq!(g.input.winners().len(), 3);
    assert!(!g.input.winners().contains(&1));
}

#[test]
fn wrong_demands_count_for_every_winner_and_old_files_with_one_player_still_load() {
    let mut t = table();
    t.add_game(GameInput {
        mistakes: vec![PriceMistake::Miscalculated],
        ..wrong_price(sauspiel(0, 1, 3))
    })
    .unwrap();
    let s = table_stats(&t);
    assert_eq!(
        (s.players[1].wrong_demands, s.players[3].wrong_demands),
        (1, 1)
    );
    assert_eq!(s.players[0].wrong_demands + s.players[2].wrong_demands, 0);
    assert!(t.to_csv().contains("Ben; Dora"));

    // Alte Dateien: eine Zahl oder null statt einer Liste.
    let json = t.to_json().unwrap();
    let as_one = json.replace(
        "\"price_error_by\": [\n          1,\n          3\n        ]",
        "\"price_error_by\": 1",
    );
    assert_ne!(json, as_one, "die Ersetzung muss greifen");
    let back = Table::from_json(&as_one).unwrap();
    assert_eq!(back.games()[0].input.price_error_by, vec![1]);
    let as_null = json.replace(
        "\"price_error_by\": [\n          1,\n          3\n        ]",
        "\"price_error_by\": null",
    );
    assert!(Table::from_json(&as_null).unwrap().games()[0]
        .input
        .price_error_by
        .is_empty());
}

fn asked(price: Cents) -> GameInput {
    GameInput {
        price_override: Some(price),
        ..sauspiel(0, 1, 3)
    }
}

#[test]
fn too_much_demanded_stands_if_nobody_notices() {
    let mut t = table();
    let g = t.add_game(asked(30)).unwrap();
    assert_eq!((g.value, g.calculated_value), (30, 20));
    assert_eq!(g.payouts, vec![-30, 30, -30, 30]);
    assert!(g.overcharged() && g.demanded_wrong() && !g.input.reclaimed);
    let s = table_stats(&t);
    assert_eq!((s.overcharged_games, s.reclaimed_games), (1, 0));
    let by = |seat: usize| &s.players[seat];
    // Die Gewinner (1 und 3) bekamen je 10 zu viel, die Verlierer (0 und 2) zahlten je 10 zu viel.
    assert_eq!((by(1).overcharge_gain, by(3).overcharge_gain), (10, 10));
    assert_eq!((by(0).overcharge_loss, by(2).overcharge_loss), (10, 10));
    assert_eq!(by(1).over_demands, 1);
    assert_eq!(s.overcharge_cents, 20);
    assert_eq!(s.reclaimed_cents, 0);
}

#[test]
fn the_losers_can_reclaim_what_was_demanded_too_much() {
    let mut t = table();
    let g = t
        .add_game(GameInput {
            reclaimed: true,
            ..asked(30)
        })
        .unwrap();
    assert_eq!(
        (g.value, g.calculated_value),
        (10, 20),
        "die Verlierer holen das Doppelte des Mehrbetrags zurück: 30 - 2 * 10"
    );
    assert_eq!(g.payouts, vec![-10, 10, -10, 10]);
    assert_eq!(
        g.input.price_override,
        Some(30),
        "der verlangte Preis bleibt für die Statistik"
    );
    assert_eq!(
        g.input.reclaimed_by,
        vec![2, 0],
        "ohne Angabe alle Verlierer"
    );
    assert_eq!(t.balances(), vec![-10, 10, -10, 10]);
    let s = table_stats(&t);
    assert_eq!((s.overcharged_games, s.reclaimed_games), (1, 1));
    assert_eq!((s.overcharge_cents, s.reclaimed_cents), (20, 40));
    assert_eq!(
        s.overcharge_net_cents, -20,
        "die Gewinner bekommen 10 weniger als zustand, je 10"
    );
    assert_eq!((s.players[0].reclaims, s.players[2].reclaims), (1, 1));
    assert_eq!(s.players[1].reclaims, 0);
    assert_eq!(
        (s.players[0].reclaim_saved, s.players[2].reclaim_saved),
        (20, 20)
    );
    assert_eq!(
        (s.players[1].reclaim_returned, s.players[3].reclaim_returned),
        (20, 20)
    );
    assert_eq!(s.players[1].overcharge_gain, 0);
    // Wer falsch verlangt hat, steht trotzdem drin.
    assert_eq!(s.players[1].wrong_demands, 1);
    assert_eq!(s.corrected_games, 1);
}

#[test]
fn only_a_loser_can_reclaim_and_only_when_it_was_too_much_and_the_tariff_allows_it() {
    let mut t = table();
    let g = t
        .add_game(GameInput {
            reclaimed: true,
            reclaimed_by: vec![1, 2],
            ..asked(30)
        })
        .unwrap();
    assert_eq!(
        g.input.reclaimed_by,
        vec![2],
        "1 ist ein Gewinner und kann nicht zurückverlangen"
    );

    // Zu wenig verlangt: Es gibt nichts zurückzuverlangen.
    let g = t
        .add_game(GameInput {
            reclaimed: true,
            ..GameInput {
                price_override: Some(10),
                ..sauspiel(1, 0, 2)
            }
        })
        .unwrap();
    assert!(!g.input.reclaimed && g.input.reclaimed_by.is_empty());
    assert_eq!(g.value, 10);

    // Der verlangte Preis stimmt: weder falsch verlangt noch zurückverlangt.
    let g = t
        .add_game(GameInput {
            reclaimed: true,
            ..GameInput {
                price_override: Some(20),
                ..sauspiel(2, 0, 3)
            }
        })
        .unwrap();
    assert!(!g.input.reclaimed && !g.demanded_wrong());

    // Der Tarif erlaubt es nicht: Der verlangte Preis gilt.
    t.set_tariff(Tariff {
        reclaim_enabled: false,
        ..Tariff::default()
    });
    let g = t
        .add_game(GameInput {
            reclaimed: true,
            ..GameInput {
                price_override: Some(30),
                ..sauspiel(3, 0, 1)
            }
        })
        .unwrap();
    assert!(!g.input.reclaimed);
    assert_eq!(g.value, 30);
}

#[test]
fn too_little_demanded_costs_the_winners_and_helps_the_losers() {
    let mut t = table();
    t.add_game(asked(10)).unwrap();
    let s = table_stats(&t);
    assert_eq!((s.undercharged_games, s.overcharged_games), (1, 0));
    assert_eq!(
        (
            s.players[1].undercharge_missed,
            s.players[3].undercharge_missed
        ),
        (10, 10)
    );
    assert_eq!(
        (
            s.players[0].undercharge_benefit,
            s.players[2].undercharge_benefit
        ),
        (10, 10)
    );
    assert_eq!(s.players[1].under_demands, 1);
    assert_eq!(s.overcharge_cents, 0);
}

#[test]
fn a_solo_overcharge_is_counted_per_opponent() {
    let mut t = table();
    let solo = GameInput {
        price_override: Some(80),
        reclaimed: false,
        ..GameInput::new(0, GameKind::Wenz, 1)
    };
    t.add_game(solo).unwrap();
    let s = table_stats(&t);
    // Der Spieler bekommt von drei Gegnern je 30 zu viel, jeder Gegner zahlt 30 zu viel.
    assert_eq!(s.players[1].overcharge_gain, 90);
    assert_eq!(s.players[0].overcharge_loss, 30);
    assert_eq!(s.overcharge_cents, 90);
}

#[test]
fn old_tariffs_allow_reclaiming_and_old_games_have_no_reclaim() {
    let mut json: serde_json::Value = serde_json::to_value(Tariff::default()).unwrap();
    json.as_object_mut().unwrap().remove("reclaim_enabled");
    assert!(
        serde_json::from_value::<Tariff>(json)
            .unwrap()
            .reclaim_enabled
    );
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    let json = t.to_json().unwrap();
    let old: String = json
        .lines()
        .filter(|l| !l.contains("\"reclaimed"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!Table::from_json(&old).unwrap().games()[0].input.reclaimed);
}

#[test]
fn overall_stats_add_up_reclaims_across_evenings() {
    let a = evening(
        &["Anna", "Ben", "Cem", "Dora"],
        vec![GameInput {
            reclaimed: true,
            ..asked(30)
        }],
    );
    let b = evening(&["Ben", "Anna", "Cem", "Dora"], vec![asked(30)]);
    let all = overall_stats(&[&a, &b]);
    assert_eq!((all.overcharged_games, all.reclaimed_games), (2, 1));
    assert_eq!((all.overcharge_cents, all.reclaimed_cents), (40, 40));
    let ben = all.players.iter().find(|p| p.name == "Ben").unwrap();
    assert!(
        ben.reclaim_returned + ben.overcharge_loss + ben.overcharge_gain + ben.reclaim_saved > 0
    );
    let csv = a.to_csv();
    assert!(csv
        .lines()
        .next()
        .unwrap()
        .contains("Zurueckverlangt,Zurueckverlangt_von"));
    assert!(csv.contains("ja,Cem; Anna"));
}

#[test]
fn the_losers_reclaim_twice_what_was_demanded_too_much() {
    // Das Spiel kostet 60, verlangt werden 80: Mehrbetrag 20, zurück 40, es bleiben 40.
    assert_eq!(reclaimed_value(60, 80), 40);
    assert_eq!(reclaimed_value(60, 60), 60, "nichts zu viel, nichts zurück");
    assert_eq!(reclaimed_value(60, 50), 50, "zu wenig verlangt bleibt");
    assert_eq!(reclaimed_value(60, 200), 0, "weniger als 0 wird es nie");
    assert_eq!(reclaimed_value(60, 120), 0);

    let mut t = table();
    t.set_tariff(Tariff {
        sauspiel: 60,
        ..Tariff::default()
    });
    let g = t
        .add_game(GameInput {
            reclaimed: true,
            ..asked(80)
        })
        .unwrap();
    assert_eq!((g.value, g.calculated_value), (40, 60));
    assert_eq!(g.payouts, vec![-40, 40, -40, 40]);
    let s = table_stats(&t);
    // Verlangt 80 gegen 60: je Gewinner 20 zu viel. Zurückgeholt 40 je Gewinner, am Ende 20 unter dem Preis.
    assert_eq!(s.overcharge_cents, 40);
    assert_eq!(s.reclaimed_cents, 80);
    assert_eq!(s.overcharge_net_cents, -40);
    assert_eq!(s.players[0].reclaim_saved, 40);
    assert_eq!(s.players[1].reclaim_returned, 40);

    // Ohne Zurückverlangen gilt der verlangte Preis.
    let g = t.add_game(asked(80)).unwrap();
    assert_eq!(g.value, 80);
}

#[test]
fn mistakes_fit_the_direction_of_the_wrong_price() {
    let high = PriceMistake::for_price(true);
    let low = PriceMistake::for_price(false);
    // Zu viel verlangt: dazugedichtet, nichts vergessen.
    assert!(high.contains(&PriceMistake::SchneiderInvented));
    assert!(high.contains(&PriceMistake::SchwarzInvented));
    assert!(high.contains(&PriceMistake::BockInvented));
    assert!(!high.contains(&PriceMistake::SchneiderMissed));
    assert!(!high.contains(&PriceMistake::SchwarzMissed));
    assert!(!high.contains(&PriceMistake::BockForgotten));
    // Zu wenig verlangt: vergessen, nichts dazugedichtet.
    assert!(low.contains(&PriceMistake::SchneiderMissed));
    assert!(!low.contains(&PriceMistake::SchneiderInvented));
    // Der Rest passt zu beidem.
    for m in [
        PriceMistake::WrongBase,
        PriceMistake::Miscalculated,
        PriceMistake::Other,
    ] {
        assert!(high.contains(&m) && low.contains(&m));
    }
    // Jede Fehlerart gibt es in mindestens einer Richtung.
    for m in PriceMistake::ALL {
        assert!(high.contains(&m) || low.contains(&m));
    }
}

#[test]
fn invented_schneider_is_stored_and_counted() {
    let mut t = table();
    let g = t
        .add_game(GameInput {
            price_override: Some(30),
            mistakes: vec![PriceMistake::SchneiderInvented],
            ..sauspiel(0, 1, 3)
        })
        .unwrap();
    assert!(g.overcharged());
    assert_eq!(g.input.mistakes, vec![PriceMistake::SchneiderInvented]);
    let s = table_stats(&t);
    assert_eq!(s.mistake_counts[&PriceMistake::SchneiderInvented], 1);
    let json = t.to_json().unwrap();
    assert_eq!(Table::from_json(&json).unwrap(), t);
}

#[test]
fn an_offence_costs_the_offender_a_penalty_from_every_other_player() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    let o = t
        .add_offence(OffenceKind::NotFollowed, 2, None, None, Some(100))
        .unwrap();
    assert_eq!((o.amount, o.after_games), (50, 1));
    assert_eq!(o.payouts, vec![50, 50, -150, 50]);
    assert_eq!(t.balances(), vec![-20 + 50, 20 + 50, -20 - 150, 20 + 50]);
    assert_eq!(t.balances().iter().sum::<Cents>(), 0);
    // Eigener Betrag.
    let o = t
        .add_offence(OffenceKind::Collusion, 0, None, Some(100), None)
        .unwrap();
    assert_eq!(o.payouts, vec![-300, 100, 100, 100]);
    assert_eq!(t.last_activity(), Some(100));
    // Zurücknehmen.
    assert!(t.remove_offence(1).is_some());
    assert!(t.remove_offence(5).is_none());
    assert_eq!(t.balances().iter().sum::<Cents>(), 0);
    assert_eq!(t.offences.len(), 1);
}

#[test]
fn offences_check_the_seat_and_the_amount() {
    let mut t = table();
    assert!(t
        .add_offence(OffenceKind::Other, 9, None, None, None)
        .is_err());
    assert!(t
        .add_offence(OffenceKind::Other, 1, None, Some(-5), None)
        .is_err());
    t.leave(3, None).unwrap();
    assert!(t
        .add_offence(OffenceKind::Other, 3, None, None, None)
        .is_err());
    // Wer nicht am Tisch sitzt, zahlt und bekommt nichts.
    let o = t
        .add_offence(OffenceKind::Other, 1, None, None, None)
        .unwrap();
    assert_eq!(o.payouts, vec![50, -100, 50, 0]);
}

#[test]
fn a_misdeal_keeps_the_dealer_and_is_free_unless_the_tariff_says_so() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    assert_eq!(t.next_dealer(), 1);
    // Standard: Neumischen kostet nichts, wird aber gezählt.
    let o = t
        .add_offence(OffenceKind::Misdeal, t.next_dealer(), None, None, None)
        .unwrap();
    assert_eq!((o.amount, o.payouts.clone()), (0, vec![0; 4]));
    assert_eq!(t.next_dealer(), 1, "derselbe Geber mischt neu");
    t.set_tariff(Tariff {
        misdeal_penalty: 20,
        ..Tariff::default()
    });
    let o = t
        .add_offence(OffenceKind::Misdeal, 1, None, None, None)
        .unwrap();
    assert_eq!(o.payouts, vec![20, -60, 20, 20]);
    let s = table_stats(&t);
    assert_eq!((s.misdeals, s.offences), (2, 0));
    assert_eq!(s.penalty_cents, 60);
    assert_eq!(s.players[1].misdeals, 2);
    assert_eq!(s.players[1].penalty_paid, 60);
    assert_eq!(s.players[0].penalty_received, 20);
}

#[test]
fn without_penalties_offences_are_counted_but_free() {
    let mut t = table();
    t.set_tariff(Tariff {
        penalties_enabled: false,
        misdeal_penalty: 20,
        ..Tariff::default()
    });
    assert_eq!(t.penalty_for(OffenceKind::NotFollowed), 0);
    assert_eq!(t.penalty_for(OffenceKind::Misdeal), 0);
    let o = t
        .add_offence(OffenceKind::NotFollowed, 1, None, Some(100), None)
        .unwrap();
    assert_eq!((o.amount, o.payouts.clone()), (0, vec![0; 4]));
    let s = table_stats(&t);
    assert_eq!((s.offences, s.penalty_cents), (1, 0));
    assert_eq!(s.players[1].offences, 1);
    assert_eq!(s.players[1].offences_by_kind[&OffenceKind::NotFollowed], 1);
}

#[test]
fn penalties_flow_into_balances_stats_and_the_history_chart() {
    let mut t = table();
    t.add_offence(OffenceKind::OutOfTurn, 0, None, None, None)
        .unwrap();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.add_offence(OffenceKind::ShowedCards, 2, None, None, None)
        .unwrap();
    t.add_game(sauspiel(1, 1, 3)).unwrap();
    let history = balance_history(&t);
    assert_eq!(history.len(), 3);
    assert_eq!(
        history[0],
        vec![-150, 50, 50, 50],
        "Strafe vor dem ersten Spiel"
    );
    // Die Strafe nach Spiel 1 steckt im Stand nach Spiel 1.
    assert_eq!(history[1], vec![-120, 120, -120, 120]);
    assert_eq!(history[2], vec![-140, 140, -140, 140]);
    assert_eq!(
        history[2],
        t.balances(),
        "der Endstand stimmt mit den Kontoständen überein"
    );
    let s = table_stats(&t);
    for (p, b) in s.players.iter().zip(t.balances()) {
        assert_eq!(p.balance, b);
    }
    assert_eq!(s.offences, 2);
}

#[test]
fn old_files_have_no_offences_and_old_tariffs_have_penalties_on() {
    let mut json: serde_json::Value = serde_json::to_value(Tariff::default()).unwrap();
    for key in ["penalties_enabled", "offence_penalty", "misdeal_penalty"] {
        json.as_object_mut().unwrap().remove(key);
    }
    let old: Tariff = serde_json::from_value(json).unwrap();
    assert!(old.penalties_enabled);
    assert_eq!((old.offence_penalty, old.misdeal_penalty), (50, 0));
    let mut t = table();
    t.add_offence(OffenceKind::Other, 0, None, None, None)
        .unwrap();
    let mut value: serde_json::Value = serde_json::from_str(&t.to_json().unwrap()).unwrap();
    value.as_object_mut().unwrap().remove("offences");
    let loaded = Table::from_json(&value.to_string()).unwrap();
    assert!(loaded.offences.is_empty());
    assert_eq!(loaded.balances(), vec![0; 4]);
    // Ein neuer Spieler bekommt für frühere Strafen 0.
    let mut t = table();
    t.add_offence(OffenceKind::Other, 0, None, None, None)
        .unwrap();
    t.add_player("Eva", None, None).unwrap();
    assert_eq!(t.offences[0].payouts.len(), 5);
    assert_eq!(t.balances().iter().sum::<Cents>(), 0);
}

#[test]
fn overall_stats_add_up_offences_across_evenings() {
    let mut a = table();
    a.add_offence(OffenceKind::NotFollowed, 1, None, None, None)
        .unwrap();
    a.add_offence(OffenceKind::Misdeal, 0, None, None, None)
        .unwrap();
    let mut b = Table::new(
        "B",
        ["Ben", "Anna", "Cem", "Dora"].map(String::from).to_vec(),
        0,
        Tariff::default(),
    )
    .unwrap();
    b.add_offence(OffenceKind::NotFollowed, 0, None, None, None)
        .unwrap();
    let all = overall_stats(&[&a, &b]);
    assert_eq!((all.offences, all.misdeals), (2, 1));
    assert_eq!(all.offences_by_kind[&OffenceKind::NotFollowed], 2);
    assert_eq!(all.penalty_cents, 150 + 150);
    let ben = all.players.iter().find(|p| p.name == "Ben").unwrap();
    assert_eq!((ben.offences, ben.penalty_paid), (2, 300));
}

#[test]
fn offences_appear_in_the_protocol_csv_in_the_order_of_the_games() {
    let mut t = table();
    t.add_offence(OffenceKind::NotFollowed, 2, None, None, Some(50))
        .unwrap();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.add_offence(OffenceKind::Misdeal, 1, None, Some(20), None)
        .unwrap();
    let csv = t.events_csv();
    let lines: Vec<&str> = csv.lines().collect();
    assert!(lines[0].ends_with("Nach_Spiel_Nr,Strafe_je_Mitspieler_Cent"));
    assert!(lines[2].contains("Vergehen: Nicht bedient,Cem,,0,50"));
    assert!(lines[3].contains("Vergehen: Falsch gegeben (neu gemischt),Ben,,1,20"));
}

fn farbsolo(tout: bool, sie: bool) -> GameInput {
    GameInput {
        suit: Some(Suit::Eichel),
        tout,
        sie,
        ..GameInput::new(0, GameKind::Farbsolo, 1)
    }
}

#[test]
fn sie_is_the_step_up_from_tout_in_a_farbsolo() {
    let mut t = table();
    // Tout: (50 + 3 Laufende zählen nicht, es gibt keine) mal 2.
    let g = t.add_game(farbsolo(true, false)).unwrap();
    assert_eq!(g.value, 100);
    // Sie: immer 8 Laufende (50 + 80), Schneider und Schwarz zählen nicht, keine Verdopplung über das Tout hinaus.
    let g = t
        .add_game(GameInput {
            schneider: true,
            schwarz: true,
            laufende: 2,
            gegenlauf: true,
            ..farbsolo(true, true)
        })
        .unwrap();
    assert_eq!((g.input.laufende, g.input.gegenlauf), (8, false));
    assert_eq!(g.value, (50 + 80) * 2);
    assert_eq!(g.payouts, vec![-260, 780, -260, -260]);
    assert_eq!(g.input.doublings(), 1);
    // Bock und Klopfen kommen darüber.
    let g = t
        .add_game(GameInput {
            knockers: vec![2],
            ..farbsolo(true, true)
        })
        .unwrap();
    assert_eq!(g.value, (50 + 80) * 4);
}

#[test]
fn sie_only_goes_with_tout_in_a_farbsolo() {
    let mut t = table();
    assert!(matches!(
        t.add_game(farbsolo(false, true)),
        Err(Error::SieNotAllowed)
    ));
    let wenz = GameInput {
        tout: true,
        sie: true,
        ..GameInput::new(0, GameKind::Wenz, 1)
    };
    assert!(matches!(t.add_game(wenz), Err(Error::SieNotAllowed)));
    assert!(t.games().is_empty());
    // Es gibt keine Spielart Sie, und alte Dateien ohne das Feld laden.
    assert_eq!(GameKind::ALL.len(), 9);
    let mut t = table();
    t.add_game(farbsolo(true, true)).unwrap();
    let json = t.to_json().unwrap();
    assert_eq!(Table::from_json(&json).unwrap(), t);
    let old: String = json
        .lines()
        .filter(|l| !l.contains("\"sie\""))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!Table::from_json(&old).unwrap().games()[0].input.sie);
    assert!(t.csv_header().contains(",Tout,Sie,"));
}

#[test]
fn a_penalty_goes_only_to_the_players_of_the_current_round() {
    let mut t = Table::new(
        "Fünf",
        ["Anna", "Ben", "Cem", "Dora", "Eva"]
            .map(String::from)
            .to_vec(),
        0,
        Tariff::default(),
    )
    .unwrap();
    // Geber Anna: Es spielen Ben, Cem, Eva (vor dem Geber) und Anna. Dora sitzt aus.
    assert_eq!(t.default_participants(0), vec![1, 2, 4, 0]);
    let o = t
        .add_offence(OffenceKind::NotFollowed, 2, None, None, None)
        .unwrap();
    assert_eq!(o.payouts, vec![50, 50, -150, 0, 50]);
    // Wer aussetzt, kann auch etwas verbrochen haben und zahlt den Mitspielern.
    let o = t
        .add_offence(OffenceKind::Other, 3, None, None, None)
        .unwrap();
    assert_eq!(o.payouts, vec![50, 50, 50, -200, 50]);
    // Die Empfänger lassen sich von Hand wählen, aber es müssen Anwesende sein.
    let o = t
        .add_offence(OffenceKind::Other, 1, Some(vec![0, 3]), Some(100), None)
        .unwrap();
    assert_eq!(o.payouts, vec![100, -200, 0, 100, 0]);
    assert!(t
        .add_offence(OffenceKind::Other, 1, Some(vec![9]), None, None)
        .is_err());
    // Nach dem Spiel von Geber Ben gibt Cem: Es spielen Dora, Eva, Ben und Cem, Eva setzt hier aus (Täter).
    t.add_game(GameInput {
        participants: vec![2, 3, 0, 1],
        ..sauspiel(1, 2, 0)
    })
    .unwrap();
    assert_eq!(t.default_penalty_recipients(4), vec![3, 1, 2]);
}

#[test]
fn doublings_are_split_by_level_for_the_evening_and_for_every_player() {
    let mut t = table();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.add_game(GameInput {
        knockers: vec![2],
        ..sauspiel(1, 1, 3)
    })
    .unwrap();
    t.add_game(GameInput {
        knockers: vec![2, 0],
        ..sauspiel(2, 1, 3)
    })
    .unwrap();
    t.add_game(GameInput {
        knockers: vec![2, 0],
        kontra_by: Some(0),
        re: true,
        ..sauspiel(3, 1, 3)
    })
    .unwrap();
    let s = table_stats(&t);
    // 0, 1, 2 und 4 Verdopplungen.
    assert_eq!(
        s.by_doublings.into_iter().collect::<Vec<_>>(),
        vec![(0, 1), (1, 1), (2, 1), (4, 1)]
    );
    // Ben spielt immer mit, Anna aussetzend nie ... Anna spielt, wenn sie nicht gibt: Geber 0 gibt im ersten Spiel.
    let ben = &s.players[1];
    assert_eq!(ben.by_doublings.values().sum::<u32>(), ben.games_played);
    assert_eq!(ben.by_doublings[&2], 1);
    let all = overall_stats(&[&t]);
    let ben = all.players.iter().find(|p| p.name == "Ben").unwrap();
    assert_eq!(ben.by_doublings[&4], 1);
    assert_eq!(ben.by_doublings.values().sum::<u32>(), ben.games_played);
}

#[test]
fn touts_and_sies_are_counted_per_player_and_across_evenings() {
    let lost_tout = GameInput {
        won: false,
        ..farbsolo(true, false)
    };
    let t1 = evening(
        &["Anna", "Ben", "Cem", "Dora"],
        vec![
            farbsolo(true, false),
            lost_tout.clone(),
            farbsolo(true, true),
            farbsolo(false, false),
        ],
    );
    let stats = table_stats(&t1);
    let ben = &stats.players[1];
    assert_eq!(
        (ben.touts.games, ben.touts.wins),
        (3, 2),
        "Sie zählt als Tout"
    );
    assert_eq!((ben.sies.games, ben.sies.wins), (1, 1));
    assert_eq!(stats.players[0].touts.games, 0);
    let t2 = evening(&["Anna", "Ben", "Cem", "Dora"], vec![lost_tout]);
    let all = overall_stats(&[&t1, &t2]);
    let ben = all.players.iter().find(|p| p.name == "Ben").unwrap();
    assert_eq!((ben.touts.games, ben.touts.wins), (4, 2));
    assert_eq!(ben.sies.games, 1);
}

#[test]
fn doubles_given_adds_knocking_kontra_and_re() {
    let t = evening(
        &["Anna", "Ben", "Cem", "Dora"],
        vec![
            GameInput {
                knockers: vec![2],
                kontra_by: Some(2),
                re: true,
                ..sauspiel(0, 1, 3)
            },
            GameInput {
                knockers: vec![2],
                ..sauspiel(1, 0, 2)
            },
        ],
    );
    let stats = table_stats(&t);
    assert_eq!(stats.players[2].doubles_given(), 3);
    assert_eq!(stats.players[1].doubles_given(), 1, "Re kommt vom Spieler");
    let all = overall_stats(&[&t]);
    let cem = all.players.iter().find(|p| p.name == "Cem").unwrap();
    assert_eq!(cem.doubles_given(), 3);
}

#[test]
fn reclaiming_never_takes_more_than_the_winners_received() {
    let mut t = table();
    // Berechnet 20, verlangt 600: Mehrbetrag 580, zurückverlangen dürften sie 1160, aber die Gewinner haben nur 600 bekommen.
    let g = t
        .add_game(GameInput {
            reclaimed: true,
            ..asked(600)
        })
        .unwrap();
    assert_eq!(g.value, 0, "der Preis sinkt höchstens auf 0, nie darunter");
    assert_eq!(
        g.payouts,
        vec![0, 0, 0, 0],
        "die Gewinner stehen wie vor dem Spiel da"
    );
    let s = table_stats(&t);
    assert_eq!(
        s.reclaimed_cents, 1200,
        "zurückgeholt werden die 600 je Gewinner, nicht 2 * 580"
    );
    assert_eq!(s.players[1].reclaim_returned, 600);
    assert_eq!(s.players[0].reclaim_saved, 600);
}

#[test]
fn re_is_given_by_one_player_of_the_declarer_team_after_a_kontra() {
    let mut t = table();
    let base = GameInput {
        kontra_by: Some(2),
        re: true,
        ..sauspiel(0, 1, 3)
    };
    for wrong in [0, 2] {
        assert_eq!(
            t.add_game(GameInput {
                re_by: Some(wrong),
                ..base.clone()
            })
            .unwrap_err(),
            Error::ReFromOtherTeam,
            "Platz {wrong} gehört nicht zur Spielerpartei"
        );
    }
    assert_eq!(
        t.add_game(GameInput {
            re_by: Some(3),
            re: false,
            ..base.clone()
        })
        .unwrap_err(),
        Error::ReFromOtherTeam,
        "Re von einem Spieler braucht Re"
    );
    t.add_game(GameInput {
        re_by: Some(3),
        ..base.clone()
    })
    .unwrap();
    t.add_game(base).unwrap();
    let s = table_stats(&t);
    assert_eq!(
        s.players[3].re_given.games, 1,
        "der Mitspieler hat Re gegeben"
    );
    assert_eq!(
        s.players[1].re_given.games, 1,
        "ohne Angabe (ältere Spiele) war es der Spieler"
    );
    assert!(t
        .to_csv_with(';')
        .lines()
        .nth(1)
        .unwrap()
        .contains(";Cem;ja;Dora;"));
}

#[test]
fn games_saved_before_re_by_still_load() {
    let mut t = table();
    t.add_game(GameInput {
        kontra_by: Some(2),
        re: true,
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    let json = t.to_json().unwrap().replace("\"re_by\": null,", "");
    let loaded = Table::from_json(&json).unwrap();
    assert_eq!(loaded.games()[0].input.re_giver(), Some(1));
}

#[test]
fn the_csv_names_everyone_who_doubled_and_the_demanded_price() {
    let mut t = table();
    t.add_game(GameInput {
        knockers: vec![0, 2],
        kontra_by: Some(2),
        re: true,
        re_by: Some(3),
        bock: true,
        price_override: Some(300),
        ..sauspiel(0, 1, 3)
    })
    .unwrap();
    let csv = t.to_csv_with(',');
    let header: Vec<&str> = csv.lines().next().unwrap().split(',').collect();
    let row: Vec<&str> = csv.lines().nth(1).unwrap().split(',').collect();
    let at = |name: &str| row[header.iter().position(|h| *h == name).unwrap()];
    assert_eq!(at("Geklopft"), "Anna; Cem");
    assert_eq!(at("Kontra"), "Cem");
    assert_eq!(at("Re_von"), "Dora");
    assert_eq!(
        at("Doppler_Anzahl"),
        "5",
        "zweimal Klopfen, Kontra, Re, Bock"
    );
    assert_eq!(at("Verlangter_Preis_Cent"), "300");
}

fn evening_with_names() -> Table {
    let mut t = evening(&["Anna", "Ben", "Cem", "Dora"], vec![]);
    t.created_at = Some(1_700_000_000);
    t.use_tariff(Some("20/50".into()), Tariff::default());
    t.use_rules(Some("Unsere Runde".into()), &GameKind::ALL)
        .unwrap();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t.add_game(sauspiel(1, 0, 2)).unwrap();
    let cheap = Tariff {
        sauspiel: 10,
        ..Tariff::default()
    };
    t.use_tariff(Some("10/50".into()), cheap);
    t.add_game(sauspiel(2, 1, 3)).unwrap();
    t.use_tariff(Some("20/50".into()), Tariff::default());
    t.add_game(sauspiel(3, 1, 3)).unwrap();
    t.add_game(sauspiel(0, 1, 3)).unwrap();
    t
}

#[test]
fn the_meta_counts_games_per_tariff_and_how_often_it_changed() {
    let t = evening_with_names();
    let meta = table_meta(&t);
    assert_eq!(
        meta.tariffs
            .iter()
            .map(|u| (u.name.as_str(), u.games))
            .collect::<Vec<_>>(),
        vec![("20/50", 4), ("10/50", 1)]
    );
    assert_eq!(meta.tariff_changes, 2);
    assert_eq!(meta.rule_sets.len(), 1);
    assert_eq!(meta.rule_set_changes, 0);
    // Wird ein Spiel bearbeitet, bleibt sein Tarifname.
    let mut t = t;
    t.replace_game(2, sauspiel(2, 0, 3)).unwrap();
    assert_eq!(t.games()[2].tariff_name.as_deref(), Some("10/50"));
    let csv = t.to_csv_with(',');
    assert!(csv.lines().next().unwrap().contains("Tarif,Regelwerk"));
    assert!(csv.lines().nth(3).unwrap().contains("10/50,Unsere Runde"));
}

#[test]
fn games_without_names_are_grouped_as_custom_tariffs_and_old_games_load() {
    let mut t = evening(&["Anna", "Ben", "Cem", "Dora"], vec![sauspiel(0, 1, 3)]);
    t.set_tariff(Tariff {
        sauspiel: 30,
        ..Tariff::default()
    });
    t.add_game(sauspiel(1, 1, 3)).unwrap();
    let meta = table_meta(&t);
    let names: Vec<&str> = meta.tariffs.iter().map(|u| u.name.as_str()).collect();
    assert_eq!(names, vec!["Eigener Tarif 1", "Eigener Tarif 2"]);
    assert_eq!(meta.tariff_changes, 1);
    assert_eq!(meta.rule_sets[0].name, "Eigene Auswahl");
    let json = t
        .to_json()
        .unwrap()
        .replace("\"tariff_name\": null,", "")
        .replace("\"rule_set_name\": null,", "");
    assert_eq!(Table::from_json(&json).unwrap().games().len(), 2);
}

#[test]
fn the_meta_tells_when_players_came_and_went() {
    let mut t = evening(&["Anna", "Ben", "Cem", "Dora"], vec![sauspiel(0, 1, 3)]);
    t.created_at = Some(1_700_000_000);
    t.leave(2, Some(1_700_003_600)).unwrap();
    t.add_player("Emil", Some(0), Some(1_700_007_200)).unwrap();
    t.ended_at = Some(1_700_010_800);
    let meta = table_meta(&t);
    let by_name = |n: &str| meta.presence.iter().find(|p| p.name == n).unwrap().clone();
    let anna = by_name("Anna");
    assert!(anna.from_start && !anna.left_early);
    assert_eq!(anna.arrived, Some(1_700_000_000));
    let cem = by_name("Cem");
    assert!(cem.left_early);
    assert_eq!(
        (cem.left, cem.left_after_games),
        (Some(1_700_003_600), Some(1))
    );
    let emil = by_name("Emil");
    assert!(!emil.from_start);
    assert_eq!(
        (emil.arrived, emil.arrived_after_games),
        (Some(1_700_007_200), 1)
    );
    assert_eq!(meta.ended, Some(1_700_010_800));
}

#[test]
fn overall_stats_average_arrival_and_departure_times_per_player() {
    let mut a = evening(&["Anna", "Ben", "Cem", "Dora"], vec![sauspiel(0, 1, 3)]);
    a.created_at = Some(1_700_000_000); // 22:13 UTC
    a.ended_at = Some(1_700_010_000);
    let mut b = evening(&["Anna", "Ben", "Cem", "Dora"], vec![sauspiel(0, 1, 3)]);
    b.created_at = Some(1_700_000_000 + 7_200);
    b.ended_at = Some(1_700_000_000 + 7_200 + 7_200);
    b.leave(3, Some(1_700_000_000 + 7_200 + 3_600)).unwrap();
    let all = overall_stats(&[&a, &b]);
    let anna = all.players.iter().find(|p| p.name == "Anna").unwrap();
    assert_eq!(anna.arrivals.len(), 2);
    let (avg, min, max) = time_spread(&anna.arrivals).unwrap();
    assert!(min < avg && avg < max);
    let dora = all.players.iter().find(|p| p.name == "Dora").unwrap();
    assert_eq!(dora.early_departures, 1);
    assert_eq!(dora.departures.len(), 2);
    assert_eq!(all.evening_starts.len(), 2);
    assert_eq!(all.evening_facts.len(), 2);
    assert_eq!(all.evening_facts[1].games, 1);
    assert_eq!(all.evening_facts[1].length, Some(7_200));
    assert_eq!(all.tariffs[0].games, 2);
    assert_eq!(minutes_after_noon(1_700_000_000, 0), (22 * 60 + 13) - 720);
    assert_eq!(
        format_minutes_after_noon(minutes_after_noon(1_700_000_000, 0)),
        "22:13"
    );
}

#[test]
fn the_csv_has_the_tariff_values_of_each_game() {
    let mut t = evening(&["Anna", "Ben", "Cem", "Dora"], vec![sauspiel(0, 1, 3)]);
    t.set_tariff(Tariff {
        sauspiel: 30,
        laufende: 20,
        ..Tariff::default()
    });
    t.add_game(sauspiel(1, 1, 3)).unwrap();
    let csv = t.to_csv_with(',');
    let mut lines = csv.lines();
    let header: Vec<&str> = lines.next().unwrap().split(',').collect();
    let value = |line: &str, column: &str| {
        let at = header.iter().position(|h| *h == column).unwrap();
        line.split(',').nth(at).unwrap().to_owned()
    };
    let (first, second) = (lines.next().unwrap(), lines.next().unwrap());
    assert_eq!(value(first, "Tarif_Sauspiel_Cent"), "20");
    assert_eq!(value(second, "Tarif_Sauspiel_Cent"), "30");
    assert_eq!(value(first, "Tarif_Laufende_Cent"), "10");
    assert_eq!(value(second, "Tarif_Laufende_Cent"), "20");
    assert_eq!(value(second, "Tarif_Zurueckverlangen"), "ja");
}
