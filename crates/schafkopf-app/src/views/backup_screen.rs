//! Sicherung: Alle Daten als ZIP in den Ordner Downloads legen und eine solche ZIP wieder
//! einspielen, auf Wunsch nur einzelne Teile (Spieler, Regelwerke, Tarife, Abende).

use std::collections::BTreeSet;

use dioxus::document::eval;
use dioxus::prelude::*;
use schafkopf_core::{pack, read_backup, unpack, Backup, Library, Table};

use super::common::{persist_library, Chip, Notice};
use crate::navigation::back_target;
use crate::summary::games_text;
use crate::{export, now_secs, storage, Screen};

/// Legt das Datei-Feld direkt im `body` an, außerhalb von Dioxus: Dioxus fängt Klicks auf eigene
/// Datei-Felder ab und öffnet einen Dialog, den es auf dem Handy nicht gibt. Dieses Feld öffnet den
/// Dateiauswahl-Dialog der WebView selbst, das `label` in der Oberfläche löst es aus. Die gewählte
/// Datei geht als Base64-Text zurück, leer, wenn nichts gewählt ist oder das Lesen scheitert.
const FILE_INPUT_JS: &str = r#"
let input = document.getElementById('dx-backup-input');
if (!input) {
    input = document.createElement('input');
    input.type = 'file';
    input.id = 'dx-backup-input';
    input.accept = '.zip,application/zip';
    input.style.cssText = 'position:fixed;left:0;top:0;width:1px;height:1px;opacity:0;pointer-events:none';
    document.body.appendChild(input);
}
input.onchange = () => {
    const file = input.files && input.files[0];
    if (!file) {
        dioxus.send('');
        return;
    }
    file.arrayBuffer().then((buf) => {
        const bytes = new Uint8Array(buf);
        let text = '';
        for (let i = 0; i < bytes.length; i += 0x8000) {
            text += String.fromCharCode.apply(null, bytes.subarray(i, i + 0x8000));
        }
        dioxus.send(btoa(text));
    }).catch(() => dioxus.send(''));
};
"#;

/// Wandelt Base64 in Bytes. `None`, wenn der Text kein gültiges Base64 ist.
fn decode_base64(text: &str) -> Option<Vec<u8>> {
    let value = |c: u8| -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some(u32::from(c - b'A')),
            b'a'..=b'z' => Some(u32::from(c - b'a') + 26),
            b'0'..=b'9' => Some(u32::from(c - b'0') + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    };
    let digits: Vec<u8> = text
        .bytes()
        .filter(|c| !c.is_ascii_whitespace() && *c != b'=')
        .collect();
    let mut out = Vec::with_capacity(digits.len() * 3 / 4);
    for chunk in digits.chunks(4) {
        if chunk.len() == 1 {
            return None;
        }
        let mut acc = 0u32;
        for (i, &c) in chunk.iter().enumerate() {
            acc |= value(c)? << (18 - 6 * i);
        }
        out.push((acc >> 16) as u8);
        if chunk.len() > 2 {
            out.push((acc >> 8) as u8);
        }
        if chunk.len() > 3 {
            out.push(acc as u8);
        }
    }
    Some(out)
}

/// Eine gelesene Sicherung, bereit zum Einspielen.
#[derive(Clone, PartialEq)]
struct Loaded {
    backup: Backup,
    /// Zeitpunkte der früheren Abende, die es hier noch nicht gibt.
    new_past: Vec<u64>,
}

#[component]
pub fn BackupScreen(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
    library: Signal<Library>,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;
    let mut library = library;
    let mut loaded = use_signal(|| None::<Loaded>);
    let mut take_players = use_signal(|| true);
    let mut take_rules = use_signal(|| true);
    let mut take_tariffs = use_signal(|| true);
    let mut take_past = use_signal(|| true);
    let mut take_current = use_signal(|| true);

    let back = back_target(Screen::Backup, table.read().is_some()).unwrap_or(Screen::Setup);
    let past_count = storage::list_past().len();
    let running = table.read().is_some();
    let data_dir = storage::data_dir().display().to_string();
    let (players, rule_sets, tariffs) = library.read().counts();

    let create = move |_| {
        let result = storage::backup_files(now_secs())
            .map_err(|e| format!("Daten lesen: {e}"))
            .and_then(|files| pack(&files).map_err(|e| e.to_string()))
            .and_then(|zip| {
                export::save_file(
                    &export::backup_file_name(now_secs()),
                    "application/zip",
                    &zip,
                )
            });
        notice.set(Some(match result {
            Ok(place) => Notice::ok(format!("Sicherung gespeichert: {place}")),
            Err(e) => Notice::error(format!("Sicherung fehlgeschlagen: {e}")),
        }));
    };

    use_future(move || async move {
        let mut reader = eval(FILE_INPUT_JS);
        while let Ok(text) = reader.recv::<String>().await {
            if text.is_empty() {
                loaded.set(None);
                continue;
            }
            let backup = decode_base64(&text)
                .ok_or_else(|| "Die Datei ließ sich nicht lesen".to_owned())
                .and_then(|bytes| unpack(&bytes).map_err(|e| e.to_string()))
                .map(|files| read_backup(&files));
            match backup {
                Ok(b) if b.library.is_none() && b.current.is_none() && b.past.is_empty() => {
                    loaded.set(None);
                    notice.set(Some(Notice::error(
                        "In dieser Datei steckt keine Sicherung der Schafkopf-Statistik",
                    )));
                }
                Ok(b) => {
                    let have: BTreeSet<u64> =
                        storage::list_past().iter().map(|p| p.stamp).collect();
                    let new_past = b
                        .past
                        .iter()
                        .map(|(stamp, _)| *stamp)
                        .filter(|stamp| !have.contains(stamp))
                        .collect();
                    loaded.set(Some(Loaded {
                        backup: b,
                        new_past,
                    }));
                }
                Err(e) => {
                    loaded.set(None);
                    notice.set(Some(Notice::error(e)));
                }
            }
        }
    });
    let import = move |_| {
        let Some(l) = loaded.read().clone() else {
            return;
        };
        let mut parts: Vec<String> = Vec::new();
        let mut problems: Vec<String> = Vec::new();

        if let Some(theirs) = &l.backup.library {
            if take_players() || take_rules() || take_tariffs() {
                let report = library.write().merge_from(
                    theirs,
                    take_players(),
                    take_rules(),
                    take_tariffs(),
                );
                match persist_library(&library.read()) {
                    Ok(()) => parts.push(format!(
                        "{} Spieler, {} Regelwerke, {} Tarife",
                        report.players, report.rule_sets, report.tariffs
                    )),
                    Err(e) => problems.push(e),
                }
            }
        }
        if take_past() {
            let mut added = 0;
            for (stamp, past) in &l.backup.past {
                if !l.new_past.contains(stamp) {
                    continue;
                }
                match storage::import_past(*stamp, past) {
                    Ok(true) => added += 1,
                    Ok(false) => {}
                    Err(e) => problems.push(format!("Abend speichern: {e}")),
                }
            }
            if !l.backup.past.is_empty() {
                parts.push(format!("{} frühere Abende", added));
            }
        }
        if take_current() && !running {
            if let Some(current) = &l.backup.current {
                match storage::save(current, None) {
                    Ok(_) => {
                        table.set(Some(current.clone()));
                        parts.push("der laufende Abend".to_owned());
                    }
                    Err(e) => problems.push(format!("Abend speichern: {e}")),
                }
            }
        }
        loaded.set(None);
        let _ =
            eval("const i = document.getElementById('dx-backup-input'); if (i) { i.value = ''; }");
        notice.set(Some(if problems.is_empty() {
            if parts.is_empty() {
                Notice::ok("Nichts eingespielt, es war nichts Neues ausgewählt")
            } else {
                Notice::ok(format!("Eingespielt: {}", parts.join(", ")))
            }
        } else {
            Notice::error(format!("Nicht alles ging: {}", problems.join("; ")))
        }));
    };

    let place = if cfg!(target_os = "android") {
        "im Ordner Downloads deines Handys"
    } else {
        "in den Ordner Downloads (sonst in den Datenordner der App)"
    };
    let storage_note = if cfg!(target_os = "android") {
        "Die App speichert alles im privaten Speicher der App. Du kommst mit dem Dateimanager nicht dran, und beim Deinstallieren der App oder einem neuen Handy ist es weg. Mach deshalb ab und zu eine Sicherung."
    } else {
        "Das ist der Datenordner der App auf diesem Gerät."
    };

    rsx! {
        h2 { "Sicherung" }
        button { class: "secondary", r#type: "button", onclick: move |_| screen.set(back), "← Zurück" }

        section {
            h3 { "Wo liegen die Daten?" }
            p { class: "hint", "{storage_note}" }
            p { class: "muted small", "Ordner: {data_dir}" }
            p { class: "muted small",
                "Jetzt vorhanden: {players} Spieler, {rule_sets} Regelwerke, {tariffs} Tarife, {past_count} frühere Abende"
                if running { ", dazu der laufende Abend." } else { "." }
            }
        }

        section {
            h3 { "Sicherung erstellen" }
            p { class: "hint",
                "Packt alles in eine ZIP-Datei {place}: den laufenden Abend, alle früheren Abende mit Spielen, Statistiken und Protokollen, dazu Spieler, Regelwerke und Tarife. Die CSV-Tabellen darin lassen sich auch direkt in Excel öffnen."
            }
            div { class: "actions",
                button { class: "primary big", r#type: "button", onclick: create, "Sicherung in Downloads speichern" }
            }
        }

        section {
            h3 { "Sicherung einspielen" }
            p { class: "hint",
                "Wähle eine ZIP-Datei, die du mit \"Sicherung erstellen\" gemacht hast. Danach suchst du aus, was übernommen wird. Was es hier schon gibt, bleibt unverändert, es kommt nur Neues dazu."
            }
            label { class: "pick-file", r#for: "dx-backup-input", "ZIP-Datei wählen" }

            if let Some(l) = loaded() {
                {
                    let (p, r, t) = l.backup.library.as_ref().map_or((0, 0, 0), |lib| lib.counts());
                    let new_count = l.new_past.len();
                    let total_past = l.backup.past.len();
                    let current_games = l.backup.current.as_ref().map(|c| c.games().len());
                    rsx! {
                        h4 { "Was in der Datei steckt" }
                        if l.backup.library.is_some() {
                            p { class: "muted small", "{p} Spieler, {r} Regelwerke, {t} Tarife" }
                            div { class: "chips",
                                Chip { label: "Spieler".to_string(), on: take_players(), onclick: move |_| take_players.set(!take_players()) }
                                Chip { label: "Regelwerke (Spielarten, Bock)".to_string(), on: take_rules(), onclick: move |_| take_rules.set(!take_rules()) }
                                Chip { label: "Tarife".to_string(), on: take_tariffs(), onclick: move |_| take_tariffs.set(!take_tariffs()) }
                            }
                        }
                        if total_past > 0 {
                            p { class: "muted small", "{total_past} frühere Abende, davon {new_count} neu hier" }
                            div { class: "chips",
                                Chip { label: format!("Frühere Abende ({new_count} neue)"), on: take_past(), onclick: move |_| take_past.set(!take_past()) }
                            }
                        }
                        if let Some(games) = current_games {
                            p { class: "muted small", "Laufender Abend mit {games_text(games)}" }
                            div { class: "chips",
                                Chip {
                                    label: "Laufenden Abend übernehmen".to_string(),
                                    on: take_current() && !running,
                                    disabled: running,
                                    onclick: move |_| take_current.set(!take_current()),
                                }
                            }
                            if running {
                                p { class: "hint", "Hier läuft schon ein Abend. Beende ihn zuerst, wenn du den aus der Sicherung übernehmen willst." }
                            }
                        }
                        if !l.backup.unreadable.is_empty() {
                            p { class: "hint", "{l.backup.unreadable.len()} Dateien in der ZIP ließen sich nicht lesen und werden übersprungen." }
                        }
                        div { class: "actions",
                            button { class: "primary", r#type: "button", onclick: import, "Auswahl einspielen" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_base64_like_the_browser_sends_it() {
        assert_eq!(decode_base64("").unwrap(), b"");
        assert_eq!(decode_base64("Zg==").unwrap(), b"f");
        assert_eq!(decode_base64("Zm8=").unwrap(), b"fo");
        assert_eq!(decode_base64("Zm9v").unwrap(), b"foo");
        assert_eq!(decode_base64("Zm9vYg==").unwrap(), b"foob");
        assert_eq!(decode_base64("Zm9v\nYmFy").unwrap(), b"foobar");
        assert_eq!(decode_base64("UEs=").unwrap(), b"PK");
        assert!(decode_base64("Zm9v!").is_none());
        assert!(decode_base64("Z").is_none());
    }
}
