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
