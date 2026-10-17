use crate::model::{Cents, Seat};
use crate::table::Table;

/// Maskiert ein Feld nach RFC 4180: Felder mit Trennzeichen, Anführungszeichen oder Zeilenumbruch
/// kommen in Anführungszeichen, enthaltene Anführungszeichen werden verdoppelt.
fn field(value: &str, separator: char) -> String {
    if value.contains([separator, '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn row(fields: impl IntoIterator<Item = String>, separator: char) -> String {
    let fields: Vec<String> = fields.into_iter().map(|f| field(&f, separator)).collect();
    fields.join(&separator.to_string())
}

fn yes_no(b: bool) -> String {
    if b { "ja" } else { "nein" }.to_owned()
}

impl Table {
    fn names_of(&self, seats: &[Seat]) -> String {
        let names: Vec<&str> = seats.iter().map(|&s| self.players[s].as_str()).collect();
        names.join("; ")
    }

    /// Kopfzeile der CSV-Datei. Beträge stehen in Cent. Pro Spieler gibt es zwei Spalten:
    /// den Gewinn oder Verlust im Spiel und den Kontostand danach.
    pub fn csv_header_with(&self, separator: char) -> String {
        let fixed = [
            "Nr",
            "Zeit",
            "Geber",
            "Spielart",
            "Farbe",
            "Spieler",
            "Mitspieler",
            "Mitspielende",
            "Geklopft",
            "Kontra",
            "Re",
            "Tout",
            "Gewonnen",
            "Schneider",
            "Schwarz",
            "Laufende",
            "Gegenlauf",
            "Bockspiel",
            "Bockrunde_ausgeloest",
            "Augen",
            "Preis_Cent",
            "Preis_berechnet_Cent",
            "Notiz",
        ];
        let players = self
            .players
            .iter()
            .flat_map(|p| [format!("{p} Gewinn_Cent"), format!("{p} Stand_Cent")]);
        row(
            fixed.into_iter().map(String::from).chain(players),
            separator,
        )
    }

    /// Eine Zeile pro Spiel. `index` ist die Position in [`Table::games`], beginnend bei 0.
    /// Gibt `None` zurück, wenn es das Spiel nicht gibt.
    ///
    /// Die App kann nach jedem Eintrag diese Zeile an die Datei anhängen.
    pub fn csv_row_with(&self, index: usize, separator: char) -> Option<String> {
        let game = self.games().get(index)?;
        let g = &game.input;
        let opt = |v: Option<String>| v.unwrap_or_default();
        let mut standing: Vec<Cents> = vec![0; self.players.len()];
        for earlier in &self.games()[..=index] {
            for (s, p) in standing.iter_mut().zip(&earlier.payouts) {
                *s += p;
            }
        }
        let fixed = [
            (index + 1).to_string(),
            opt(g.played_at.map(|t| t.to_string())),
            self.players[g.dealer].clone(),
            g.kind.label().to_owned(),
            opt(g.suit.map(|s| s.label().to_owned())),
            self.players[g.declarer].clone(),
            opt(g.partner.map(|p| self.players[p].clone())),
            self.names_of(&g.participants),
            self.names_of(&g.knockers),
            opt(g.kontra_by.map(|k| self.players[k].clone())),
            yes_no(g.re),
            yes_no(g.tout),
            yes_no(g.won),
            yes_no(g.schneider || g.schwarz),
            yes_no(g.schwarz),
            g.laufende.to_string(),
            yes_no(g.gegenlauf),
            yes_no(g.bock),
            yes_no(g.starts_bock),
            opt(g.points.map(|p| p.to_string())),
            game.value.to_string(),
            game.calculated_value.to_string(),
            opt(g.note.clone()),
        ];
        let per_player = game
            .payouts
            .iter()
            .zip(&standing)
            .flat_map(|(p, s)| [p.to_string(), s.to_string()]);
        Some(row(fixed.into_iter().chain(per_player), separator))
    }

    /// Die komplette CSV-Datei: Kopfzeile und alle Spiele.
    pub fn to_csv_with(&self, separator: char) -> String {
        let mut out = self.csv_header_with(separator);
        out.push('\n');
        for i in 0..self.games().len() {
            out.push_str(&self.csv_row_with(i, separator).expect("Index im Bereich"));
            out.push('\n');
        }
        out
    }
}

impl Table {
    /// Kopfzeile der CSV-Datei mit Komma als Trennzeichen, siehe [`Table::csv_header_with`].
    pub fn csv_header(&self) -> String {
        self.csv_header_with(',')
    }

    /// Eine Zeile pro Spiel mit Komma als Trennzeichen, siehe [`Table::csv_row_with`].
    pub fn csv_row(&self, index: usize) -> Option<String> {
        self.csv_row_with(index, ',')
    }

    /// Die komplette CSV-Datei mit Komma als Trennzeichen.
    pub fn to_csv(&self) -> String {
        self.to_csv_with(',')
    }
}
