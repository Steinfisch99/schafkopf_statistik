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
    assert_eq!(t.active_seats(4), vec![0, 1, 2, 3]);
    let g = t.add_game(sauspiel(4, 0, 1)).unwrap();
    assert_eq!(g.payouts, vec![20, 20, -20, -20, 0]);
    assert_eq!(t.next_dealer(), 0);
    // der Geber kann nicht mitspielen
    let err = t.add_game(sauspiel(0, 0, 1)).unwrap_err();
    assert_eq!(err, Error::SeatNotActive(0));
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
    assert_eq!(t(vec!["a", "b", "c"]).unwrap_err(), Error::PlayerCount(3));
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
                    partner: kind.is_team_game().then_some(2),
                    ..GameInput::new(3, kind, 1)
                };
                let p = payouts(&tariff, &input, 4, &active);
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
    assert_eq!(history.len(), 3);
    assert_eq!(history.last().unwrap(), &t.balances());
}
