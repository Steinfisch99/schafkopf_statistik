//! Export der Spiele als CSV-Datei.

use schafkopf_core::Table;

/// Wie die CSV für andere Programme aufgebaut ist.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CsvFormat {
    /// Semikolon und UTF-8 mit Kennung (BOM): so öffnet das deutsche Excel Umlaute und Spalten richtig.
    Excel,
    /// Komma, wie es Tabellenkalkulationen mit englischen Einstellungen und Programme erwarten.
    Standard,
}

impl CsvFormat {
    pub fn label(self) -> &'static str {
        match self {
            CsvFormat::Excel => "Für Excel (Semikolon)",
            CsvFormat::Standard => "Allgemein (Komma)",
        }
    }

    /// Der Dateiinhalt, bei [`CsvFormat::Excel`] mit der UTF-8-Kennung am Anfang.
    pub fn render(self, table: &Table) -> String {
        match self {
            CsvFormat::Excel => format!("\u{feff}{}", table.to_csv_with(';')),
            CsvFormat::Standard => table.to_csv(),
        }
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

/// Speichert die CSV und sagt, wo sie liegt. Auf Android im Ordner Downloads.
pub fn save_csv(file_name: &str, csv: &str) -> Result<String, String> {
    #[cfg(target_os = "android")]
    {
        crate::android::save_to_downloads(file_name, "text/csv", csv.as_bytes())
            .map(|()| format!("Downloads/{file_name}"))
    }
    #[cfg(not(target_os = "android"))]
    {
        let dir = std::env::var_os("HOME")
            .map(|home| std::path::PathBuf::from(home).join("Downloads"))
            .filter(|dir| dir.is_dir())
            .unwrap_or_else(crate::storage::data_dir);
        save_in(&dir, file_name, csv)
    }
}

/// Schreibt die Datei in einen Ordner und legt ihn bei Bedarf an. Bestehende Dateien bleiben
/// erhalten, der neue Name bekommt dann eine Nummer.
#[cfg(not(target_os = "android"))]
fn save_in(dir: &std::path::Path, file_name: &str, csv: &str) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("Ordner anlegen: {e}"))?;
    let (stem, ext) = file_name.rsplit_once('.').unwrap_or((file_name, ""));
    let mut path = dir.join(file_name);
    let mut n = 1;
    while path.exists() {
        path = dir.join(format!("{stem} ({n}).{ext}"));
        n += 1;
    }
    std::fs::write(&path, csv).map_err(|e| format!("Schreiben: {e}"))?;
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
    fn excel_format_has_bom_and_semicolons() {
        let t = table();
        let excel = CsvFormat::Excel.render(&t);
        assert!(excel.starts_with("\u{feff}Nr;Zeit;"));
        assert!(!CsvFormat::Standard.render(&t).starts_with('\u{feff}'));
        assert!(CsvFormat::Standard.render(&t).starts_with("Nr,Zeit,"));
        assert_eq!(
            excel.trim_start_matches('\u{feff}').lines().count(),
            CsvFormat::Standard.render(&t).lines().count()
        );
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
