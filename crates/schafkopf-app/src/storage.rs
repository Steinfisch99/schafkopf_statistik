//! Speichern der Tisch-Datei (JSON) und des Spieljournals (CSV).

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use schafkopf_core::Table;

const JSON_FILE: &str = "schafkopf-tisch.json";
const CSV_FILE: &str = "schafkopf-spiele.csv";

/// Android-App-Kennung aus `Dioxus.toml`; bestimmt das private Datenverzeichnis der App.
#[cfg(target_os = "android")]
const APP_ID: &str = "de.schafkopf.statistik";

/// Ordner, in den gespeichert wird.
///
/// Vorrang hat `SCHAFKOPF_DATA_DIR`. Auf Android ist es das private Verzeichnis der App
/// (`/data/data/<App-Kennung>/files`), sonst das Heimverzeichnis oder der aktuelle Ordner.
/// Ob der Android-Pfad auf dem Gerät wirklich beschreibbar ist, soll der Prototyp zeigen.
pub fn data_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("SCHAFKOPF_DATA_DIR") {
        return PathBuf::from(dir);
    }
    #[cfg(target_os = "android")]
    {
        PathBuf::from(format!("/data/data/{APP_ID}/files"))
    }
    #[cfg(not(target_os = "android"))]
    {
        match std::env::var_os("HOME") {
            Some(home) => PathBuf::from(home).join("schafkopf"),
            None => PathBuf::from("schafkopf-daten"),
        }
    }
}

/// Lädt den zuletzt gespeicherten Tisch, falls es einen gibt und er lesbar ist.
pub fn load() -> Option<Table> {
    let json = fs::read_to_string(data_dir().join(JSON_FILE)).ok()?;
    Table::from_json(&json).ok()
}

/// Schreibt den Tisch als JSON neu und bringt die CSV auf den Stand.
///
/// Mit `new_game = Some(index)` wird nur die Zeile dieses Spiels an die CSV angehängt, außer
/// es ist das erste Spiel; dann wird die Datei samt Kopfzeile neu geschrieben, damit nie die
/// Reste eines früheren Tisches in der Datei bleiben.
pub fn save(table: &Table, new_game: Option<usize>) -> io::Result<PathBuf> {
    let dir = data_dir();
    fs::create_dir_all(&dir)?;
    let json = table
        .to_json()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(dir.join(JSON_FILE), json)?;

    let csv_path = dir.join(CSV_FILE);
    match new_game {
        Some(index) if index > 0 => {
            let row = table.csv_row(index).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Spiel nicht gefunden")
            })?;
            let mut file = OpenOptions::new()
                .append(true)
                .create(true)
                .open(&csv_path)?;
            writeln!(file, "{row}")?;
        }
        _ => fs::write(&csv_path, table.to_csv())?,
    }
    Ok(dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use schafkopf_core::{GameInput, GameKind, Tariff};

    #[test]
    fn csv_grows_by_one_row_per_game_and_json_roundtrips() {
        let dir = std::env::temp_dir().join(format!("schafkopf-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        std::env::set_var("SCHAFKOPF_DATA_DIR", &dir);

        let players = ["A", "B", "C", "D"].map(String::from).to_vec();
        let mut table = Table::new("T", players, 0, Tariff::default()).unwrap();
        save(&table, None).unwrap();
        let csv = || fs::read_to_string(dir.join(CSV_FILE)).unwrap();
        assert_eq!(csv().lines().count(), 1);

        for i in 0..3 {
            table
                .add_game(GameInput::new(table.next_dealer(), GameKind::Wenz, 1))
                .unwrap();
            save(&table, Some(i)).unwrap();
            assert_eq!(csv().lines().count(), i + 2);
        }
        assert_eq!(csv(), table.to_csv());
        assert_eq!(load().unwrap(), table);
        fs::remove_dir_all(&dir).unwrap();
    }
}
