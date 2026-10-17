//! Speichern der Tisch-Datei (JSON) und des Spieljournals (CSV).

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use schafkopf_core::{Library, Table};

const JSON_FILE: &str = "schafkopf-tisch.json";
const CSV_FILE: &str = "schafkopf-spiele.csv";
const ARCHIVE_DIR: &str = "archiv";
const LIBRARY_FILE: &str = "schafkopf-bibliothek.json";

/// Android-App-Kennung aus `Dioxus.toml`; bestimmt das private Datenverzeichnis der App.
#[cfg(target_os = "android")]
const APP_ID: &str = "de.schafkopf.statistik";

/// Ordner, in den gespeichert wird.
///
/// Vorrang hat `SCHAFKOPF_DATA_DIR`. Auf Android ist es das private Verzeichnis der App
/// (`/data/data/<App-Kennung>/files`), sonst das Heimverzeichnis oder der aktuelle Ordner.
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
    load_from(&data_dir())
}

/// Schreibt den Tisch als JSON neu und bringt die CSV auf den Stand, siehe [`save_in`].
pub fn save(table: &Table, new_game: Option<usize>) -> io::Result<PathBuf> {
    let dir = data_dir();
    save_in(&dir, table, new_game)?;
    Ok(dir)
}

/// Legt den laufenden Abend ins Archiv, bevor ein neuer beginnt. `stamp` macht die Namen eindeutig.
pub fn archive_current(stamp: u64) -> io::Result<bool> {
    archive_in(&data_dir(), stamp)
}

/// Lädt die Bibliothek mit Regelwerken und Tarifen. Gibt es noch keine oder ist die Datei nicht
/// lesbar, beginnt die App mit den Voreinstellungen.
pub fn load_library() -> Library {
    load_library_from(&data_dir())
}

/// Speichert die Bibliothek und gibt den Ordner zurück.
pub fn save_library(library: &Library) -> io::Result<PathBuf> {
    let dir = data_dir();
    save_library_in(&dir, library)?;
    Ok(dir)
}

fn load_library_from(dir: &Path) -> Library {
    fs::read_to_string(dir.join(LIBRARY_FILE))
        .ok()
        .and_then(|json| Library::from_json(&json).ok())
        .unwrap_or_else(Library::with_defaults)
}

fn save_library_in(dir: &Path, library: &Library) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let json = library
        .to_json()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(dir.join(LIBRARY_FILE), json)
}

fn load_from(dir: &Path) -> Option<Table> {
    let json = fs::read_to_string(dir.join(JSON_FILE)).ok()?;
    Table::from_json(&json).ok()
}

/// Mit `new_game = Some(index)` wird nur die Zeile dieses Spiels an die CSV angehängt, außer
/// es ist das erste Spiel. Dann (und bei `None`, etwa nach Tarifänderung oder Rückgängig) wird
/// die Datei samt Kopfzeile neu geschrieben, damit sie immer zum Tisch passt.
fn save_in(dir: &Path, table: &Table, new_game: Option<usize>) -> io::Result<()> {
    fs::create_dir_all(dir)?;
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
    Ok(())
}

fn archive_in(dir: &Path, stamp: u64) -> io::Result<bool> {
    let mut moved = false;
    for file in [JSON_FILE, CSV_FILE] {
        let from = dir.join(file);
        if from.exists() {
            let archive = dir.join(ARCHIVE_DIR);
            fs::create_dir_all(&archive)?;
            let (name, ext) = file.rsplit_once('.').unwrap_or((file, ""));
            fs::rename(&from, archive.join(format!("{name}-{stamp}.{ext}")))?;
            moved = true;
        }
    }
    Ok(moved)
}

#[cfg(test)]
mod tests {
    use super::*;
    use schafkopf_core::{GameInput, GameKind, Tariff};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("schafkopf-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn table() -> Table {
        let players = ["A", "B", "C", "D"].map(String::from).to_vec();
        Table::new("T", players, 0, Tariff::default()).unwrap()
    }

    #[test]
    fn csv_grows_by_one_row_per_game_and_json_roundtrips() {
        let dir = temp_dir("save");
        let mut table = table();
        save_in(&dir, &table, None).unwrap();
        let csv = || fs::read_to_string(dir.join(CSV_FILE)).unwrap();
        assert_eq!(csv().lines().count(), 1);

        for i in 0..3 {
            table
                .add_game(GameInput::new(table.next_dealer(), GameKind::Wenz, 1))
                .unwrap();
            save_in(&dir, &table, Some(i)).unwrap();
            assert_eq!(csv().lines().count(), i + 2);
        }
        assert_eq!(csv(), table.to_csv());
        assert_eq!(load_from(&dir).unwrap(), table);

        table.undo_last();
        save_in(&dir, &table, None).unwrap();
        assert_eq!(csv(), table.to_csv());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn library_roundtrips_and_falls_back_to_defaults() {
        let dir = temp_dir("library");
        assert_eq!(
            load_library_from(&dir),
            Library::with_defaults(),
            "noch keine Datei"
        );
        let mut library = Library::with_defaults();
        library
            .save_tariff(
                None,
                "Zweiter",
                Tariff {
                    sauspiel: 30,
                    ..Tariff::default()
                },
            )
            .unwrap();
        save_library_in(&dir, &library).unwrap();
        assert_eq!(load_library_from(&dir), library);
        // Eine kaputte Datei lässt die App nicht abstürzen.
        fs::write(dir.join(LIBRARY_FILE), "{ kaputt").unwrap();
        assert_eq!(load_library_from(&dir), Library::with_defaults());
        // Der Abend wird beim Archivieren nicht von der Bibliothek berührt.
        save_library_in(&dir, &library).unwrap();
        save_in(&dir, &table(), None).unwrap();
        archive_in(&dir, 5).unwrap();
        assert_eq!(load_library_from(&dir), library);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn archive_moves_both_files_and_keeps_older_archives() {
        let dir = temp_dir("archive");
        assert!(!archive_in(&dir, 1).unwrap(), "nichts zu archivieren");

        let table = table();
        save_in(&dir, &table, None).unwrap();
        assert!(archive_in(&dir, 100).unwrap());
        assert!(load_from(&dir).is_none());
        assert!(!dir.join(CSV_FILE).exists());
        assert!(dir.join("archiv/schafkopf-tisch-100.json").exists());
        assert!(dir.join("archiv/schafkopf-spiele-100.csv").exists());

        save_in(&dir, &table, None).unwrap();
        assert!(archive_in(&dir, 200).unwrap());
        assert_eq!(fs::read_dir(dir.join(ARCHIVE_DIR)).unwrap().count(), 4);
        fs::remove_dir_all(&dir).unwrap();
    }
}
