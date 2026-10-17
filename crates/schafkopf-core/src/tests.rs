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
