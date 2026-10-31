//! Sankey-Diagramm der Geldflüsse: links wer zahlt, rechts wer bekommt, dazwischen Bänder.
//!
//! Jeder Knoten hat einen Platz von mindestens `SLOT_MIN` Höhe, damit Name und Betrag auch bei sehr
//! schmalen Bändern untereinander stehen. Der Balken selbst bleibt so hoch, wie der Betrag es verlangt.

use std::fmt::Write;

use crate::{format_euro, player_color, Cents};

const WIDTH: f64 = 360.0;
const BAR_W: f64 = 8.0;
const LABEL_W: f64 = 82.0;
const TOP: f64 = 24.0;
const BOTTOM: f64 = 8.0;
const GAP: f64 = 6.0;
const MAX_NAME: usize = 12;
/// Mindesthöhe des Platzes eines Knotens: zwei Textzeilen.
const SLOT_MIN: f64 = 26.0;
/// Ab dieser Bandhöhe steht der Betrag zusätzlich im Band.
const MIN_LABEL_H: f64 = 11.0;

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

/// Die Knoten einer Spalte von oben nach unten.
struct Column {
    /// Oberkante des Balkens und Mitte des Platzes, je Spieler (0 für Spieler ohne Knoten).
    bar_top: Vec<f64>,
    mid: Vec<f64>,
    /// Gesamthöhe der Spalte.
    extent: f64,
}

fn column(n: usize, members: &[usize], sums: &[Cents], scale: f64) -> Column {
    let mut col = Column {
        bar_top: vec![0.0; n],
        mid: vec![0.0; n],
        extent: 0.0,
    };
    let mut y = 0.0;
    for &i in members {
        let h = (sums[i] as f64 * scale).max(1.0);
        let slot = h.max(SLOT_MIN);
        col.bar_top[i] = y + (slot - h) / 2.0;
        col.mid[i] = y + slot / 2.0;
        y += slot + GAP;
    }
    col.extent = if members.is_empty() { 0.0 } else { y - GAP };
    col
}

/// Ein Band von `(x0, y0)` nach `(x1, y1)` mit Höhe `h`, dazu der Betrag in der Mitte, wenn es einen gibt und
/// das Band hoch genug ist.
#[allow(clippy::too_many_arguments)]
fn band(
    svg: &mut String,
    (x0, y0): (f64, f64),
    (x1, y1): (f64, f64),
    h: f64,
    color: &str,
    opacity: f64,
    title: &str,
    amount: Option<Cents>,
) {
    let xm = (x0 + x1) / 2.0;
    let _ = write!(
        svg,
        "<path fill=\"{color}\" fill-opacity=\"{opacity}\" d=\"M{x0:.1},{y0:.1} C{xm:.1},{y0:.1} {xm:.1},{y1:.1} {x1:.1},{y1:.1} \
         L{x1:.1},{b1:.1} C{xm:.1},{b1:.1} {xm:.1},{b0:.1} {x0:.1},{b0:.1} Z\"><title>{title}</title></path>",
        b0 = y0 + h,
        b1 = y1 + h,
    );
    if let (Some(amount), true) = (amount, h >= MIN_LABEL_H && opacity > 0.2) {
        let _ = write!(
            svg,
            "<text x=\"{xm:.1}\" y=\"{:.1}\" text-anchor=\"middle\" font-size=\"9\" fill=\"currentColor\">{}</text>",
            (y0 + y1 + h) / 2.0 + 3.0,
            format_euro(amount)
        );
    }
}

/// Balken, Name und Betrag (zwei Zeilen) eines Knotens.
#[allow(clippy::too_many_arguments)]
fn node(
    svg: &mut String,
    x: f64,
    right_side: bool,
    top: f64,
    mid: f64,
    h: f64,
    player: usize,
    name: &str,
    amount: Cents,
    dim: bool,
) {
    let _ = write!(
        svg,
        "<rect x=\"{x:.1}\" y=\"{top:.1}\" width=\"{BAR_W}\" height=\"{h:.1}\" fill=\"{}\" fill-opacity=\"{}\"/>",
        player_color(player),
        if dim { 0.35 } else { 1.0 }
    );
    let (tx, anchor) = if right_side {
        (x + BAR_W + 4.0, "start")
    } else {
        (x - 4.0, "end")
    };
    let _ = write!(
        svg,
        "<text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"currentColor\">{}</text>\
         <text x=\"{tx:.1}\" y=\"{:.1}\" text-anchor=\"{anchor}\" fill=\"currentColor\" opacity=\"0.75\">{}</text>",
        mid - 1.5,
        escape(&short(name)),
        mid + 9.5,
        format_euro(amount)
    );
}

fn header(height: f64, label: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {WIDTH} {height:.1}\" role=\"img\" \
         aria-label=\"{label}\" style=\"width:100%;height:auto\" \
         font-family=\"sans-serif\" font-size=\"10\">"
    )
}

fn empty(label: &str) -> String {
    format!(
        "{}<text x=\"{}\" y=\"60\" text-anchor=\"middle\" fill=\"currentColor\">Noch kein Geld bewegt</text></svg>",
        header(120.0, label),
        WIDTH / 2.0
    )
}

/// Maßstab (Pixel je Cent) so, dass die höhere Spalte etwa `36` Pixel je Zeile bekommt.
fn scale_for(total: Cents, rows: usize) -> f64 {
    let wanted = (36.0 * rows as f64).max(96.0) - GAP * rows.saturating_sub(1) as f64;
    wanted / total as f64
}

/// Sankey-Diagramm: `flows[von][an]` ist, was `von` an `an` gezahlt hat. Mit `highlight` bleiben nur die
/// Bänder dieses Spielers kräftig, die anderen werden blass.
pub fn sankey_svg(names: &[String], flows: &[Vec<Cents>], highlight: Option<usize>) -> String {
    let label = "Geldflüsse zwischen den Spielern";
    let n = names.len().min(flows.len());
    let out_sum: Vec<Cents> = (0..n).map(|i| flows[i].iter().take(n).sum()).collect();
    let in_sum: Vec<Cents> = (0..n)
        .map(|j| (0..n).map(|i| flows[i].get(j).copied().unwrap_or(0)).sum())
        .collect();
    let total: Cents = out_sum.iter().sum();
    if total <= 0 {
        return empty(label);
    }
    let left: Vec<usize> = (0..n).filter(|&i| out_sum[i] > 0).collect();
    let right: Vec<usize> = (0..n).filter(|&j| in_sum[j] > 0).collect();
    let scale = scale_for(total, left.len().max(right.len()));
    let (lc, rc) = (
        column(n, &left, &out_sum, scale),
        column(n, &right, &in_sum, scale),
    );
    let extent = lc.extent.max(rc.extent);
    let (l_shift, r_shift) = (
        TOP + (extent - lc.extent) / 2.0,
        TOP + (extent - rc.extent) / 2.0,
    );
    let height = (TOP + extent + BOTTOM).max(120.0);

    let x_left = LABEL_W;
    let x_right = WIDTH - LABEL_W - BAR_W;
    let mut svg = header(height, label);
    let _ = write!(
        svg,
        "<text x=\"{x_left}\" y=\"12\" text-anchor=\"end\" fill=\"currentColor\" opacity=\"0.7\">zahlt</text>\
         <text x=\"{}\" y=\"12\" fill=\"currentColor\" opacity=\"0.7\">bekommt</text>",
        x_right + BAR_W
    );

    // Bänder: je Zahler von oben nach unten nach Empfänger, am Empfänger nach Zahler.
    let mut out_at: Vec<f64> = (0..n).map(|i| lc.bar_top[i] + l_shift).collect();
    let mut in_at: Vec<f64> = (0..n).map(|j| rc.bar_top[j] + r_shift).collect();
    let (x0, x1) = (x_left + BAR_W, x_right);
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
            let title = format!(
                "{} zahlt an {}: {}",
                escape(&names[i]),
                escape(&names[j]),
                format_euro(value)
            );
            band(
                &mut svg,
                (x0, y0),
                (x1, y1),
                h,
                player_color(i),
                opacity,
                &title,
                None,
            );
        }
    }

    for (members, col, shift, sums, x, right_side) in [
        (&left, &lc, l_shift, &out_sum, x_left, false),
        (&right, &rc, r_shift, &in_sum, x_right, true),
    ] {
        for &i in members {
            let h = (sums[i] as f64 * scale).max(1.0);
            let dim = matches!(highlight, Some(p) if p != i);
            node(
                &mut svg,
                x,
                right_side,
                col.bar_top[i] + shift,
                col.mid[i] + shift,
                h,
                i,
                &names[i],
                sums[i],
                dim,
            );
        }
    }
    svg.push_str("</svg>");
    svg
}

/// Sankey-Diagramm eines Spielers: links von wem er Geld bekam, in der Mitte er selbst, rechts an wen er
/// zahlte. Name und Betrag stehen an jedem Knoten, bei hohen Bändern zusätzlich im Band.
pub fn player_sankey_svg(names: &[String], flows: &[Vec<Cents>], player: usize) -> String {
    let n = names.len().min(flows.len());
    let label = format!(
        "Geldflüsse von {}",
        escape(names.get(player).map_or("", String::as_str))
    );
    let got: Vec<Cents> = (0..n)
        .map(|i| {
            if i == player {
                0
            } else {
                flows[i].get(player).copied().unwrap_or(0)
            }
        })
        .collect();
    let paid: Vec<Cents> = (0..n)
        .map(|j| {
            if j == player {
                0
            } else {
                flows
                    .get(player)
                    .and_then(|r| r.get(j))
                    .copied()
                    .unwrap_or(0)
            }
        })
        .collect();
    let (got_total, paid_total): (Cents, Cents) = (got.iter().sum(), paid.iter().sum());
    let biggest = got_total.max(paid_total);
    if biggest <= 0 || player >= n {
        return empty(&label);
    }
    let left: Vec<usize> = (0..n).filter(|&i| got[i] > 0).collect();
    let right: Vec<usize> = (0..n).filter(|&j| paid[j] > 0).collect();
    let scale = scale_for(biggest, left.len().max(right.len()));
    let (lc, rc) = (
        column(n, &left, &got, scale),
        column(n, &right, &paid, scale),
    );
    let center_h = biggest as f64 * scale;
    let extent = lc.extent.max(rc.extent).max(center_h);
    let (l_shift, r_shift, c_top) = (
        TOP + (extent - lc.extent) / 2.0,
        TOP + (extent - rc.extent) / 2.0,
        TOP + (extent - center_h) / 2.0,
    );
    let height = (TOP + extent + BOTTOM + 14.0).max(120.0);

    let center_x = (WIDTH - BAR_W) / 2.0;
    let (x_left, x_right) = (LABEL_W, WIDTH - LABEL_W - BAR_W);
    let color = player_color(player);
    let mut svg = header(height, &label);
    let _ = write!(
        svg,
        "<text x=\"{x_left}\" y=\"12\" text-anchor=\"end\" fill=\"currentColor\" opacity=\"0.7\">bekam von</text>\
         <text x=\"{}\" y=\"12\" fill=\"currentColor\" opacity=\"0.7\">zahlte an</text>",
        x_right + BAR_W
    );

    // Bänder: links zur Mitte (Farbe des Zahlers), von der Mitte nach rechts (seine Farbe).
    let mut at_in = c_top;
    for &i in &left {
        let h = got[i] as f64 * scale;
        let title = format!(
            "{} zahlt an {}: {}",
            escape(&names[i]),
            escape(&names[player]),
            format_euro(got[i])
        );
        let from = (x_left + BAR_W, lc.bar_top[i] + l_shift);
        band(
            &mut svg,
            from,
            (center_x, at_in),
            h,
            player_color(i),
            0.5,
            &title,
            Some(got[i]),
        );
        at_in += h;
    }
    let mut at_out = c_top;
    for &j in &right {
        let h = paid[j] as f64 * scale;
        let title = format!(
            "{} zahlt an {}: {}",
            escape(&names[player]),
            escape(&names[j]),
            format_euro(paid[j])
        );
        let to = (x_right, rc.bar_top[j] + r_shift);
        band(
            &mut svg,
            (center_x + BAR_W, at_out),
            to,
            h,
            color,
            0.5,
            &title,
            Some(paid[j]),
        );
        at_out += h;
    }

    // Der Spieler in der Mitte: Balken, Name und was unterm Strich blieb.
    let _ = write!(
        svg,
        "<rect x=\"{center_x:.1}\" y=\"{c_top:.1}\" width=\"{BAR_W}\" height=\"{center_h:.1}\" fill=\"{color}\"/>\
         <text x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"middle\" fill=\"currentColor\">{}: {}</text>",
        center_x + BAR_W / 2.0,
        c_top + center_h + 12.0,
        escape(&short(&names[player])),
        format_euro(got_total - paid_total)
    );

    for (members, col, shift, sums, x, right_side) in [
        (&left, &lc, l_shift, &got, x_left, false),
        (&right, &rc, r_shift, &paid, x_right, true),
    ] {
        for &i in members {
            let h = (sums[i] as f64 * scale).max(1.0);
            node(
                &mut svg,
                x,
                right_side,
                col.bar_top[i] + shift,
                col.mid[i] + shift,
                h,
                i,
                &names[i],
                sums[i],
                false,
            );
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
        // Jeder Knoten zeigt seinen Betrag, auch der kleine (Cora bekommt 1,00 €).
        assert!(svg.contains(">1,00\u{a0}€</text>"));
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
    fn player_view_shows_inflows_and_outflows_with_amounts() {
        let flows = vec![vec![0, 500, 100], vec![200, 0, 0], vec![0, 0, 0]];
        let svg = player_sankey_svg(&names(), &flows, 0);
        // Anna bekam 2,00 von Ben und zahlte 5,00 an Ben und 1,00 an Cora: unterm Strich −4,00.
        assert_eq!(svg.matches("<path").count(), 3);
        assert!(svg.contains("zahlte an") && svg.contains("bekam von"));
        assert!(svg.contains("Anna: −4,00\u{a0}€"));
        assert!(svg.contains(">5,00\u{a0}€</text>") && svg.contains(">2,00\u{a0}€</text>"));
        // Auch das schmale Band (1,00 €) hat seinen Betrag, am Knoten.
        assert!(svg.contains(">1,00\u{a0}€</text>"));
        assert!(
            player_sankey_svg(&names(), &vec![vec![0; 3]; 3], 2).contains("Noch kein Geld bewegt")
        );
    }

    #[test]
    fn no_money_gives_a_note_instead_of_bands() {
        let svg = sankey_svg(&names(), &vec![vec![0; 3]; 3], None);
        assert!(svg.contains("Noch kein Geld bewegt") && !svg.contains("<path"));
        assert!(sankey_svg(&[], &[], None).contains("Noch kein Geld bewegt"));
    }
}
