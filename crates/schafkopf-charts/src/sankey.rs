//! Sankey-Diagramm der Geldflüsse: links wer zahlt, rechts wer bekommt, dazwischen Bänder.

use std::fmt::Write;

use crate::{format_euro, player_color, Cents};

const WIDTH: f64 = 360.0;
const BAR_W: f64 = 8.0;
const LABEL_W: f64 = 82.0;
const TOP: f64 = 24.0;
const BOTTOM: f64 = 8.0;
const GAP: f64 = 8.0;
const MAX_NAME: usize = 12;

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn short(name: &str) -> String {
    if name.chars().count() > MAX_NAME {
        let cut: String = name.chars().take(MAX_NAME - 1).collect();
        format!("{cut}…")
    } else {
        name.to_owned()
    }
}

/// Sankey-Diagramm: `flows[von][an]` ist, was `von` an `an` gezahlt hat. Mit `highlight` bleiben nur die
/// Bänder dieses Spielers kräftig, die anderen werden blass.
pub fn sankey_svg(names: &[String], flows: &[Vec<Cents>], highlight: Option<usize>) -> String {
    let n = names.len().min(flows.len());
    let out_sum: Vec<Cents> = (0..n).map(|i| flows[i].iter().take(n).sum()).collect();
    let in_sum: Vec<Cents> = (0..n)
        .map(|j| (0..n).map(|i| flows[i].get(j).copied().unwrap_or(0)).sum())
        .collect();
    let total: Cents = out_sum.iter().sum();
    let left: Vec<usize> = (0..n).filter(|&i| out_sum[i] > 0).collect();
    let right: Vec<usize> = (0..n).filter(|&j| in_sum[j] > 0).collect();
    let rows = left.len().max(right.len());
    let height = (TOP + BOTTOM + 36.0 * rows as f64).max(120.0);

    let mut svg = String::new();
    let _ = write!(
        svg,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {WIDTH} {height}\" role=\"img\" \
         aria-label=\"Geldflüsse zwischen den Spielern\" style=\"width:100%;height:auto\" \
         font-family=\"sans-serif\" font-size=\"10\">"
    );
    if total <= 0 {
        let _ = write!(
            svg,
            "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"currentColor\">Noch kein Geld bewegt</text></svg>",
            WIDTH / 2.0,
            height / 2.0
        );
        return svg;
    }

    let x_left = LABEL_W;
    let x_right = WIDTH - LABEL_W - BAR_W;
    let avail = height - TOP - BOTTOM - GAP * (rows.saturating_sub(1)) as f64;
    let scale = avail / total as f64;

    // Oberkante jedes Knotens: nacheinander von oben, mit Lücke dazwischen.
    let stack = |members: &[usize], sums: &[Cents]| -> Vec<f64> {
        let mut tops = vec![0.0; n];
        let mut y = TOP;
        for &i in members {
            tops[i] = y;
            y += sums[i] as f64 * scale + GAP;
        }
        tops
    };
    let left_top = stack(&left, &out_sum);
    let right_top = stack(&right, &in_sum);

    let _ = write!(
        svg,
        "<text x=\"{x_left}\" y=\"12\" text-anchor=\"end\" fill=\"currentColor\" opacity=\"0.7\">zahlt</text>\
         <text x=\"{}\" y=\"12\" fill=\"currentColor\" opacity=\"0.7\">bekommt</text>",
        x_right + BAR_W
    );

    // Bänder: je Zahler von oben nach unten nach Empfänger, am Empfänger nach Zahler.
    let mut out_at = left_top.clone();
    let mut in_at = right_top.clone();
    let (x0, x1) = (x_left + BAR_W, x_right);
    let xm = (x0 + x1) / 2.0;
    for &i in &left {
        for &j in &right {
            let value = flows[i].get(j).copied().unwrap_or(0);
            if value <= 0 || i == j {
                continue;
            }
            let h = value as f64 * scale;
            let (y0, y1) = (out_at[i], in_at[j]);
            out_at[i] += h;
            in_at[j] += h;
            let opacity = match highlight {
                Some(p) if p == i || p == j => 0.75,
                Some(_) => 0.08,
                None => 0.5,
            };
            let _ = write!(
                svg,
                "<path fill=\"{}\" fill-opacity=\"{opacity}\" d=\"M{x0:.1},{y0:.1} C{xm:.1},{y0:.1} {xm:.1},{y1:.1} {x1:.1},{y1:.1} \
                 L{x1:.1},{b1:.1} C{xm:.1},{b1:.1} {xm:.1},{b0:.1} {x0:.1},{b0:.1} Z\">\
                 <title>{} zahlt an {}: {}</title></path>",
                player_color(i),
                escape(&names[i]),
                escape(&names[j]),
                format_euro(value),
                b0 = y0 + h,
                b1 = y1 + h,
            );
        }
    }

    // Die Knoten mit Namen und Betrag.
    for (members, tops, sums, x, right_side) in [
        (&left, &left_top, &out_sum, x_left, false),
        (&right, &right_top, &in_sum, x_right, true),
    ] {
        for &i in members {
            let h = (sums[i] as f64 * scale).max(1.0);
            let dim = matches!(highlight, Some(p) if p != i);
            let _ = write!(
                svg,
                "<rect x=\"{x:.1}\" y=\"{:.1}\" width=\"{BAR_W}\" height=\"{h:.1}\" fill=\"{}\" fill-opacity=\"{}\"/>",
                tops[i],
                player_color(i),
                if dim { 0.35 } else { 1.0 }
            );
            let (tx, anchor) = if right_side {
                (x + BAR_W + 4.0, "start")
            } else {
                (x - 4.0, "end")
            };
            let mid = tops[i] + h / 2.0;
            let name = escape(&short(&names[i]));
            let amount = format_euro(sums[i]);
            if h >= 22.0 {
                let _ = write!(
                    svg,
                    "<text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"currentColor\">{name}</text>\
                     <text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"currentColor\" opacity=\"0.7\">{amount}</text>",
                    mid - 1.0,
                    mid + 10.0
                );
            } else {
                let _ = write!(
                    svg,
                    "<text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"currentColor\">{name}</text>",
                    mid + 3.5
                );
            }
        }
    }
    svg.push_str("</svg>");
    svg
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        ["Anna", "Ben & Co", "Cora"].map(String::from).to_vec()
    }

    #[test]
    fn draws_a_band_per_flow_and_escapes_names() {
        let flows = vec![vec![0, 500, 100], vec![200, 0, 0], vec![0, 0, 0]];
        let svg = sankey_svg(&names(), &flows, None);
        assert_eq!(svg.matches("<path").count(), 3);
        assert!(svg.contains("Ben &amp; Co"));
        assert!(svg.contains("Anna zahlt an Ben &amp; Co: 5,00\u{a0}€"));
        assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
    }

    #[test]
    fn highlight_dims_the_bands_of_the_others() {
        let flows = vec![vec![0, 500, 100], vec![200, 0, 0], vec![0, 0, 0]];
        let svg = sankey_svg(&names(), &flows, Some(2));
        assert_eq!(svg.matches("fill-opacity=\"0.75\"").count(), 1);
        assert_eq!(svg.matches("fill-opacity=\"0.08\"").count(), 2);
    }

    #[test]
    fn no_money_gives_a_note_instead_of_bands() {
        let svg = sankey_svg(&names(), &vec![vec![0; 3]; 3], None);
        assert!(svg.contains("Noch kein Geld bewegt") && !svg.contains("<path"));
        assert!(sankey_svg(&[], &[], None).contains("Noch kein Geld bewegt"));
    }
}
