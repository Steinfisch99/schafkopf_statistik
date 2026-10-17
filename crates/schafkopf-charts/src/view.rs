//! Welcher Ausschnitt des Verlaufs sichtbar ist und welches Spiel markiert ist.

/// Sichtbarer Bereich des Geldstand-Diagramms, in Spielen gezählt.
///
/// Spiel 0 ist der Start bei 0, Spiel `games` der aktuelle Stand. Der Bereich ist immer
/// mindestens [`ChartView::min_len`] Spiele breit und liegt innerhalb von `0..=games`.
#[derive(Clone, Debug, PartialEq)]
pub struct ChartView {
    /// Anzahl Spiele, für die diese Ansicht gilt. Ändert sie sich, gilt wieder der ganze Verlauf.
    pub games: usize,
    pub start: f64,
    pub end: f64,
    /// Das angetippte Spiel (0 bis `games`).
    pub selected: Option<usize>,
}

impl ChartView {
    /// Der ganze Verlauf, nichts markiert.
    pub fn full(games: usize) -> Self {
        ChartView {
            games,
            start: 0.0,
            end: games as f64,
            selected: None,
        }
    }

    /// Kleinster Ausschnitt in Spielen: drei, oder alle, wenn es weniger gibt.
    pub fn min_len(&self) -> f64 {
        (self.games as f64).min(3.0)
    }

    pub fn len(&self) -> f64 {
        self.end - self.start
    }

    /// Ob der ganze Verlauf zu sehen ist.
    pub fn is_full(&self) -> bool {
        self.start <= 0.0 && self.end >= self.games as f64
    }

    /// Setzt den Ausschnitt und schiebt oder staucht ihn so, dass er gültig bleibt.
    pub fn set_range(&mut self, start: f64, end: f64) {
        let games = self.games as f64;
        let len = (end - start).clamp(self.min_len(), games);
        let start = if len >= games {
            0.0
        } else {
            start.clamp(0.0, games - len)
        };
        self.start = start;
        self.end = start + len;
    }

    /// Zeigt die letzten `n` Spiele.
    pub fn show_last(&mut self, n: usize) {
        let games = self.games as f64;
        self.set_range(games - n as f64, games);
    }

    pub fn reset(&mut self) {
        self.start = 0.0;
        self.end = self.games as f64;
    }

    /// Das Spiel an einer Stelle des Zeichenbereichs (0 links, 1 rechts), als Kommazahl.
    pub fn game_at(&self, fraction: f64) -> f64 {
        self.start + fraction.clamp(0.0, 1.0) * self.len()
    }

    /// Das nächste ganze Spiel an einer Stelle des Zeichenbereichs.
    pub fn nearest_game(&self, fraction: f64) -> usize {
        (self.game_at(fraction).round().max(0.0) as usize).min(self.games)
    }

    /// Diese Ansicht, falls sie zur aktuellen Spielzahl passt, sonst der ganze Verlauf.
    pub fn adapted(&self, games: usize) -> ChartView {
        if self.games == games {
            ChartView {
                selected: self.selected.filter(|&s| s <= games),
                ..self.clone()
            }
        } else {
            ChartView::full(games)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_view_covers_everything() {
        let v = ChartView::full(10);
        assert!(v.is_full());
        assert_eq!((v.start, v.end, v.len()), (0.0, 10.0, 10.0));
        assert_eq!(v.min_len(), 3.0);
        assert_eq!(ChartView::full(2).min_len(), 2.0);
        assert_eq!(ChartView::full(0).min_len(), 0.0);
    }

    #[test]
    fn ranges_are_clamped_to_the_data() {
        let mut v = ChartView::full(10);
        v.set_range(-5.0, 4.0);
        assert_eq!(
            (v.start, v.end),
            (0.0, 9.0),
            "Länge bleibt, Start wird geschoben"
        );
        v.set_range(8.0, 20.0);
        assert_eq!(
            (v.start, v.end),
            (0.0, 10.0),
            "länger als alles: ganzer Verlauf"
        );
        v.set_range(5.0, 5.5);
        assert_eq!(v.len(), 3.0, "nicht kleiner als drei Spiele");
        v.set_range(9.5, 12.5);
        assert_eq!((v.start, v.end), (7.0, 10.0));
        assert!(!v.is_full());
    }

    #[test]
    fn last_n_games_and_reset() {
        let mut v = ChartView::full(30);
        v.show_last(10);
        assert_eq!((v.start, v.end), (20.0, 30.0));
        v.show_last(100);
        assert!(v.is_full());
        v.show_last(1);
        assert_eq!(v.len(), 3.0);
        v.reset();
        assert!(v.is_full());
    }

    #[test]
    fn maps_positions_to_games() {
        let mut v = ChartView::full(10);
        v.set_range(4.0, 8.0);
        assert_eq!(v.game_at(0.0), 4.0);
        assert_eq!(v.game_at(0.5), 6.0);
        assert_eq!(v.game_at(2.0), 8.0, "außerhalb wird begrenzt");
        assert_eq!(v.nearest_game(0.6), 6);
        assert_eq!(v.nearest_game(0.0), 4);
        assert_eq!(ChartView::full(10).nearest_game(1.0), 10);
        assert_eq!(ChartView::full(0).nearest_game(0.7), 0);
    }

    #[test]
    fn adapts_to_a_changed_number_of_games() {
        let mut v = ChartView::full(10);
        v.set_range(2.0, 6.0);
        v.selected = Some(4);
        assert_eq!(v.adapted(10), v);
        let after_new_game = v.adapted(11);
        assert!(after_new_game.is_full());
        assert_eq!((after_new_game.games, after_new_game.selected), (11, None));
    }
}
