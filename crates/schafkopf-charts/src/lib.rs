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

/// Liniendiagramm des Geldstands aller Spieler.
///
/// `history[g][s]` ist der Stand von Sitzplatz `s` nach `g` Spielen. Der erste Eintrag ist der Start
/// bei 0 (siehe `schafkopf_core::balance_history`). Die Nulllinie ist immer sichtbar, auch bei
/// nur negativen Werten. Mit weniger als zwei Einträgen entsteht ein leeres Diagramm mit Achsen.
pub fn balance_chart_svg(history: &[Vec<Cents>]) -> String {
    const W: f64 = 360.0;
    const H: f64 = 240.0;
    const LEFT: f64 = 46.0;
    const RIGHT: f64 = 10.0;
    const TOP: f64 = 10.0;
    const BOTTOM: f64 = 24.0;

    let players = history.first().map_or(0, Vec::len);
    let values = history.iter().flatten().copied();
    let lo = values.clone().min().unwrap_or(0).min(0);
    let hi = values.max().unwrap_or(0).max(0);
    // Mindestens ein Euro Spanne, damit ein leerer oder kleiner Verlauf nicht winzig skaliert wird.
    let step = nice_step((hi - lo).max(100), 4);
    let axis_lo = lo.div_euclid(step) * step;
    let axis_hi = (hi + step - 1).div_euclid(step) * step;
    let axis_hi = if axis_hi == axis_lo {
        axis_lo + step
    } else {
        axis_hi
    };

    let games = history.len().saturating_sub(1);
    let x = |g: usize| {
        if games == 0 {
            LEFT
        } else {
            LEFT + (W - LEFT - RIGHT) * g as f64 / games as f64
        }
    };
    let y = |v: Cents| TOP + (H - TOP - BOTTOM) * (axis_hi - v) as f64 / (axis_hi - axis_lo) as f64;

    let mut svg = String::new();
    let _ = write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W} {H}\" role=\"img\" \
         aria-label=\"Geldstand der Spieler\" style=\"width:100%;height:auto\" \
         font-family=\"sans-serif\" font-size=\"10\">\
         <title>Geldstand der Spieler</title>"
    );

    let mut tick = axis_lo;
    while tick <= axis_hi {
        let ty = y(tick);
        let (opacity, width) = if tick == 0 { (0.7, 1.4) } else { (0.2, 1.0) };
        let _ = write!(
            svg,
            "<line x1=\"{LEFT}\" x2=\"{x2:.1}\" y1=\"{ty:.1}\" y2=\"{ty:.1}\" stroke=\"currentColor\" \
             stroke-opacity=\"{opacity}\" stroke-width=\"{width}\"/>\
             <text x=\"{tx}\" y=\"{ty2:.1}\" text-anchor=\"end\" fill=\"currentColor\">{label}</text>",
            x2 = W - RIGHT,
            tx = LEFT - 5.0,
            ty2 = ty + 3.5,
            label = format_amount(tick),
        );
        tick += step;
    }

    let _ = write!(
        svg,
        "<text x=\"{LEFT}\" y=\"{by}\" fill=\"currentColor\">0</text>\
         <text x=\"{rx}\" y=\"{by}\" text-anchor=\"end\" fill=\"currentColor\">Spiel {games}</text>",
        by = H - 6.0,
        rx = W - RIGHT,
    );

    for seat in 0..players {
        let color = player_color(seat);
        let points: Vec<String> = history
            .iter()
            .enumerate()
            .map(|(g, row)| format!("{:.1},{:.1}", x(g), y(row[seat])))
            .collect();
        if points.len() > 1 {
            let _ = write!(
                svg,
                "<polyline fill=\"none\" stroke=\"{color}\" stroke-width=\"2\" \
                 stroke-linejoin=\"round\" stroke-linecap=\"round\" points=\"{}\"/>",
                points.join(" ")
            );
        }
        let last = history.len() - 1;
        let _ = write!(
            svg,
            "<circle cx=\"{:.1}\" cy=\"{:.1}\" r=\"3.5\" fill=\"{color}\"/>",
            x(last),
            y(history[last][seat])
        );
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
}
