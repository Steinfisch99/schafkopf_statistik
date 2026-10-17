//! Gesten auf dem Diagramm: Tippen wählt ein Spiel, Doppeltippen setzt zurück, Ziehen verschiebt,
//! zwei Finger zoomen. Reine Rechnung ohne Oberfläche, damit sie sich mit simulierten Fingern testen
//! lässt. Positionen sind Pixel innerhalb des Diagramms, `width` seine Breite in Pixeln.

use schafkopf_charts::{plot_fraction, plot_width_fraction, ChartView};

/// Bewegung in Pixeln, ab der aus einem Tippen ein Ziehen wird.
const TAP_SLOP_PX: f64 = 8.0;
const DOUBLE_TAP_MS: u64 = 350;
const DOUBLE_TAP_DISTANCE_PX: f64 = 30.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Pointer {
    id: i32,
    x: f64,
    y: f64,
}

#[derive(Clone, Copy, Debug)]
struct Pan {
    x0: f64,
    start0: f64,
    end0: f64,
}

#[derive(Clone, Copy, Debug)]
struct Pinch {
    start_distance: f64,
    start_len: f64,
    /// Das Spiel zwischen den Fingern, das beim Zoomen an seiner Stelle bleibt.
    anchor_game: f64,
}

/// Was ein losgelassener Finger bewirkt hat.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Nothing,
    /// Getippt: das Spiel an dieser Stelle (als Kommazahl) markieren.
    Select(f64),
    /// Doppelt getippt: den ganzen Verlauf zeigen.
    Reset,
}

#[derive(Clone, Debug, Default)]
pub struct Gestures {
    pointers: Vec<Pointer>,
    down_at: Option<(f64, f64)>,
    moved: bool,
    pan: Option<Pan>,
    pinch: Option<Pinch>,
    last_tap: Option<(f64, f64, u64)>,
}

fn distance(a: (f64, f64), b: (f64, f64)) -> f64 {
    ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt()
}

fn fraction(x: f64, width: f64) -> f64 {
    plot_fraction(x / width.max(1.0))
}

impl Gestures {
    fn pair(&self) -> Option<((f64, f64), (f64, f64))> {
        match self.pointers.as_slice() {
            [a, b, ..] => Some(((a.x, a.y), (b.x, b.y))),
            _ => None,
        }
    }

    fn start_pan(&mut self, x: f64, view: &ChartView) {
        self.pan = Some(Pan {
            x0: x,
            start0: view.start,
            end0: view.end,
        });
    }

    pub fn down(&mut self, id: i32, x: f64, y: f64, view: &ChartView, width: f64) {
        self.pointers.retain(|p| p.id != id);
        self.pointers.push(Pointer { id, x, y });
        if self.pointers.len() == 1 {
            self.down_at = Some((x, y));
            self.moved = false;
            self.pinch = None;
            self.start_pan(x, view);
        } else if let Some((a, b)) = self.pair() {
            self.moved = true;
            self.pan = None;
            self.pinch = Some(Pinch {
                start_distance: distance(a, b).max(1.0),
                start_len: view.len(),
                anchor_game: view.game_at(fraction((a.0 + b.0) / 2.0, width)),
            });
        }
    }

    pub fn moved_to(&mut self, id: i32, x: f64, y: f64, view: &mut ChartView, width: f64) {
        let Some(pointer) = self.pointers.iter_mut().find(|p| p.id == id) else {
            return;
        };
        pointer.x = x;
        pointer.y = y;

        if let (Some(pinch), Some((a, b))) = (self.pinch, self.pair()) {
            let new_len = pinch.start_len * pinch.start_distance / distance(a, b).max(1.0);
            let start = pinch.anchor_game - fraction((a.0 + b.0) / 2.0, width) * new_len;
            view.set_range(start, start + new_len);
        } else if let (Some(pan), 1) = (self.pan, self.pointers.len()) {
            if !self.moved {
                let from = self.down_at.unwrap_or((x, y));
                if distance(from, (x, y)) < TAP_SLOP_PX {
                    return;
                }
                self.moved = true;
            }
            let plot_px = (width * plot_width_fraction()).max(1.0);
            let delta = -(x - pan.x0) / plot_px * (pan.end0 - pan.start0);
            view.set_range(pan.start0 + delta, pan.end0 + delta);
        }
    }

    pub fn up(
        &mut self,
        id: i32,
        x: f64,
        y: f64,
        now_ms: u64,
        view: &mut ChartView,
        width: f64,
    ) -> Outcome {
        let Some(index) = self.pointers.iter().position(|p| p.id == id) else {
            return Outcome::Nothing;
        };
        let was_single = self.pointers.len() == 1;
        self.pointers.remove(index);

        if was_single && !self.moved && self.pinch.is_none() {
            self.pan = None;
            return match self.last_tap {
                Some((lx, ly, at))
                    if now_ms.saturating_sub(at) <= DOUBLE_TAP_MS
                        && distance((lx, ly), (x, y)) <= DOUBLE_TAP_DISTANCE_PX =>
                {
                    self.last_tap = None;
                    view.reset();
                    Outcome::Reset
                }
                _ => {
                    self.last_tap = Some((x, y, now_ms));
                    Outcome::Select(view.game_at(fraction(x, width)))
                }
            };
        }

        self.last_tap = None;
        self.pinch = None;
        match self.pointers.first().copied() {
            // Ein Finger bleibt liegen: ohne Sprung weiter ziehen.
            Some(rest) => {
                self.moved = true;
                self.start_pan(rest.x, view);
            }
            None => self.pan = None,
        }
        Outcome::Nothing
    }

    /// Das System hat die Berührung abgebrochen, zum Beispiel weil die Seite scrollt. Touch meldet
    /// nach dem Loslassen zusätzlich "Finger hat das Feld verlassen": Für einen Finger, der gar nicht
    /// mehr gedrückt ist, passiert deshalb nichts, sonst ginge das Doppeltippen verloren.
    pub fn cancel(&mut self, id: i32) {
        if !self.pointers.iter().any(|p| p.id == id) {
            return;
        }
        self.pointers.retain(|p| p.id != id);
        self.pinch = None;
        self.pan = None;
        self.last_tap = None;
        self.moved = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f64 = 360.0;

    /// Pixel-Position im Diagramm, die dem Anteil `f` im Zeichenbereich entspricht.
    fn x_at(f: f64) -> f64 {
        46.0 + f * 304.0
    }

    /// Prüft, dass getippt wurde und welches Spiel gemeint ist, mit Toleranz für Rundung.
    fn assert_selected(outcome: Outcome, game: f64) {
        match outcome {
            Outcome::Select(g) => assert!((g - game).abs() < 1e-9, "{g} statt {game}"),
            other => panic!("kein Tippen: {other:?}"),
        }
    }

    fn zoomed(games: usize, start: f64, end: f64) -> ChartView {
        let mut v = ChartView::full(games);
        v.set_range(start, end);
        v
    }

    #[test]
    fn a_tap_selects_the_game_under_the_finger() {
        let mut view = ChartView::full(10);
        let mut g = Gestures::default();
        g.down(1, x_at(0.5), 100.0, &view, W);
        let out = g.up(1, x_at(0.5), 100.0, 1000, &mut view, W);
        assert_selected(out, 5.0);
        assert!(view.is_full());
    }

    #[test]
    fn tapping_a_zoomed_view_uses_its_own_scale() {
        let mut view = zoomed(30, 10.0, 20.0);
        let mut g = Gestures::default();
        g.down(1, x_at(1.0), 50.0, &view, W);
        assert_selected(g.up(1, x_at(1.0), 50.0, 0, &mut view, W), 20.0);
    }

    #[test]
    fn small_jitter_is_still_a_tap() {
        let mut view = ChartView::full(10);
        let mut g = Gestures::default();
        g.down(1, 200.0, 100.0, &view, W);
        g.moved_to(1, 204.0, 103.0, &mut view, W);
        assert!(matches!(
            g.up(1, 204.0, 103.0, 10, &mut view, W),
            Outcome::Select(_)
        ));
        assert!(view.is_full());
    }

    #[test]
    fn two_quick_taps_reset_the_view() {
        let mut view = zoomed(30, 10.0, 20.0);
        let mut g = Gestures::default();
        g.down(1, 200.0, 100.0, &view, W);
        assert!(matches!(
            g.up(1, 200.0, 100.0, 1000, &mut view, W),
            Outcome::Select(_)
        ));
        g.down(2, 205.0, 102.0, &view, W);
        assert_eq!(g.up(2, 205.0, 102.0, 1200, &mut view, W), Outcome::Reset);
        assert!(view.is_full());
    }

    #[test]
    fn slow_or_distant_second_taps_are_just_taps() {
        let mut view = zoomed(30, 10.0, 20.0);
        let mut g = Gestures::default();
        g.down(1, 200.0, 100.0, &view, W);
        g.up(1, 200.0, 100.0, 1000, &mut view, W);
        g.down(2, 200.0, 100.0, &view, W);
        assert!(matches!(
            g.up(2, 200.0, 100.0, 1500, &mut view, W),
            Outcome::Select(_)
        ));
        g.down(3, 100.0, 100.0, &view, W);
        assert!(matches!(
            g.up(3, 100.0, 100.0, 1600, &mut view, W),
            Outcome::Select(_)
        ));
        assert!(!view.is_full());
    }

    #[test]
    fn dragging_pans_the_view_and_is_not_a_tap() {
        let mut view = zoomed(30, 10.0, 20.0);
        let mut g = Gestures::default();
        g.down(1, 250.0, 100.0, &view, W);
        g.moved_to(1, 200.0, 100.0, &mut view, W);
        g.moved_to(1, 150.0, 100.0, &mut view, W);
        // 100 Pixel nach links = 100/304 des Ausschnitts (10 Spiele) nach rechts.
        assert!((view.start - (10.0 + 100.0 / 304.0 * 10.0)).abs() < 1e-9);
        assert!((view.len() - 10.0).abs() < 1e-9);
        assert_eq!(g.up(1, 150.0, 100.0, 50, &mut view, W), Outcome::Nothing);
    }

    #[test]
    fn panning_stops_at_the_ends_of_the_data() {
        let mut view = zoomed(30, 2.0, 12.0);
        let mut g = Gestures::default();
        g.down(1, 100.0, 0.0, &view, W);
        g.moved_to(1, 340.0, 0.0, &mut view, W);
        assert_eq!((view.start, view.end), (0.0, 10.0));
        g.moved_to(1, -2000.0, 0.0, &mut view, W);
        assert_eq!((view.start, view.end), (20.0, 30.0));
    }

    #[test]
    fn spreading_two_fingers_zooms_in_around_their_center() {
        let mut view = ChartView::full(30);
        let mut g = Gestures::default();
        let center = x_at(0.5);
        g.down(1, center - 50.0, 100.0, &view, W);
        g.down(2, center + 50.0, 100.0, &view, W);
        g.moved_to(1, center - 100.0, 100.0, &mut view, W);
        g.moved_to(2, center + 100.0, 100.0, &mut view, W);
        // Abstand doppelt so groß: halber Ausschnitt, die Mitte (Spiel 15) bleibt in der Mitte.
        assert!((view.len() - 15.0).abs() < 1e-9);
        assert!((view.game_at(0.5) - 15.0).abs() < 1e-9);
    }

    #[test]
    fn pinching_two_fingers_together_zooms_out() {
        let mut view = zoomed(30, 10.0, 16.0);
        let mut g = Gestures::default();
        g.down(1, 150.0, 100.0, &view, W);
        g.down(2, 250.0, 100.0, &view, W);
        g.moved_to(2, 200.0, 100.0, &mut view, W);
        assert!((view.len() - 12.0).abs() < 1e-9);
        // Nie weiter als der ganze Verlauf.
        g.moved_to(2, 150.0, 100.0, &mut view, W);
        g.moved_to(1, 150.0, 100.0, &mut view, W);
        assert!(view.len() <= 30.0);
    }

    #[test]
    fn zoom_never_goes_below_three_games() {
        let mut view = ChartView::full(30);
        let mut g = Gestures::default();
        g.down(1, 100.0, 0.0, &view, W);
        g.down(2, 110.0, 0.0, &view, W);
        g.moved_to(2, 5000.0, 0.0, &mut view, W);
        assert!((view.len() - 3.0).abs() < 1e-9);
    }

    #[test]
    fn lifting_one_finger_after_a_pinch_continues_with_a_pan_without_jumping() {
        let mut view = ChartView::full(30);
        let mut g = Gestures::default();
        g.down(1, 100.0, 0.0, &view, W);
        g.down(2, 200.0, 0.0, &view, W);
        g.moved_to(2, 300.0, 0.0, &mut view, W);
        let before = (view.start, view.end);
        assert_eq!(g.up(1, 100.0, 0.0, 10, &mut view, W), Outcome::Nothing);
        assert_eq!((view.start, view.end), before);
        g.moved_to(2, 300.0, 0.0, &mut view, W);
        assert_eq!((view.start, view.end), before, "kein Sprung beim Loslassen");
        g.moved_to(2, 250.0, 0.0, &mut view, W);
        assert!(view.start > before.0, "weiteres Ziehen verschiebt");
        assert_eq!(g.up(2, 250.0, 0.0, 20, &mut view, W), Outcome::Nothing);
    }

    #[test]
    fn the_leave_event_after_lifting_a_finger_keeps_the_double_tap() {
        let mut view = zoomed(30, 10.0, 20.0);
        let mut g = Gestures::default();
        g.down(1, 200.0, 100.0, &view, W);
        assert!(matches!(
            g.up(1, 200.0, 100.0, 1000, &mut view, W),
            Outcome::Select(_)
        ));
        g.cancel(1); // Touch meldet nach dem Loslassen noch "verlassen"
        g.down(2, 203.0, 101.0, &view, W);
        assert_eq!(g.up(2, 203.0, 101.0, 1200, &mut view, W), Outcome::Reset);
        g.cancel(2);
        assert!(view.is_full());
    }

    #[test]
    fn a_cancelled_touch_changes_nothing_and_next_touch_works() {
        let mut view = zoomed(30, 10.0, 20.0);
        let mut g = Gestures::default();
        g.down(1, 200.0, 0.0, &view, W);
        g.cancel(1);
        assert_eq!(g.up(1, 200.0, 0.0, 5, &mut view, W), Outcome::Nothing);
        g.down(2, 200.0, 0.0, &view, W);
        assert!(matches!(
            g.up(2, 200.0, 0.0, 9999, &mut view, W),
            Outcome::Select(_)
        ));
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        let mut view = ChartView::full(0);
        let mut g = Gestures::default();
        g.down(1, 10.0, 10.0, &view, 0.0);
        g.moved_to(1, 400.0, 10.0, &mut view, 0.0);
        g.down(2, 10.0, 10.0, &view, 0.0);
        g.moved_to(2, 0.0, 0.0, &mut view, 0.0);
        g.up(2, 0.0, 0.0, 1, &mut view, 0.0);
        g.up(1, 0.0, 0.0, 2, &mut view, 0.0);
        g.moved_to(7, 1.0, 1.0, &mut view, 360.0);
        assert!(view.start.is_finite() && view.end.is_finite());
        let mut one = ChartView::full(1);
        g.down(3, 100.0, 0.0, &one, 360.0);
        g.moved_to(3, 300.0, 0.0, &mut one, 360.0);
        assert_eq!((one.start, one.end), (0.0, 1.0));
    }
}
