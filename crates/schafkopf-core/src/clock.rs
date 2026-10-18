//! Uhrzeiten aus Unix-Sekunden, ohne Zeitzonen-Bibliothek: Die App gibt die Abweichung von UTC mit.

/// Jahr, Monat, Tag, Stunde und Minute für einen Zeitpunkt in Unix-Sekunden, in der Zeit mit
/// `offset_min` Minuten Abstand zu UTC (Deutschland im Sommer: 120).
pub fn civil(unix_secs: i64, offset_min: i32) -> (i64, u32, u32, u32, u32) {
    let local = unix_secs + i64::from(offset_min) * 60;
    let days = local.div_euclid(86_400);
    let secs = local.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let year = yoe + era * 400 + i64::from(month <= 2);
    (
        year,
        month,
        day,
        (secs / 3_600) as u32,
        (secs % 3_600 / 60) as u32,
    )
}

/// "2026-10-17 19:30" in der Ortszeit.
pub fn format_local(unix_secs: i64, offset_min: i32) -> String {
    let (y, mo, d, h, mi) = civil(unix_secs, offset_min);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}")
}

/// "19:30" in der Ortszeit.
pub fn format_clock(unix_secs: i64, offset_min: i32) -> String {
    let (_, _, _, h, mi) = civil(unix_secs, offset_min);
    format!("{h:02}:{mi:02}")
}

/// "3 h 40 min" oder "25 min".
pub fn format_duration(secs: i64) -> String {
    let minutes = secs.max(0) / 60;
    if minutes >= 60 {
        format!("{} h {:02} min", minutes / 60, minutes % 60)
    } else {
        format!("{minutes} min")
    }
}

/// Minuten seit 12:00 Uhr mittags in der Ortszeit (0 bis 1439). So liegt 01:00 Uhr nach 23:00 Uhr, und
/// sich Uhrzeiten eines Abends lassen sich mitteln, auch wenn sie über Mitternacht gehen.
pub fn minutes_after_noon(unix_secs: i64, offset_min: i32) -> i32 {
    let (_, _, _, h, mi) = civil(unix_secs, offset_min);
    (h as i32 * 60 + mi as i32 - 720).rem_euclid(1_440)
}

/// "21:15" für Minuten seit 12:00 Uhr, siehe [`minutes_after_noon`].
pub fn format_minutes_after_noon(minutes: i32) -> String {
    let m = (minutes + 720).rem_euclid(1_440);
    format!("{:02}:{:02}", m / 60, m % 60)
}
