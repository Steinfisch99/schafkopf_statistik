//! Export der Spiele als CSV-Datei.

use schafkopf_core::Table;

/// UTF-8-Kennung (BOM) am Dateianfang. Ohne sie raten viele Programme die Zeichenkodierung und
/// zeigen Umlaute falsch an.
const UTF8_BOM: char = '\u{feff}';

/// Wie die CSV für andere Programme aufgebaut ist. Beide Formate sind UTF-8 mit Kennung.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CsvFormat {
    /// Semikolon als Trennzeichen: So teilt das deutsche Excel die Spalten richtig auf.
    Semicolon,
    /// Komma als Trennzeichen, wie es Tabellenkalkulationen mit englischen Einstellungen und
    /// Programme erwarten.
    Comma,
}

impl CsvFormat {
    pub fn label(self) -> &'static str {
        match self {
            CsvFormat::Semicolon => "Semikolon (deutsches Excel)",
            CsvFormat::Comma => "Komma (international)",
        }
    }

    /// Der Dateiinhalt mit der UTF-8-Kennung am Anfang.
    pub fn render(self, table: &Table) -> String {
        let separator = match self {
            CsvFormat::Semicolon => ';',
            CsvFormat::Comma => ',',
        };
        format!("{UTF8_BOM}{}", table.to_csv_with(separator))
    }

    /// Das Protokoll des Abends (Beginn, Kommen, Gehen, Umsetzen, Ende) mit UTF-8-Kennung.
    pub fn render_log(self, table: &Table) -> String {
        let separator = match self {
            CsvFormat::Semicolon => ';',
            CsvFormat::Comma => ',',
        };
        format!("{UTF8_BOM}{}", table.events_csv_with(separator))
    }
}

/// Datum (Jahr, Monat, Tag) in UTC zu Unix-Sekunden.
pub fn date_ymd(unix_secs: u64) -> (i64, u32, u32) {
    let z = (unix_secs / 86_400) as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if month <= 2 { year + 1 } else { year }, month, day)
}

/// Dateiname nach dem Tag, an dem der Abend begonnen wurde: `schafkopf-2026-10-17.csv`.
pub fn file_name(table: &Table, now_secs: u64) -> String {
    let secs = table.created_at.map_or(now_secs, |t| t.max(0) as u64);
    let (y, m, d) = date_ymd(secs);
    format!("schafkopf-{y:04}-{m:02}-{d:02}.csv")
}

/// Dateiname des Protokolls: `schafkopf-2026-10-17-protokoll.csv`.
pub fn log_file_name(table: &Table, now_secs: u64) -> String {
    let name = file_name(table, now_secs);
    format!("{}-protokoll.csv", name.trim_end_matches(".csv"))
}

/// Speichert die CSV und sagt, wo sie liegt. Auf Android im Ordner Downloads.
pub fn save_csv(file_name: &str, csv: &str) -> Result<String, String> {
    save_file(file_name, "text/csv", csv.as_bytes())
}

/// Dateiname der Sicherung aller Daten: `schafkopf-sicherung-2026-10-17.zip`.
pub fn backup_file_name(now_secs: u64) -> String {
    let (y, m, d) = date_ymd(now_secs);
    format!("schafkopf-sicherung-{y:04}-{m:02}-{d:02}.zip")
}

/// Speichert eine Datei und sagt, wo sie liegt. Auf Android im Ordner Downloads.
pub fn save_file(file_name: &str, mime: &str, data: &[u8]) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        crate::android::save_to_downloads(file_name, mime, data)
            .map(|()| format!("Downloads/{file_name}"))
    }
    #[cfg(not(target_os = "android"))]
    {
        let _ = mime;
        let dir = std::env::var_os("HOME")
            .map(|home| std::path::PathBuf::from(home).join("Downloads"))
            .filter(|dir| dir.is_dir())
            .unwrap_or_else(crate::storage::data_dir);
        save_in(&dir, file_name, data)
    }
}

/// Schreibt die Datei in einen Ordner und legt ihn bei Bedarf an. Bestehende Dateien bleiben
/// erhalten, der neue Name bekommt dann eine Nummer.
#[cfg(not(target_os = "android"))]
fn save_in(
    dir: &std::path::Path,
    file_name: &str,
    data: impl AsRef<[u8]>,
) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Ordner anlegen: {e}"))?;
    let (stem, ext) = file_name.rsplit_once('.').unwrap_or((file_name, ""));
    let mut path = dir.join(file_name);
    let mut n = 1;
    while path.exists() {
        path = dir.join(format!("{stem} ({n}).{ext}"));
        n += 1;
    }
    std::fs::write(&path, data).map_err(|e| format!("Schreiben: {e}"))?;
    Ok(path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use schafkopf_core::{GameInput, GameKind, Tariff};

    fn table() -> Table {
        let players = ["Anna", "Ben", "Cem", "Dora"].map(String::from).to_vec();
        let mut t = Table::new("T", players, 0, Tariff::default()).unwrap();
        t.add_game(GameInput::new(0, GameKind::Wenz, 1)).unwrap();
        t
    }

    #[test]
    fn converts_unix_time_to_dates() {
        assert_eq!(date_ymd(0), (1970, 1, 1));
        assert_eq!(date_ymd(951_782_400), (2000, 2, 29));
        assert_eq!(date_ymd(1_000_000_000), (2001, 9, 9));
        assert_eq!(date_ymd(1_709_164_799), (2024, 2, 28));
        assert_eq!(date_ymd(1_709_164_800), (2024, 2, 29));
        assert_eq!(date_ymd(1_709_251_200), (2024, 3, 1));
        assert_eq!(date_ymd(1_798_761_599), (2026, 12, 31));
    }

    #[test]
    fn file_name_uses_start_of_evening() {
        let mut t = table();
        assert_eq!(file_name(&t, 1_000_000_000), "schafkopf-2001-09-09.csv");
        t.created_at = Some(951_782_400);
        assert_eq!(file_name(&t, 1_000_000_000), "schafkopf-2000-02-29.csv");
    }

    #[test]
    fn both_formats_are_utf8_with_bom_and_umlauts_survive() {
        let players = ["Jürgen", "Zoë", "Ärmel", "Dora"]
            .map(String::from)
            .to_vec();
        let mut t = Table::new("T", players, 0, Tariff::default()).unwrap();
        t.add_game(GameInput::new(0, GameKind::Wenz, 1)).unwrap();
        for format in [CsvFormat::Semicolon, CsvFormat::Comma] {
            let text = format.render(&t);
            let bytes = text.as_bytes();
            assert_eq!(
                &bytes[..3],
                [0xEF, 0xBB, 0xBF],
                "{format:?} beginnt mit der UTF-8-Kennung"
            );
            let decoded = std::str::from_utf8(bytes).unwrap();
            assert!(decoded.contains("Jürgen Gewinn_Cent") && decoded.contains("Ärmel Stand_Cent"));
        }
    }

    #[test]
    fn formats_differ_only_in_the_separator() {
        let t = table();
        let semicolon = CsvFormat::Semicolon.render(&t);
        let comma = CsvFormat::Comma.render(&t);
        assert!(semicolon.starts_with("\u{feff}Nr;Zeit;"));
        assert!(comma.starts_with("\u{feff}Nr,Zeit,"));
        assert_eq!(semicolon.lines().count(), comma.lines().count());
    }

    #[test]
    fn saving_never_overwrites_an_existing_file() {
        let dir = std::env::temp_dir().join(format!("schafkopf-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let first = save_in(&dir, "a.csv", "eins").unwrap();
        let second = save_in(&dir, "a.csv", "zwei").unwrap();
        assert!(first.ends_with("a.csv") && second.ends_with("a (1).csv"));
        assert_eq!(std::fs::read_to_string(first).unwrap(), "eins");
        assert_eq!(std::fs::read_to_string(second).unwrap(), "zwei");
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
