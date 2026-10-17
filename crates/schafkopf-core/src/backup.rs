//! Sicherung aller Daten als ZIP: packen, auspacken und lesen, was darin liegt. Die App liest und
//! schreibt die Dateien selbst, hier geht es nur um die Bytes, damit alles ohne Handy testbar ist.

use std::io::{Cursor, Read, Write};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::{Error, Result};
use crate::library::Library;
use crate::table::Table;

/// So viel darf eine Sicherung ausgepackt höchstens groß sein, gegen kaputte oder böswillige Dateien.
pub const MAX_UNPACKED_BYTES: u64 = 64 * 1024 * 1024;

/// Namen in der ZIP, wie im Datenordner der App.
pub const LIBRARY_FILE: &str = "schafkopf-bibliothek.json";
pub const CURRENT_FILE: &str = "schafkopf-tisch.json";
const PAST_PREFIX: &str = "schafkopf-tisch-";

fn backup_error(e: impl std::fmt::Display) -> Error {
    Error::Backup(e.to_string())
}

/// Packt Dateien (Name mit `/` als Trenner, Inhalt) in eine ZIP.
pub fn pack(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>> {
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, data) in files {
        zip.start_file(name.as_str(), options)
            .map_err(backup_error)?;
        zip.write_all(data).map_err(backup_error)?;
    }
    Ok(zip.finish().map_err(backup_error)?.into_inner())
}

/// Ob der Name einer Datei in der ZIP unbedenklich ist: relativ, ohne `..`, ohne Laufwerk.
fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('/')
        && !name.contains('\\')
        && !name.contains(':')
        && !name.split('/').any(|part| part == "..")
}

/// Packt eine ZIP aus. Ordner, unsichere Namen und alles jenseits der Größengrenze werden nicht
/// übernommen, bei zu viel Inhalt ist es ein Fehler.
pub fn unpack(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    let mut archive = ZipArchive::new(Cursor::new(bytes))
        .map_err(|_| Error::Backup("Das ist keine ZIP-Datei".into()))?;
    let mut total: u64 = 0;
    let mut files = Vec::new();
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(backup_error)?;
        let Ok(name) = entry.name().map(|n| n.into_owned()) else {
            continue;
        };
        if entry.is_dir() || !safe_name(&name) {
            continue;
        }
        let mut data = Vec::new();
        // Gelesen wird nie mehr als die Grenze erlaubt, egal was die ZIP über ihre Größe sagt.
        let room = MAX_UNPACKED_BYTES.saturating_sub(total) + 1;
        entry
            .take(room)
            .read_to_end(&mut data)
            .map_err(backup_error)?;
        total += data.len() as u64;
        if total > MAX_UNPACKED_BYTES {
            return Err(Error::Backup("Die Sicherung ist zu groß".into()));
        }
        files.push((name, data));
    }
    Ok(files)
}

/// Was eine Sicherung enthält, soweit es sich lesen ließ.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Backup {
    pub library: Option<Library>,
    /// Der Abend, der bei der Sicherung gerade lief.
    pub current: Option<Table>,
    /// Frühere Abende mit ihrem Zeitpunkt im Archiv.
    pub past: Vec<(u64, Table)>,
    /// Dateien, die zu keinem Teil passen oder sich nicht lesen ließen.
    pub unreadable: Vec<String>,
}

/// Liest die Teile einer Sicherung aus den ausgepackten Dateien. Es zählt nur der letzte Teil des
/// Namens, so geht es auch, wenn jemand die ZIP in einem Ordner neu gepackt hat.
pub fn read_backup(files: &[(String, Vec<u8>)]) -> Backup {
    let mut backup = Backup::default();
    for (name, data) in files {
        let base = name.rsplit('/').next().unwrap_or(name);
        let Ok(text) = std::str::from_utf8(data) else {
            continue;
        };
        if base == LIBRARY_FILE {
            match Library::from_json(text) {
                Ok(l) => backup.library = Some(l),
                Err(_) => backup.unreadable.push(name.clone()),
            }
        } else if base == CURRENT_FILE {
            match Table::from_json(text) {
                Ok(t) => backup.current = Some(t),
                Err(_) => backup.unreadable.push(name.clone()),
            }
        } else if let Some(stamp) = base
            .strip_prefix(PAST_PREFIX)
            .and_then(|s| s.strip_suffix(".json"))
            .and_then(|s| s.parse::<u64>().ok())
        {
            match Table::from_json(text) {
                Ok(t) => backup.past.push((stamp, t)),
                Err(_) => backup.unreadable.push(name.clone()),
            }
        }
    }
    backup.past.sort_by_key(|(stamp, _)| *stamp);
    backup
}
