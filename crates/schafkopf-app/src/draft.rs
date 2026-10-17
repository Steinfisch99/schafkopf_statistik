//! Entwürfe für die Eingabemasken: reine Logik ohne Oberfläche, damit sie sich testen lässt.

use schafkopf_charts::format_amount;
use schafkopf_core::{parse_euro, Game, GameInput, GameKind, Seat, Suit, Table, Tariff};

/// Ein Spiel, das gerade eingetragen wird.
#[derive(Clone, Debug, PartialEq)]
pub struct GameDraft {
    pub dealer: Seat,
    pub participants: Vec<Seat>,
    pub kind: GameKind,
    pub suit: Option<Suit>,
    pub declarer: Option<Seat>,
    pub partner: Option<Seat>,
    pub knockers: Vec<Seat>,
    pub kontra_by: Option<Seat>,
    pub re: bool,
    pub tout: bool,
    pub won: bool,
    pub schneider: bool,
    pub schwarz: bool,
    pub laufende: u8,
    pub price_text: String,
    /// Beim Bearbeiten: Angaben des ursprünglichen Spiels, die das Formular nicht zeigt, aber nicht
    /// verlieren darf.
    pub played_at: Option<i64>,
    pub points: Option<u8>,
    pub note: Option<String>,
}

impl GameDraft {
    /// Neues Spiel: nächster Geber, die üblichen Mitspielenden, Sauspiel als Vorauswahl.
    pub fn new(table: &Table) -> Self {
        let dealer = table.next_dealer();
        GameDraft {
            dealer,
            participants: table.default_participants(dealer),
            kind: if table.is_enabled(GameKind::Sauspiel) {
                GameKind::Sauspiel
            } else {
                table
                    .enabled_kinds
                    .first()
                    .copied()
                    .unwrap_or(GameKind::Sauspiel)
            },
            suit: None,
            declarer: None,
            partner: None,
            knockers: Vec::new(),
            kontra_by: None,
            re: false,
            tout: false,
            won: true,
            schneider: false,
            schwarz: false,
            laufende: 0,
            price_text: String::new(),
            played_at: None,
            points: None,
            note: None,
        }
    }

    /// Entwurf aus einem bestehenden Spiel, zum Bearbeiten.
    pub fn from_game(table: &Table, index: usize) -> Option<Self> {
        let g = table.games().get(index)?;
        let i = &g.input;
        Some(GameDraft {
            dealer: i.dealer,
            participants: i.participants.clone(),
            kind: i.kind,
            suit: i.suit,
            declarer: Some(i.declarer),
            partner: i.partner,
            knockers: i.knockers.clone(),
            kontra_by: i.kontra_by,
            re: i.re,
            tout: i.tout,
            won: i.won,
            schneider: i.schneider || i.schwarz,
            schwarz: i.schwarz,
            laufende: i.laufende,
            price_text: i.price_override.map(format_amount).unwrap_or_default(),
            played_at: i.played_at,
            points: i.points,
            note: i.note.clone(),
        })
    }

    /// Wirft alles raus, was nicht mehr zu den Mitspielenden gehört.
    fn prune(&mut self) {
        let keep = |s: &Seat| self.participants.contains(s);
        self.declarer = self.declarer.filter(keep);
        self.partner = self.partner.filter(keep);
        self.kontra_by = self.kontra_by.filter(keep);
        let participants = self.participants.clone();
        self.knockers.retain(|s| participants.contains(s));
        if self.kontra_by.is_none() {
            self.re = false;
        }
    }

    pub fn set_dealer(&mut self, table: &Table, dealer: Seat) {
        self.dealer = dealer;
        self.participants = table.default_participants(dealer);
        self.prune();
    }

    /// Nimmt einen Spieler ins Spiel oder aus dem Spiel, wenn mehr als vier am Tisch sitzen.
    /// Es bleiben höchstens vier Mitspielende, und die Reihenfolge ist die Spielreihenfolge.
    pub fn set_participant(&mut self, table: &Table, seat: Seat, on: bool) {
        let n = table.player_count();
        if on && !self.participants.contains(&seat) && self.participants.len() < 4 {
            self.participants.push(seat);
        } else if !on {
            self.participants.retain(|&s| s != seat);
        }
        let dealer = self.dealer;
        self.participants.sort_by_key(|&s| (s + n - dealer - 1) % n);
        self.prune();
    }

    pub fn set_kind(&mut self, kind: GameKind) {
        self.kind = kind;
        self.suit = None;
        if !kind.is_team_game() {
            self.partner = None;
        }
        if !kind.allows_tout() {
            self.tout = false;
        }
        if !kind.has_extras() {
            self.schneider = false;
            self.schwarz = false;
            self.laufende = 0;
        }
        self.won = kind != GameKind::Ramsch;
    }

    pub fn is_on_declarer_team(&self, seat: Seat) -> bool {
        self.declarer == Some(seat) || self.partner == Some(seat)
    }

    fn drop_kontra_if_own_team(&mut self) {
        if self.kontra_by.is_some_and(|k| self.is_on_declarer_team(k)) {
            self.kontra_by = None;
            self.re = false;
        }
    }

    /// Ramsch: umschalten zwischen "einer verliert" und Durchmarsch. Die Rolle der gewählten Person
    /// wechselt dabei, also muss neu gewählt werden.
    pub fn toggle_durchmarsch(&mut self) {
        if self.kind == GameKind::Ramsch {
            self.won = !self.won;
            self.declarer = None;
        }
    }

    pub fn toggle_declarer(&mut self, seat: Seat) {
        if self.declarer == Some(seat) {
            self.declarer = None;
        } else {
            self.declarer = Some(seat);
            if self.partner == Some(seat) {
                self.partner = None;
            }
        }
        self.drop_kontra_if_own_team();
    }

    pub fn toggle_partner(&mut self, seat: Seat) {
        if !self.kind.is_team_game() {
            return;
        }
        if self.partner == Some(seat) {
            self.partner = None;
        } else {
            self.partner = Some(seat);
            if self.declarer == Some(seat) {
                self.declarer = None;
            }
        }
        self.drop_kontra_if_own_team();
    }

    pub fn toggle_knock(&mut self, seat: Seat) {
        if self.knockers.contains(&seat) {
            self.knockers.retain(|&s| s != seat);
        } else {
            self.knockers.push(seat);
        }
    }

    /// Kontra kommt von einem Gegenspieler; wer zur Spielerpartei gehört, kann es nicht geben.
    pub fn toggle_kontra(&mut self, seat: Seat) {
        if self.kontra_by == Some(seat) {
            self.kontra_by = None;
            self.re = false;
        } else if !self.is_on_declarer_team(seat) {
            self.kontra_by = Some(seat);
        }
    }

    pub fn toggle_re(&mut self) {
        if self.kontra_by.is_some() {
            self.re = !self.re;
        }
    }

    pub fn set_schneider(&mut self, on: bool) {
        self.schneider = on;
        if !on {
            self.schwarz = false;
        }
    }

    /// Schwarz zählt immer auch als Schneider.
    pub fn set_schwarz(&mut self, on: bool) {
        self.schwarz = on;
        if on {
            self.schneider = true;
        }
    }

    pub fn set_laufende(&mut self, n: u8) {
        self.laufende = n.min(8);
    }

    /// Was noch fehlt, damit das Spiel eingetragen werden kann.
    pub fn problem(&self) -> Option<String> {
        let kind = self.kind;
        if self.declarer.is_none() {
            return Some(if kind == GameKind::Ramsch {
                if self.won {
                    "Durchmarschierer wählen"
                } else {
                    "Verlierer wählen"
                }
                .into()
            } else {
                "Spieler wählen".into()
            });
        }
        if kind.is_team_game() && self.partner.is_none() {
            return Some("Mitspieler wählen".into());
        }
        if kind.is_team_game() && self.participants.len() != 4 {
            return Some(format!("{} braucht genau vier Mitspielende", kind.label()));
        }
        if kind == GameKind::Sauspiel && self.suit.is_none() {
            return Some("Gerufene Sau wählen".into());
        }
        if kind.needs_trump_suit() && self.suit.is_none() {
            return Some("Trumpffarbe wählen".into());
        }
        if self.participants.len() < 2 {
            return Some("Mindestens zwei Mitspielende nötig".into());
        }
        if !self.price_text.trim().is_empty() && parse_euro(&self.price_text).is_none() {
            return Some("Verlangter Preis nicht lesbar, zum Beispiel 0,40".into());
        }
        None
    }

    pub fn to_input(&self, played_at: Option<i64>) -> Result<GameInput, String> {
        if let Some(problem) = self.problem() {
            return Err(problem);
        }
        let mut input = GameInput::new(
            self.dealer,
            self.kind,
            self.declarer.ok_or("Spieler wählen")?,
        );
        input.participants = self.participants.clone();
        input.suit = self.suit;
        input.partner = self.partner;
        input.knockers = self.knockers.clone();
        input.kontra_by = self.kontra_by;
        input.re = self.re;
        input.tout = self.tout;
        input.won = self.won;
        input.schneider = self.schneider || self.schwarz;
        input.schwarz = self.schwarz;
        input.laufende = self.laufende;
        input.price_override = parse_euro(&self.price_text);
        input.played_at = self.played_at.or(played_at);
        input.points = self.points;
        input.note = self.note.clone();
        Ok(input)
    }

    /// Preis und Auszahlung, wie sie beim Eintragen herauskämen.
    pub fn preview(&self, table: &Table) -> Result<Game, String> {
        let input = self.to_input(None)?;
        table.preview_game(input).map_err(|e| e.to_string())
    }

    /// Preis und Auszahlung beim Bearbeiten des Spiels mit diesem Index: Es gilt der Tarif, mit dem
    /// das Spiel ursprünglich abgerechnet wurde.
    pub fn preview_edit(&self, table: &Table, index: usize) -> Result<Game, String> {
        let input = self.to_input(None)?;
        table.preview_edit(index, input).map_err(|e| e.to_string())
    }
}

/// Der Tarif als Text in den Eingabefeldern.
#[derive(Clone, Debug, PartialEq)]
pub struct TariffDraft {
    pub sauspiel: String,
    pub solo: String,
    pub bettel: String,
    pub ramsch: String,
    pub schneider: String,
    pub schwarz: String,
    pub laufende: String,
    pub min_sauspiel: String,
    pub min_wenz_geier: String,
    pub min_solo: String,
}

impl TariffDraft {
    pub fn from_tariff(t: &Tariff) -> Self {
        TariffDraft {
            sauspiel: format_amount(t.sauspiel),
            solo: format_amount(t.solo),
            bettel: format_amount(t.bettel),
            ramsch: format_amount(t.ramsch),
            schneider: format_amount(t.schneider),
            schwarz: format_amount(t.schwarz),
            laufende: format_amount(t.laufende),
            min_sauspiel: t.laufende_min_sauspiel.to_string(),
            min_wenz_geier: t.laufende_min_wenz_geier.to_string(),
            min_solo: t.laufende_min_solo.to_string(),
        }
    }

    pub fn to_tariff(&self) -> Result<Tariff, String> {
        let money = |label: &str, text: &str| {
            parse_euro(text)
                .ok_or_else(|| format!("{label}: Betrag nicht lesbar, zum Beispiel 0,20"))
        };
        let count = |label: &str, text: &str| match text.trim().parse::<u8>() {
            Ok(n) if (1..=8).contains(&n) => Ok(n),
            _ => Err(format!("{label}: eine Zahl von 1 bis 8 eingeben")),
        };
        Ok(Tariff {
            sauspiel: money("Sauspiel", &self.sauspiel)?,
            solo: money("Solo", &self.solo)?,
            bettel: money("Bettel", &self.bettel)?,
            ramsch: money("Ramsch", &self.ramsch)?,
            schneider: money("Schneider", &self.schneider)?,
            schwarz: money("Schwarz", &self.schwarz)?,
            laufende: money("Laufende", &self.laufende)?,
            laufende_min_sauspiel: count("Laufende ab (Sauspiel)", &self.min_sauspiel)?,
            laufende_min_wenz_geier: count("Laufende ab (Wenz und Geier)", &self.min_wenz_geier)?,
            laufende_min_solo: count("Laufende ab (Solo)", &self.min_solo)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(n: usize) -> Table {
        let players = (0..n).map(|i| format!("S{i}")).collect();
        Table::new("T", players, 0, Tariff::default()).unwrap()
    }

    #[test]
    fn sauspiel_flow_from_draft_to_payouts() {
        let t = table(4);
        let mut d = GameDraft::new(&t);
        assert_eq!(d.dealer, 0);
        assert_eq!(d.participants, vec![1, 2, 3, 0]);
        assert_eq!(d.problem().as_deref(), Some("Spieler wählen"));
        d.toggle_declarer(1);
        assert_eq!(d.problem().as_deref(), Some("Mitspieler wählen"));
        d.toggle_partner(3);
        assert_eq!(d.problem().as_deref(), Some("Gerufene Sau wählen"));
        d.suit = Some(Suit::Eichel);
        assert_eq!(d.problem(), None);
        let preview = d.preview(&t).unwrap();
        assert_eq!(preview.value, 20);
        assert_eq!(preview.payouts, vec![-20, 20, -20, 20]);
    }

    #[test]
    fn extras_knock_and_demanded_price() {
        let t = table(4);
        let mut d = GameDraft::new(&t);
        d.toggle_declarer(2);
        d.toggle_partner(0);
        d.suit = Some(Suit::Gras);
        d.toggle_knock(1);
        d.set_schneider(true);
        d.set_laufende(3);
        d.won = false;
        // (20 + 10 + 30) * 2
        assert_eq!(d.preview(&t).unwrap().value, 120);
        d.price_text = "1,50".into();
        let g = d.preview(&t).unwrap();
        assert_eq!((g.value, g.calculated_value), (150, 120));
        d.price_text = "abc".into();
        assert!(d.problem().unwrap().contains("Preis"));
        d.price_text = "  ".into();
        assert_eq!(d.problem(), None);
    }

    #[test]
    fn roles_exclude_each_other() {
        let t = table(4);
        let mut d = GameDraft::new(&t);
        d.toggle_declarer(1);
        d.toggle_partner(1);
        assert_eq!((d.declarer, d.partner), (None, Some(1)));
        d.toggle_declarer(1);
        assert_eq!((d.declarer, d.partner), (Some(1), None));
        d.toggle_partner(2);
        d.toggle_kontra(2);
        assert_eq!(d.kontra_by, None, "Mitspieler gibt kein Kontra");
        d.toggle_kontra(3);
        d.toggle_re();
        assert!(d.re);
        d.toggle_kontra(3);
        assert!(!d.re, "Re fällt mit Kontra weg");
        d.toggle_kontra(3);
        d.toggle_partner(3);
        assert_eq!(d.kontra_by, None, "wer Mitspieler wird, gibt kein Kontra");
        d.toggle_re();
        assert!(!d.re);
    }

    #[test]
    fn changing_kind_resets_dependent_fields() {
        let t = table(4);
        let mut d = GameDraft::new(&t);
        d.toggle_declarer(1);
        d.toggle_partner(2);
        d.suit = Some(Suit::Gras);
        d.set_kind(GameKind::Wenz);
        assert_eq!((d.partner, d.suit), (None, None));
        d.tout = true;
        d.set_laufende(5);
        d.set_kind(GameKind::Bettel);
        assert!(!d.tout);
        assert_eq!(d.laufende, 0);
        d.set_kind(GameKind::Ramsch);
        assert!(!d.won);
        assert_eq!(d.problem(), None, "Ramsch braucht nur den Verlierer");
        d.set_kind(GameKind::Farbsolo);
        assert!(d.won);
        assert_eq!(d.problem().as_deref(), Some("Trumpffarbe wählen"));
    }

    #[test]
    fn ramsch_switches_between_loser_and_durchmarsch() {
        let t = table(4);
        let mut d = GameDraft::new(&t);
        d.set_kind(GameKind::Ramsch);
        assert!(!d.won);
        assert_eq!(d.problem().as_deref(), Some("Verlierer wählen"));
        d.toggle_declarer(2);
        assert_eq!(d.problem(), None);
        let loss = d.preview(&t).unwrap();
        assert_eq!(loss.payouts[2], -60, "der Verlierer zahlt dreimal 0,20");
        d.toggle_durchmarsch();
        assert!(d.won);
        assert_eq!(d.declarer, None, "die Rolle wechselt, neu wählen");
        assert_eq!(d.problem().as_deref(), Some("Durchmarschierer wählen"));
        d.toggle_declarer(2);
        assert_eq!(
            d.preview(&t).unwrap().payouts[2],
            60,
            "der Durchmarschierer bekommt dreimal 0,20"
        );
        d.toggle_durchmarsch();
        assert!(!d.won);
        // Bei anderen Spielarten tut der Umschalter nichts.
        d.set_kind(GameKind::Wenz);
        d.toggle_durchmarsch();
        assert!(d.won);
    }

    #[test]
    fn editing_roundtrips_a_game_and_keeps_hidden_fields() {
        let mut t = table(4);
        let input = GameInput {
            suit: Some(Suit::Gras),
            partner: Some(3),
            knockers: vec![2],
            kontra_by: Some(0),
            re: true,
            schneider: true,
            laufende: 3,
            price_override: Some(150),
            points: Some(95),
            played_at: Some(1234),
            note: Some("Notiz".into()),
            ..GameInput::new(0, GameKind::Sauspiel, 1)
        };
        t.add_game(input.clone()).unwrap();
        let draft = GameDraft::from_game(&t, 0).unwrap();
        assert_eq!(draft.price_text, "1,50");
        let back = draft.to_input(Some(9999)).unwrap();
        // Zeitpunkt, Augen und Notiz bleiben erhalten, die Mitspielenden sind jetzt festgelegt.
        assert_eq!(back.played_at, Some(1234));
        assert_eq!(
            (back.points, back.note.as_deref()),
            (Some(95), Some("Notiz"))
        );
        assert_eq!(
            GameInput {
                participants: vec![],
                ..back.clone()
            },
            GameInput {
                participants: vec![],
                ..input
            }
        );
        assert!(GameDraft::from_game(&t, 1).is_none());

        let mut edited = draft;
        edited.price_text = String::new();
        edited.won = false;
        let preview = edited.preview_edit(&t, 0).unwrap();
        // (20 + 10 Schneider + 3 * 10 Laufende) * 2^3 für Klopfen, Kontra und Re; Preis von Hand entfernt.
        assert_eq!((preview.value, preview.calculated_value), (480, 480));
    }

    #[test]
    fn new_game_starts_with_the_first_enabled_kind() {
        let mut t = table(4);
        assert_eq!(GameDraft::new(&t).kind, GameKind::Sauspiel);
        t.use_rules(None, &[GameKind::Geier, GameKind::Wenz])
            .unwrap();
        assert_eq!(
            GameDraft::new(&t).kind,
            GameKind::Wenz,
            "Wenz steht vor Geier"
        );
        t.use_rules(None, &[GameKind::Sauspiel, GameKind::Geier])
            .unwrap();
        assert_eq!(GameDraft::new(&t).kind, GameKind::Sauspiel);
    }

    #[test]
    fn schwarz_implies_schneider() {
        let t = table(4);
        let mut d = GameDraft::new(&t);
        d.set_schwarz(true);
        assert!(d.schneider);
        d.set_schneider(false);
        assert!(!d.schwarz);
        d.toggle_declarer(1);
        d.toggle_partner(2);
        d.suit = Some(Suit::Gras);
        d.set_schwarz(true);
        let input = d.to_input(Some(5)).unwrap();
        assert!(input.schneider && input.schwarz);
        assert_eq!(input.played_at, Some(5));
    }

    #[test]
    fn large_tables_participants_and_dealer_change() {
        let t = table(6);
        let mut d = GameDraft::new(&t);
        assert_eq!(d.participants, vec![1, 2, 5, 0]);
        d.toggle_declarer(5);
        d.set_participant(&t, 5, false);
        assert_eq!(
            d.declarer, None,
            "ausgetauschter Spieler verliert seine Rolle"
        );
        d.set_participant(&t, 3, true);
        assert_eq!(d.participants, vec![1, 2, 3, 0]);
        d.set_participant(&t, 4, true);
        assert_eq!(d.participants.len(), 4, "mehr als vier gehen nicht");
        d.set_dealer(&t, 2);
        assert_eq!(d.participants, vec![3, 4, 1, 2]);
    }

    #[test]
    fn team_game_needs_exactly_four() {
        let t = table(3);
        let mut d = GameDraft::new(&t);
        d.toggle_declarer(1);
        d.toggle_partner(2);
        d.suit = Some(Suit::Gras);
        assert!(d.problem().unwrap().contains("vier"));
        d.set_kind(GameKind::Wenz);
        assert_eq!(d.problem(), None);
        assert!(d.preview(&t).is_ok());
    }

    #[test]
    fn tariff_draft_roundtrip_and_errors() {
        let t = Tariff::default();
        let draft = TariffDraft::from_tariff(&t);
        assert_eq!(draft.sauspiel, "0,20");
        assert_eq!(draft.to_tariff().unwrap(), t);
        let mut bad = draft.clone();
        bad.solo = "viel".into();
        assert!(bad.to_tariff().unwrap_err().starts_with("Solo"));
        let mut bad = draft;
        bad.min_wenz_geier = "0".into();
        assert!(bad.to_tariff().unwrap_err().contains("Wenz"));
    }
}
