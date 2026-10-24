//! Speichern der Tisch-Datei (JSON) und des Spieljournals (CSV).

use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use schafkopf_core::{Library, Table};

const JSON_FILE: &str = schafkopf_core::CURRENT_FILE;
const CSV_FILE: &str = "schafkopf-spiele.csv";
const EVENTS_FILE: &str = "schafkopf-ereignisse.csv";
const ARCHIVE_DIR: &str = "archiv";
const LIBRARY_FILE: &str = schafkopf_core::LIBRARY_FILE;

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

/// Ein früherer Abend aus dem Archiv. `stamp` ist der Zeitpunkt, an dem er ins Archiv kam.
#[derive(Debug, Clone, PartialEq)]
pub struct PastEvening {
    pub stamp: u64,
    pub table: Table,
}

/// Alle früheren Abende, der neueste zuerst. Dateien, die sich nicht lesen lassen, werden
/// übersprungen, damit ein beschädigter Abend nicht alle anderen unsichtbar macht.
pub fn list_past() -> Vec<PastEvening> {
    list_past_in(&data_dir())
}

/// Lädt einen früheren Abend.
pub fn load_past(stamp: u64) -> Option<Table> {
    load_past_in(&data_dir(), stamp)
}

/// Speichert einen früheren Abend neu, zum Beispiel nachdem er als Testabend markiert wurde.
pub fn save_past(stamp: u64, table: &Table) -> io::Result<()> {
    save_past_in(&data_dir(), stamp, table)
}

/// Löscht einen früheren Abend samt seiner CSV. Fehlt eine der Dateien, ist das kein Fehler.
pub fn delete_past(stamp: u64) -> io::Result<()> {
    delete_past_in(&data_dir(), stamp)
}

/// Macht einen früheren Abend wieder zum laufenden, damit er sich bearbeiten lässt. Der Abend wird
/// erst als laufender gespeichert und dann aus dem Archiv entfernt, damit nichts verloren geht.
/// Ein vorhandener laufender Abend wird überschrieben, das prüft der Aufrufer.
pub fn reopen_past(stamp: u64) -> io::Result<Option<Table>> {
    reopen_past_in(&data_dir(), stamp)
}

/// Verwirft den laufenden Abend, ohne ihn zu archivieren. Das ist für Abende ohne ein einziges
/// Spiel gedacht, damit das Archiv nicht mit leeren Abenden zuwächst.
pub fn discard_current() -> io::Result<()> {
    discard_in(&data_dir())
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

    fs::write(dir.join(EVENTS_FILE), table.events_csv())?;
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
    for file in [JSON_FILE, CSV_FILE, EVENTS_FILE] {
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

fn past_file(dir: &Path, stem: &str, stamp: u64, ext: &str) -> PathBuf {
    dir.join(ARCHIVE_DIR).join(format!("{stem}-{stamp}.{ext}"))
}

fn load_past_in(dir: &Path, stamp: u64) -> Option<Table> {
    let json = fs::read_to_string(past_file(dir, "schafkopf-tisch", stamp, "json")).ok()?;
    Table::from_json(&json).ok()
}

fn list_past_in(dir: &Path) -> Vec<PastEvening> {
    let Ok(entries) = fs::read_dir(dir.join(ARCHIVE_DIR)) else {
        return Vec::new();
    };
    let mut past: Vec<PastEvening> = entries
        .filter_map(|entry| {
            let name = entry.ok()?.file_name().into_string().ok()?;
            let stamp: u64 = name
                .strip_prefix("schafkopf-tisch-")?
                .strip_suffix(".json")?
                .parse()
                .ok()?;
            Some(PastEvening {
                stamp,
                table: load_past_in(dir, stamp)?,
            })
        })
        .collect();
    past.sort_by_key(|p| std::cmp::Reverse(p.stamp));
    past
}

/// Alle Dateien des Datenordners für die Sicherung: der laufende Abend, die Bibliothek und das
/// Archiv, dazu eine kurze Beschreibung. Die Namen sind die im Datenordner, mit `/` als Trenner.
pub fn backup_files(now_secs: u64) -> io::Result<Vec<(String, Vec<u8>)>> {
    backup_files_in(&data_dir(), now_secs)
}

/// Nimmt einen früheren Abend aus einer Sicherung ins Archiv auf, samt CSV-Dateien. `false`, wenn
/// es unter diesem Zeitpunkt schon einen Abend gibt (er bleibt dann, wie er ist).
pub fn import_past(stamp: u64, table: &Table) -> io::Result<bool> {
    import_past_in(&data_dir(), stamp, table)
}

fn backup_files_in(dir: &Path, now_secs: u64) -> io::Result<Vec<(String, Vec<u8>)>> {
    let mut files = Vec::new();
    for name in [LIBRARY_FILE, JSON_FILE, CSV_FILE, EVENTS_FILE] {
        match fs::read(dir.join(name)) {
            Ok(data) => files.push((name.to_owned(), data)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
    }
    if let Ok(entries) = fs::read_dir(dir.join(ARCHIVE_DIR)) {
        let mut archive: Vec<(String, Vec<u8>)> = Vec::new();
        for entry in entries {
            let entry = entry?;
            let Ok(name) = entry.file_name().into_string() else {
                continue;
            };
            if entry.file_type()?.is_file() {
                archive.push((format!("{ARCHIVE_DIR}/{name}"), fs::read(entry.path())?));
            }
        }
        archive.sort();
        files.extend(archive);
    }
    let (y, m, d) = crate::export::date_ymd(now_secs);
    let info = format!(
        "Schafkopf-Statistik: Sicherung vom {d}.{m}.{y}\n\n\
         schafkopf-bibliothek.json   Spieler, Regelwerke und Tarife\n\
         schafkopf-tisch.json        der laufende Abend\n\
         schafkopf-spiele.csv        seine Spiele als Tabelle\n\
         schafkopf-ereignisse.csv    sein Protokoll (Beginn, Ende, Kommen und Gehen)\n\
         archiv/                     alle früheren Abende (JSON, Spiele-CSV und Protokoll-CSV)\n\n\
         Einspielen: In der App unter \"Sicherung\" die ZIP-Datei wählen.\n\
         Die CSV-Dateien lassen sich auch direkt in Excel oder LibreOffice öffnen.\n"
    );
    files.push(("INFO.txt".to_owned(), info.into_bytes()));
    Ok(files)
}

fn import_past_in(dir: &Path, stamp: u64, table: &Table) -> io::Result<bool> {
    let json_path = past_file(dir, "schafkopf-tisch", stamp, "json");
    if json_path.exists() {
        return Ok(false);
    }
    fs::create_dir_all(dir.join(ARCHIVE_DIR))?;
    let json = table
        .to_json()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(&json_path, json)?;
    fs::write(
        past_file(dir, "schafkopf-spiele", stamp, "csv"),
        table.to_csv(),
    )?;
    fs::write(
        past_file(dir, "schafkopf-ereignisse", stamp, "csv"),
        table.events_csv(),
    )?;
    Ok(true)
}

fn save_past_in(dir: &Path, stamp: u64, table: &Table) -> io::Result<()> {
    let json = table
        .to_json()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    fs::write(past_file(dir, "schafkopf-tisch", stamp, "json"), json)
}

fn delete_past_in(dir: &Path, stamp: u64) -> io::Result<()> {
    for (stem, ext) in [
        ("schafkopf-tisch", "json"),
        ("schafkopf-spiele", "csv"),
        ("schafkopf-ereignisse", "csv"),
    ] {
        match fs::remove_file(past_file(dir, stem, stamp, ext)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(())
}

fn reopen_past_in(dir: &Path, stamp: u64) -> io::Result<Option<Table>> {
    let Some(table) = load_past_in(dir, stamp) else {
        return Ok(None);
    };
    save_in(dir, &table, None)?;
    delete_past_in(dir, stamp)?;
    Ok(Some(table))
}

fn discard_in(dir: &Path) -> io::Result<()> {
    for file in [JSON_FILE, CSV_FILE, EVENTS_FILE] {
        match fs::remove_file(dir.join(file)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(())
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
    fn past_evenings_are_listed_newest_first_and_can_be_deleted() {
        let dir = temp_dir("past");
        assert!(list_past_in(&dir).is_empty(), "noch kein Archiv");
        let mut first = table();
        first
            .add_game(GameInput::new(0, GameKind::Wenz, 1))
            .unwrap();
        save_in(&dir, &first, None).unwrap();
        archive_in(&dir, 100).unwrap();
        let mut second = table();
        second
            .add_game(GameInput::new(0, GameKind::Wenz, 2))
            .unwrap();
        second
            .add_game(GameInput::new(1, GameKind::Wenz, 2))
            .unwrap();
        save_in(&dir, &second, None).unwrap();
        archive_in(&dir, 200).unwrap();

        let list = list_past_in(&dir);
        assert_eq!(list.iter().map(|p| p.stamp).collect::<Vec<_>>(), [200, 100]);
        assert_eq!(list[0].table.games().len(), 2);
        assert_eq!(load_past_in(&dir, 100).unwrap(), first);
        assert!(load_past_in(&dir, 999).is_none());

        delete_past_in(&dir, 200).unwrap();
        assert_eq!(list_past_in(&dir).len(), 1);
        assert!(
            !past_file(&dir, "schafkopf-spiele", 200, "csv").exists(),
            "auch die CSV ist weg"
        );
        delete_past_in(&dir, 200).unwrap();
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn the_backup_holds_everything_and_import_skips_what_exists() {
        let dir = temp_dir("backup");
        let mut running = table();
        running
            .add_game(GameInput::new(0, GameKind::Wenz, 1))
            .unwrap();
        save_in(&dir, &running, None).unwrap();
        save_library_in(&dir, &Library::with_defaults()).unwrap();
        let mut old = table();
        old.add_game(GameInput::new(1, GameKind::Wenz, 2)).unwrap();
        assert!(import_past_in(&dir, 55, &old).unwrap());
        assert!(!import_past_in(&dir, 55, &old).unwrap(), "schon da");

        let files = backup_files_in(&dir, 1_000_000_000).unwrap();
        let names: Vec<&str> = files.iter().map(|(n, _)| n.as_str()).collect();
        for expected in [
            LIBRARY_FILE,
            JSON_FILE,
            CSV_FILE,
            EVENTS_FILE,
            "archiv/schafkopf-tisch-55.json",
            "archiv/schafkopf-spiele-55.csv",
            "archiv/schafkopf-ereignisse-55.csv",
            "INFO.txt",
        ] {
            assert!(names.contains(&expected), "{expected} fehlt in {names:?}");
        }
        // Über die ZIP und wieder zurück bleibt alles lesbar.
        let backup = schafkopf_core::read_backup(
            &schafkopf_core::unpack(&schafkopf_core::pack(&files).unwrap()).unwrap(),
        );
        assert!(backup.library.is_some());
        assert_eq!(backup.current.unwrap().games().len(), 1);
        assert_eq!(backup.past.len(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_backup_of_an_empty_folder_is_just_the_description() {
        let dir = temp_dir("backup-empty");
        let files = backup_files_in(&dir, 0).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].0, "INFO.txt");
    }
    #[test]
    fn a_past_evening_can_be_marked_as_a_test_evening() {
        let dir = temp_dir("debug");
        let mut table = table();
        save_in(&dir, &table, None).unwrap();
        archive_in(&dir, 7).unwrap();
        assert!(!load_past_in(&dir, 7).unwrap().debug);
        table.debug = true;
        save_past_in(&dir, 7, &table).unwrap();
        assert!(load_past_in(&dir, 7).unwrap().debug);
        assert_eq!(list_past_in(&dir).len(), 1);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unreadable_archive_files_are_skipped() {
        let dir = temp_dir("past-broken");
        let archive = dir.join(ARCHIVE_DIR);
        fs::create_dir_all(&archive).unwrap();
        fs::write(archive.join("schafkopf-tisch-5.json"), "{ kaputt").unwrap();
        fs::write(archive.join("schafkopf-tisch-abc.json"), "{}").unwrap();
        fs::write(archive.join("notizen.txt"), "x").unwrap();
        save_in(&dir, &table(), None).unwrap();
        archive_in(&dir, 7).unwrap();
        let list = list_past_in(&dir);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].stamp, 7);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn discarding_the_current_evening_leaves_no_archive() {
        let dir = temp_dir("discard");
        discard_in(&dir).unwrap();
        save_in(&dir, &table(), None).unwrap();
        discard_in(&dir).unwrap();
        assert!(load_from(&dir).is_none());
        assert!(!dir.join(CSV_FILE).exists());
        assert!(list_past_in(&dir).is_empty());
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
        assert!(dir.join("archiv/schafkopf-ereignisse-100.csv").exists());

        save_in(&dir, &table, None).unwrap();
        assert!(archive_in(&dir, 200).unwrap());
        assert_eq!(fs::read_dir(dir.join(ARCHIVE_DIR)).unwrap().count(), 6);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn reopening_a_past_evening_makes_it_current_again() {
        let dir = temp_dir("reopen");
        assert!(reopen_past_in(&dir, 9).unwrap().is_none());

        let mut table = table();
        table.finish(1234);
        save_in(&dir, &table, None).unwrap();
        archive_in(&dir, 9).unwrap();

        let reopened = reopen_past_in(&dir, 9).unwrap().unwrap();
        assert_eq!(reopened.ended_at, Some(1234));
        assert_eq!(load_from(&dir).unwrap().ended_at, Some(1234));
        assert!(dir.join(CSV_FILE).exists());
        assert!(list_past_in(&dir).is_empty());
        assert!(!dir.join("archiv/schafkopf-spiele-9.csv").exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
