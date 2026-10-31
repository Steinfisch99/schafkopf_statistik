//! Geldflüsse zwischen den Spielern: wer wem wie viel gezahlt hat, für das Sankey-Diagramm.
//!
//! Aus den Auszahlungen eines Spiels ist nicht ablesbar, wer genau wen bezahlt hat (beim Sauspiel zahlt jeder
//! Verlierer an beide Gewinner). Die Verlierer zahlen deshalb im Verhältnis der Gewinne an die Gewinner,
//! auf den Cent genau. Strafen laufen gleich. Beim Stock zahlen die, die eingezahlt haben (Einlage,
//! Verdopplung), im Verhältnis ihres Anteils am Stock an die, die ihn bekommen.

use crate::model::Cents;
use crate::table::Table;

/// `flows[von][an]`: was Spieler `von` an Spieler `an` gezahlt hat, in Cent.
pub type FlowMatrix = Vec<Vec<Cents>>;

/// Ein Sitzplatz und ein Betrag.
type Part = (usize, Cents);

/// Kürzt die Liste von hinten, bis sie genau `total` ergibt (sie ergibt mindestens so viel).
fn trimmed(parts: &[Part], total: Cents) -> Vec<Part> {
    let mut over = parts.iter().map(|p| p.1).sum::<Cents>() - total;
    let mut out = parts.to_vec();
    for p in out.iter_mut().rev() {
        let cut = over.min(p.1);
        p.1 -= cut;
        over -= cut;
    }
    out
}

/// Teilt `total` im Verhältnis der Gewichte auf, auf den Cent genau (größte Reste zuerst).
fn proportional(total: Cents, weights: &[Cents]) -> Vec<Cents> {
    let sum: Cents = weights.iter().sum();
    if sum <= 0 {
        return vec![0; weights.len()];
    }
    let mut shares: Vec<Cents> = weights.iter().map(|w| w * total / sum).collect();
    let mut order: Vec<usize> = (0..weights.len()).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(weights[i] * total % sum));
    let mut left = total - shares.iter().sum::<Cents>();
    for i in order {
        if left > 0 {
            shares[i] += 1;
            left -= 1;
        }
    }
    shares
}

/// Verteilt, was die Zahler (`rows`) zahlen, im Verhältnis auf die Empfänger (`cols`) und addiert es auf
/// `out[von][an]`. Bewegt wird das kleinere der beiden Gesamtbeträge.
fn spread(out: &mut [Vec<Cents>], rows: &[Part], cols: &[Part]) {
    let total = rows
        .iter()
        .map(|r| r.1)
        .sum::<Cents>()
        .min(cols.iter().map(|c| c.1).sum());
    if total <= 0 {
        return;
    }
    let rows = trimmed(rows, total);
    let cols = trimmed(cols, total);
    // Erst abgerundet im Verhältnis, dann die fehlenden Cent dort, wo Zeile und Spalte noch offen sind.
    let mut cells: Vec<Vec<Cents>> = rows
        .iter()
        .map(|r| cols.iter().map(|c| r.1 * c.1 / total).collect())
        .collect();
    let mut row_left: Vec<Cents> = rows
        .iter()
        .zip(&cells)
        .map(|(r, line)| r.1 - line.iter().sum::<Cents>())
        .collect();
    let mut col_left: Vec<Cents> = cols
        .iter()
        .enumerate()
        .map(|(c, col)| col.1 - cells.iter().map(|line| line[c]).sum::<Cents>())
        .collect();
    for (r, line) in cells.iter_mut().enumerate() {
        for (c, cell) in line.iter_mut().enumerate() {
            let put = row_left[r].min(col_left[c]);
            *cell += put;
            row_left[r] -= put;
            col_left[c] -= put;
        }
    }
    for (r, &(from, _)) in rows.iter().enumerate() {
        for (c, &(to, _)) in cols.iter().enumerate() {
            if from != to {
                out[from][to] += cells[r][c];
            }
        }
    }
}

/// Wer zahlt (negativ) und wer bekommt (positiv) in einer Zahlungsliste.
fn sides(payouts: &[Cents]) -> (Vec<Part>, Vec<Part>) {
    let pick = |sign: Cents| -> Vec<Part> {
        payouts
            .iter()
            .enumerate()
            .filter(|(_, &p)| p * sign > 0)
            .map(|(i, &p)| (i, p * sign))
            .collect()
    };
    (pick(-1), pick(1))
}

/// Wer wem wie viel gezahlt hat: Spiele, Strafen und Stock eines Abends, Sitzplatz für Sitzplatz.
pub fn money_flows(table: &Table) -> FlowMatrix {
    let n = table.players.len();
    let mut out = vec![vec![0; n]; n];
    for (number, game) in table.games().iter().enumerate() {
        let (payers, receivers) = sides(&game.payouts);
        // Zwei gegen zwei mit gleichen Beträgen (Sauspiel, Hochzeit): Jeder Verlierer zahlt seinen ganzen Betrag
        // an einen Gewinner, so fließen nie halbe Preise. Von Spiel zu Spiel wechselt die Zuordnung, damit
        // keiner dauernd an denselben zahlt.
        if let ([a, b], [c, d]) = (payers.as_slice(), receivers.as_slice()) {
            if a.1 == b.1 && c.1 == d.1 && a.1 == c.1 {
                let (first, second) = if number % 2 == 0 { (*c, *d) } else { (*d, *c) };
                spread(&mut out, &[*a], &[first]);
                spread(&mut out, &[*b], &[second]);
                continue;
            }
        }
        spread(&mut out, &payers, &receivers);
    }
    for offence in &table.offences {
        let (payers, receivers) = sides(&offence.payouts);
        spread(&mut out, &payers, &receivers);
    }
    // Im Stock liegt, was jeder eingezahlt hat. Wer etwas bekommt, bekommt es im Verhältnis dazu von den
    // Einzahlern.
    let mut pool = vec![0 as Cents; n];
    for m in &table.stock_moves {
        let payouts: Vec<Cents> = m.payouts.iter().copied().take(n).collect();
        let (payers, receivers) = sides(&payouts);
        for &(seat, v) in &payers {
            pool[seat] += v;
        }
        let wanted: Cents = receivers.iter().map(|r| r.1).sum();
        let available: Cents = pool.iter().sum();
        let taken = wanted.min(available);
        if taken <= 0 {
            continue;
        }
        let shares = proportional(taken, &pool);
        let from_pool: Vec<Part> = shares
            .iter()
            .copied()
            .enumerate()
            .filter(|p| p.1 > 0)
            .collect();
        spread(&mut out, &from_pool, &receivers);
        for (seat, s) in shares.into_iter().enumerate() {
            pool[seat] -= s;
        }
    }
    out
}

/// Die Flüsse über mehrere Abende, Spieler über den Namen zusammengeführt (in der Reihenfolge von
/// `OverallHistory::names`).
pub fn overall_flows(tables: &[&Table], names: &[String]) -> FlowMatrix {
    let mut out = vec![vec![0; names.len()]; names.len()];
    for table in tables {
        let seats: Vec<Option<usize>> = table
            .players
            .iter()
            .map(|p| names.iter().position(|n| n == p))
            .collect();
        for (a, row) in money_flows(table).iter().enumerate() {
            for (b, &v) in row.iter().enumerate() {
                if let (Some(Some(x)), Some(Some(y))) = (seats.get(a), seats.get(b)) {
                    out[*x][*y] += v;
                }
            }
        }
    }
    out
}
