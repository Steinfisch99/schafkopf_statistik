//! Ringdiagramm (Torte mit Loch) als SVG-Text.

use std::f64::consts::PI;
use std::fmt::Write;

const SIZE: f64 = 120.0;
const RADIUS: f64 = 42.0;
const THICKNESS: f64 = 20.0;

/// Macht Text für ein SVG-Attribut oder -Element ungefährlich.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Ringdiagramm. Die Stücke sind `(Wert, Farbe)` und beginnen oben, im Uhrzeigersinn. Stücke ohne
/// positiven Wert werden übersprungen. In der Mitte steht `center` groß und darunter `sub` klein.
/// `label` ist die Beschreibung für Screenreader. Ohne Summe zeigt der Ring nur seinen Hintergrund.
pub fn donut_svg(slices: &[(f64, &str)], center: &str, sub: &str, label: &str) -> String {
    let positive = |v: f64| if v.is_finite() && v > 0.0 { v } else { 0.0 };
    let total: f64 = slices.iter().map(|(v, _)| positive(*v)).sum();
    let circumference = 2.0 * PI * RADIUS;
    let mid = SIZE / 2.0;

    let mut svg = String::new();
    let _ = write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {SIZE} {SIZE}\" role=\"img\" \
         aria-label=\"{}\" style=\"width:100%;height:auto\" font-family=\"sans-serif\">\
         <circle cx=\"{mid}\" cy=\"{mid}\" r=\"{RADIUS}\" fill=\"none\" stroke=\"currentColor\" \
         stroke-opacity=\"0.12\" stroke-width=\"{THICKNESS}\"/>",
        escape(label)
    );

    if total > 0.0 {
        let mut offset = 0.0;
        for (value, color) in slices {
            let value = positive(*value);
            if value == 0.0 {
                continue;
            }
            let length = circumference * value / total;
            let _ = write!(
                svg,
                "<circle cx=\"{mid}\" cy=\"{mid}\" r=\"{RADIUS}\" fill=\"none\" stroke=\"{}\" \
                 stroke-width=\"{THICKNESS}\" stroke-dasharray=\"{length:.2} {rest:.2}\" \
                 stroke-dashoffset=\"{shift:.2}\" transform=\"rotate(-90 {mid} {mid})\"/>",
                escape(color),
                rest = (circumference - length).max(0.0),
                shift = -offset,
            );
            offset += length;
        }
    }

    let (center_y, sub_y) = if sub.is_empty() {
        (mid + 7.0, 0.0)
    } else {
        (mid + 1.0, mid + 16.0)
    };
    let _ = write!(
        svg,
        "<text x=\"{mid}\" y=\"{center_y}\" text-anchor=\"middle\" font-size=\"22\" font-weight=\"700\" \
         fill=\"currentColor\">{}</text>",
        escape(center)
    );
    if !sub.is_empty() {
        let _ = write!(
            svg,
            "<text x=\"{mid}\" y=\"{sub_y}\" text-anchor=\"middle\" font-size=\"9\" \
             fill=\"currentColor\" fill-opacity=\"0.75\">{}</text>",
            escape(sub)
        );
    }
    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    const CIRCUMFERENCE: f64 = 2.0 * PI * RADIUS;

    /// Alle `stroke-dasharray`-Längen der Stücke.
    fn lengths(svg: &str) -> Vec<f64> {
        svg.split("stroke-dasharray=\"")
            .skip(1)
            .map(|s| s.split(' ').next().unwrap().parse().unwrap())
            .collect()
    }

    #[test]
    fn slices_share_the_ring_in_proportion() {
        let svg = donut_svg(
            &[(3.0, "#009E73"), (1.0, "#D55E00")],
            "75 %",
            "4 Spiele",
            "Siege",
        );
        let l = lengths(&svg);
        assert_eq!(l.len(), 2);
        assert!((l[0] - CIRCUMFERENCE * 0.75).abs() < 0.01);
        assert!((l[1] - CIRCUMFERENCE * 0.25).abs() < 0.01);
        assert!((l[0] + l[1] - CIRCUMFERENCE).abs() < 0.02);
        assert!(svg.contains(">75 %<") && svg.contains(">4 Spiele<"));
        assert!(svg.contains("aria-label=\"Siege\""));
    }

    #[test]
    fn second_slice_starts_where_the_first_ends() {
        let svg = donut_svg(&[(1.0, "red"), (1.0, "blue")], "", "", "x");
        let offsets: Vec<f64> = svg
            .split("stroke-dashoffset=\"")
            .skip(1)
            .map(|s| s.split('"').next().unwrap().parse().unwrap())
            .collect();
        assert_eq!(offsets[0], 0.0);
        assert!((offsets[1] + CIRCUMFERENCE / 2.0).abs() < 0.01);
    }

    #[test]
    fn without_a_total_only_the_background_ring_is_drawn() {
        for slices in [
            vec![],
            vec![(0.0, "red")],
            vec![(f64::NAN, "red"), (-3.0, "blue")],
        ] {
            let svg = donut_svg(&slices, "–", "", "leer");
            assert_eq!(svg.matches("<circle").count(), 1, "{slices:?}");
            assert!(svg.contains(">–<") && !svg.contains("NaN") && !svg.contains("inf"));
        }
    }

    #[test]
    fn one_slice_fills_the_whole_ring_and_empty_slices_are_skipped() {
        let svg = donut_svg(&[(5.0, "red"), (0.0, "blue")], "100 %", "", "alles");
        assert_eq!(lengths(&svg).len(), 1);
        assert!((lengths(&svg)[0] - CIRCUMFERENCE).abs() < 0.01);
        assert_eq!(
            svg.matches("<text").count(),
            1,
            "ohne Unterzeile nur ein Text"
        );
    }

    #[test]
    fn text_is_escaped() {
        let svg = donut_svg(&[(1.0, "red")], "<b>&", "\"x\"", "A & B");
        assert!(svg.contains("&lt;b&gt;&amp;") && svg.contains("&quot;x&quot;"));
        assert!(svg.contains("aria-label=\"A &amp; B\""));
        assert!(!svg.contains("<b>"));
    }
}
