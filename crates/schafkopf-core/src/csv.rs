use crate::model::{Cents, Seat};
use crate::table::{Offence, Table, TableEvent};

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
            "Sie",
            "Gewonnen",
            "Schneider",
            "Schwarz",
            "Laufende",
            "Gegenlauf",
            "Bockspiel",
            "Bockrunde_ausgeloest",
            "Bockrunden_Anzahl",
            "Bock_Verdopplungen",
            "Augen",
            "Preis_Cent",
            "Preis_berechnet_Cent",
            "Falsch_verlangt_von",
            "Fehlerarten",
            "Zurueckverlangt",
            "Zurueckverlangt_von",
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
            yes_no(g.sie),
            yes_no(g.won),
            yes_no(g.schneider || g.schwarz),
            yes_no(g.schwarz),
            g.laufende.to_string(),
            yes_no(g.gegenlauf),
            yes_no(g.bock),
            yes_no(g.starts_bock),
            g.bock_triggers().to_string(),
            g.bock_doublings().to_string(),
            opt(g.points.map(|p| p.to_string())),
            game.value.to_string(),
            game.calculated_value.to_string(),
            self.names_of(&g.price_error_by),
            g.mistakes
                .iter()
                .map(|m| m.label())
                .collect::<Vec<_>>()
                .join("; "),
            yes_no(g.reclaimed),
            self.names_of(&g.reclaimed_by),
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

impl Table {
    /// Das Protokoll des Abends als CSV: Beginn, Kommen, Gehen, Umsetzen und Ende, je mit Uhrzeit
    /// (Ortszeit) und der Zahl der bis dahin gespielten Spiele.
    pub fn events_csv_with(&self, separator: char) -> String {
        let header = [
            "Zeit",
            "Zeit_Unix",
            "Ereignis",
            "Spieler",
            "Hinter",
            "Nach_Spiel_Nr",
            "Strafe_je_Mitspieler_Cent",
        ];
        let mut lines = vec![row(header.map(String::from), separator)];
        let time = |at: Option<i64>| {
            (
                at.map(|t| crate::clock::format_local(t, self.utc_offset_min))
                    .unwrap_or_default(),
                at.map(|t| t.to_string()).unwrap_or_default(),
            )
        };
        let mut push = |at: Option<i64>,
                        what: &str,
                        who: String,
                        behind: String,
                        games: usize,
                        penalty: String| {
            let (local, unix) = time(at);
            lines.push(row(
                [
                    local,
                    unix,
                    what.to_owned(),
                    who,
                    behind,
                    games.to_string(),
                    penalty,
                ],
                separator,
            ));
        };
        push(
            self.created_at,
            "Abend beginnt",
            String::new(),
            String::new(),
            0,
            String::new(),
        );
        // Kommen und Gehen sowie Vergehen in der Reihenfolge der Spiele, bei gleichem Stand wie eingetragen.
        let mut entries: Vec<(usize, Option<&TableEvent>, Option<&Offence>)> = self
            .events
            .iter()
            .map(|e| (e.after_games, Some(e), None))
            .chain(self.offences.iter().map(|o| (o.after_games, None, Some(o))))
            .collect();
        entries.sort_by_key(|&(games, ..)| games);
        for (_, event, offence) in entries {
            if let Some(e) = event {
                let behind = e
                    .behind
                    .and_then(|b| self.players.get(b).cloned())
                    .unwrap_or_default();
                let who = self.players.get(e.seat).cloned().unwrap_or_default();
                push(
                    e.at,
                    e.kind.label(),
                    who,
                    behind,
                    e.after_games,
                    String::new(),
                );
            }
            if let Some(o) = offence {
                let who = self.players.get(o.offender).cloned().unwrap_or_default();
                let what = format!("Vergehen: {}", o.kind.label());
                push(
                    o.at,
                    &what,
                    who,
                    String::new(),
                    o.after_games,
                    o.amount.to_string(),
                );
            }
        }
        if let Some(end) = self.ended_at {
            push(
                Some(end),
                "Abend endet",
                String::new(),
                String::new(),
                self.games().len(),
                String::new(),
            );
        }
        let mut out = lines.join("\n");
        out.push('\n');
        out
    }

    /// Das Protokoll mit Komma als Trennzeichen, siehe [`Table::events_csv_with`].
    pub fn events_csv(&self) -> String {
        self.events_csv_with(',')
    }
}
