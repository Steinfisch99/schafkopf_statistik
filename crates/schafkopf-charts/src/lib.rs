//! Diagramme für die Schafkopf-App als SVG-Text.
//!
//! Die Funktionen bekommen reine Zahlen und liefern ein `<svg>`-Element, das die UI einbettet.
//! Texte und Gitterlinien nutzen `currentColor`, damit sie im hellen und dunklen Modus lesbar sind.

use std::fmt::Write;

/// Geldbeträge in Cent, wie im Kern.
pub type Cents = i64;

/// Farbenblind-taugliche Palette (Okabe/Ito) plus zwei Ergänzungen für bis zu 10 Spieler.
const PALETTE: [&str; 10] = [
    "#0072B2", "#E69F00", "#009E73", "#D55E00", "#CC79A7", "#56B4E9", "#B8A600", "#7A7A7A",
    "#882255", "#44AA99",
];

/// Farbe des Spielers mit diesem Sitzplatz (für Diagramm und Legende).
pub fn player_color(seat: usize) -> &'static str {
    PALETTE[seat % PALETTE.len()]
}

/// Betrag ohne Währung, deutsch formatiert: `-120` wird zu `−1,20`.
pub fn format_amount(cents: Cents) -> String {
    let sign = if cents < 0 { "\u{2212}" } else { "" };
    let abs = cents.unsigned_abs();
    format!("{sign}{},{:02}", abs / 100, abs % 100)
}

/// Betrag in Euro mit geschütztem Leerzeichen: `-120` wird zu `−1,20 €`.
pub fn format_euro(cents: Cents) -> String {
    format!("{}\u{a0}€", format_amount(cents))
}

/// Ein "runder" Abstand (1, 2, 5 mal Zehnerpotenz), der etwa `target` Linien ergibt.
fn nice_step(range: Cents, target: Cents) -> Cents {
    let raw = (range / target).max(1);
    let mut magnitude = 1;
    while magnitude * 10 <= raw {
        magnitude *= 10;
    }
    [1, 2, 5, 10]
        .into_iter()
        .map(|m| m * magnitude)
        .find(|&s| s >= raw)
        .unwrap_or(magnitude * 10)
}

mod view;

pub use view::ChartView;

/// Breite und Höhe der Zeichnung in SVG-Einheiten. Das Diagramm skaliert auf die Breite der Seite.
const WIDTH: f64 = 360.0;
const HEIGHT: f64 = 240.0;
/// Ränder um den Zeichenbereich, links Platz für die Beträge, unten für die Spielnummern.
const LEFT: f64 = 46.0;
const RIGHT: f64 = 10.0;
const TOP: f64 = 10.0;
const BOTTOM: f64 = 24.0;

/// Wo im Zeichenbereich (0 links, 1 rechts) liegt eine Stelle, die als Anteil an der Breite des
/// ganzen Diagramms angegeben ist? Damit rechnet die App Fingerpositionen in Spiele um.
pub fn plot_fraction(fraction_of_width: f64) -> f64 {
    ((fraction_of_width * WIDTH - LEFT) / (WIDTH - LEFT - RIGHT)).clamp(0.0, 1.0)
}

/// Anteil des Zeichenbereichs an der Breite des ganzen Diagramms.
pub fn plot_width_fraction() -> f64 {
    (WIDTH - LEFT - RIGHT) / WIDTH
}

/// Liniendiagramm des Geldstands aller Spieler, ganzer Verlauf.
///
/// `history[g][s]` ist der Stand von Sitzplatz `s` nach `g` Spielen. Der erste Eintrag ist der Start
/// bei 0 (siehe `schafkopf_core::balance_history`). Siehe [`balance_chart_svg_view`] für einen
/// Ausschnitt mit Markierung.
pub fn balance_chart_svg(history: &[Vec<Cents>]) -> String {
    let games = history.len().saturating_sub(1);
    balance_chart_svg_view(history, &ChartView::full(games))
}

/// Liniendiagramm des Geldstands mit sichtbarem Ausschnitt und markiertem Spiel.
///
/// Die Hochachse passt sich an die sichtbaren Spiele an, mindestens ein Euro hoch. Im ganzen
/// Verlauf ist die Nulllinie immer dabei, weil der Start bei 0 liegt. Ist ein Spiel markiert, zeigt
/// eine senkrechte Linie und je ein Punkt pro Spieler dessen Stand, sonst der Punkt am Ende.
pub fn balance_chart_svg_view(history: &[Vec<Cents>], view: &ChartView) -> String {
    let players = history.first().map_or(0, Vec::len);
    let games = history.len().saturating_sub(1);
    let view = view.adapted(games);
    let (start, end) = (view.start, view.end);

    let first = start.floor() as usize;
    let last = (end.ceil() as usize).min(games).max(first);
    let visible = history.get(first..=last).unwrap_or(&[]);
    let values = visible.iter().flatten().copied();
    let (mut lo, mut hi) = (values.clone().min().unwrap_or(0), values.max().unwrap_or(0));
    // Mindestens ein Euro Spanne, damit ein leerer oder kleiner Verlauf nicht winzig skaliert wird.
    if hi - lo < 100 {
        let mid = (lo + hi).div_euclid(2);
        (lo, hi) = (mid - 50, mid + 50);
    }
    let step = nice_step(hi - lo, 4);
    let axis_lo = lo.div_euclid(step) * step;
    let mut axis_hi = (hi + step - 1).div_euclid(step) * step;
    if axis_hi == axis_lo {
        axis_hi += step;
    }

    let plot_w = WIDTH - LEFT - RIGHT;
    let plot_h = HEIGHT - TOP - BOTTOM;
    let span = end - start;
    let x = |g: f64| {
        if span <= 0.0 {
            LEFT
        } else {
            LEFT + plot_w * (g - start) / span
        }
    };
    let y = |v: Cents| TOP + plot_h * (axis_hi - v) as f64 / (axis_hi - axis_lo) as f64;

    let mut svg = String::new();
    let _ = write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {WIDTH} {HEIGHT}\" role=\"img\" \
         aria-label=\"Geldstand der Spieler\" style=\"width:100%;height:auto\" \
         font-family=\"sans-serif\" font-size=\"10\">\
         <clipPath id=\"plot\"><rect x=\"{LEFT}\" y=\"{TOP}\" width=\"{plot_w}\" height=\"{plot_h}\"/></clipPath>"
    );

    // Waagerechte Linien mit Beträgen, die Nulllinie kräftiger.
    let mut tick = axis_lo;
    while tick <= axis_hi {
        let ty = y(tick);
        let (opacity, width) = if tick == 0 { (0.7, 1.4) } else { (0.2, 1.0) };
        let _ = write!(
            svg,
            "<line x1=\"{LEFT}\" x2=\"{x2:.1}\" y1=\"{ty:.1}\" y2=\"{ty:.1}\" stroke=\"currentColor\" \
             stroke-opacity=\"{opacity}\" stroke-width=\"{width}\"/>\
             <text x=\"{tx}\" y=\"{ty2:.1}\" text-anchor=\"end\" fill=\"currentColor\">{label}</text>",
            x2 = WIDTH - RIGHT,
            tx = LEFT - 5.0,
            ty2 = ty + 3.5,
            label = format_amount(tick),
        );
        tick += step;
    }

    // Spielnummern unten, in runden Abständen.
    if games > 0 {
        let x_step = nice_step(span.ceil() as Cents, 5).max(1);
        let mut game = (start / x_step as f64).ceil() as Cents * x_step;
        while game as f64 <= end + 1e-9 {
            let _ = write!(
                svg,
                "<text x=\"{:.1}\" y=\"{}\" text-anchor=\"middle\" fill=\"currentColor\">{game}</text>",
                x(game as f64),
                HEIGHT - 6.0,
            );
            game += x_step;
        }
    }

    let focus = view.selected.unwrap_or(games);
    if let Some(selected) = view.selected {
        if (start..=end).contains(&(selected as f64)) {
            let _ = write!(
                svg,
                "<line x1=\"{sx:.1}\" x2=\"{sx:.1}\" y1=\"{TOP}\" y2=\"{bottom}\" stroke=\"currentColor\" \
                 stroke-opacity=\"0.55\" stroke-width=\"1.2\" stroke-dasharray=\"3 3\"/>",
                sx = x(selected as f64),
                bottom = TOP + plot_h,
            );
        }
    }

    // Die Linien, am Rand des Zeichenbereichs abgeschnitten.
    svg.push_str("<g clip-path=\"url(#plot)\">");
    #[allow(clippy::needless_range_loop)]
    // `seat` ist der Spieler, nicht nur ein Index in `history`
    for seat in 0..players {
        let points: Vec<String> = (first..=last)
            .map(|g| format!("{:.1},{:.1}", x(g as f64), y(history[g][seat])))
            .collect();
        if points.len() > 1 {
            let _ = write!(
                svg,
                "<polyline fill=\"none\" stroke=\"{}\" stroke-width=\"2\" \
                 stroke-linejoin=\"round\" stroke-linecap=\"round\" points=\"{}\"/>",
                player_color(seat),
                points.join(" ")
            );
        }
    }
    svg.push_str("</g>");

    // Ein Punkt je Spieler am markierten Spiel, sonst am letzten.
    if !history.is_empty() && (start..=end).contains(&(focus as f64)) {
        for (seat, &balance) in history[focus].iter().enumerate() {
            let _ = write!(
                svg,
                "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"{}\" fill=\"{}\"/>",
                x(focus as f64),
                y(balance),
                if view.selected.is_some() { 4.5 } else { 3.5 },
                player_color(seat)
            );
        }
    }

    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amounts_are_formatted_german() {
        assert_eq!(format_amount(0), "0,00");
        assert_eq!(format_amount(5), "0,05");
        assert_eq!(format_amount(-120), "\u{2212}1,20");
        assert_eq!(format_euro(1999), "19,99\u{a0}€");
    }

    #[test]
    fn nice_steps_are_round() {
        assert_eq!(nice_step(100, 4), 50);
        assert_eq!(nice_step(40, 4), 10);
        assert_eq!(nice_step(1, 4), 1);
        assert_eq!(nice_step(1000, 4), 500);
        assert_eq!(nice_step(270, 4), 100);
    }

    #[test]
    fn one_line_per_player_and_zero_line_is_drawn() {
        let history = vec![vec![0, 0, 0], vec![-40, 40, 0], vec![-20, 60, -40]];
        let svg = balance_chart_svg(&history);
        assert_eq!(svg.matches("<polyline").count(), 3);
        assert_eq!(svg.matches("<circle").count(), 3);
        assert!(svg.contains(">0,00<"));
        assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
    }

    #[test]
    fn works_without_games_and_with_only_negative_values() {
        let empty = balance_chart_svg(&[]);
        assert!(empty.contains("<svg") && !empty.contains("NaN"));
        let start = balance_chart_svg(&[vec![0, 0]]);
        assert!(!start.contains("NaN") && start.matches("<circle").count() == 2);
        assert_eq!(start.matches("<polyline").count(), 0);
        let negative = balance_chart_svg(&[vec![0, 0], vec![-100, -300]]);
        assert!(!negative.contains("NaN") && !negative.contains("inf"));
        assert!(negative.contains(">0,00<"));
    }

    #[test]
    fn higher_balance_is_drawn_higher() {
        let svg = balance_chart_svg(&[vec![0, 0], vec![100, -100]]);
        let cy = |c: &str| -> f64 {
            let i = svg.find(&format!("fill=\"{c}\"/>")).unwrap();
            let head = &svg[..i];
            let k = head.rfind("cy=\"").unwrap() + 4;
            head[k..].split('"').next().unwrap().parse().unwrap()
        };
        assert!(cy(player_color(0)) < cy(player_color(1)));
    }

    fn sample(games: usize) -> Vec<Vec<Cents>> {
        (0..=games as i64)
            .map(|g| vec![g * 10, -g * 10 + (g % 3) * 5, (g % 4) * 7])
            .collect()
    }

    #[test]
    fn zoomed_view_draws_only_the_visible_points() {
        let history = sample(30);
        let mut view = ChartView::full(30);
        view.set_range(10.0, 20.0);
        let svg = balance_chart_svg_view(&history, &view);
        let points = |svg: &str| {
            let i = svg.find("points=\"").unwrap() + 8;
            svg[i..].split('"').next().unwrap().split(' ').count()
        };
        assert_eq!(points(&svg), 11);
        assert_eq!(points(&balance_chart_svg(&history)), 31);
        assert!(svg.contains("clip-path=\"url(#plot)\""));
        assert!(!svg.contains("NaN") && !svg.contains("inf"));
    }

    #[test]
    fn zoomed_axis_fits_the_visible_values() {
        let history = sample(30);
        let full = balance_chart_svg(&history);
        let mut view = ChartView::full(30);
        view.set_range(0.0, 3.0);
        let zoomed = balance_chart_svg_view(&history, &view);
        // Ganzer Verlauf: Achse bis ±4,00. Ausschnitt der ersten drei Spiele: nur gut einen Euro Spanne.
        assert!(full.contains(">4,00<") && full.contains(">\u{2212}4,00<"));
        assert!(!zoomed.contains(">4,00<") && zoomed.contains(">0,50<"));
    }

    #[test]
    fn game_numbers_are_round_and_inside_the_view() {
        let history = sample(30);
        let mut view = ChartView::full(30);
        view.set_range(12.5, 22.5);
        let svg = balance_chart_svg_view(&history, &view);
        assert!(svg.contains(">14<") && svg.contains(">16<") && svg.contains(">22<"));
        assert!(!svg.contains(">12<") && !svg.contains(">24<"));
    }

    #[test]
    fn selected_game_gets_a_line_and_dots_for_every_player() {
        let history = sample(10);
        let mut view = ChartView::full(10);
        assert!(!balance_chart_svg_view(&history, &view).contains("stroke-dasharray"));
        view.selected = Some(4);
        let svg = balance_chart_svg_view(&history, &view);
        assert!(svg.contains("stroke-dasharray"));
        assert_eq!(svg.matches("<circle").count(), 3);
        assert!(svg.contains("r=\"4.5\""));
        // Liegt die Markierung außerhalb des Ausschnitts, bleibt sie weg.
        view.set_range(6.0, 10.0);
        let svg = balance_chart_svg_view(&history, &view);
        assert!(!svg.contains("stroke-dasharray") && !svg.contains("<circle"));
    }

    #[test]
    fn plot_fraction_maps_finger_positions() {
        assert_eq!(
            plot_fraction(0.0),
            0.0,
            "links vom Zeichenbereich wird begrenzt"
        );
        assert_eq!(plot_fraction(1.0), 1.0);
        let left_edge = LEFT / WIDTH;
        assert!((plot_fraction(left_edge) - 0.0).abs() < 1e-9);
        let middle = (LEFT + (WIDTH - LEFT - RIGHT) / 2.0) / WIDTH;
        assert!((plot_fraction(middle) - 0.5).abs() < 1e-9);
        assert!((plot_width_fraction() - 304.0 / 360.0).abs() < 1e-9);
    }

    #[test]
    fn stale_views_fall_back_to_the_full_chart() {
        let history = sample(10);
        let mut old = ChartView::full(5);
        old.set_range(1.0, 4.0);
        assert_eq!(
            balance_chart_svg_view(&history, &old),
            balance_chart_svg(&history)
        );
    }
}
