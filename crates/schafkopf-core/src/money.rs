use crate::model::Cents;

/// Liest einen Euro-Betrag aus einer Eingabe wie `0,20`, `0.2`, `1` oder `2,5 €` in Cent.
///
/// Komma und Punkt gelten als Dezimaltrenner, höchstens zwei Nachkommastellen. Leere Eingaben,
/// negative Beträge und alles andere ergeben `None`.
pub fn parse_euro(text: &str) -> Option<Cents> {
    let text = text.trim().trim_end_matches('€').trim();
    let (euros, cents) = match text.split_once([',', '.']) {
        Some((euros, cents)) => (euros, cents),
        None => (text, ""),
    };
    if euros.is_empty() && cents.is_empty() {
        return None;
    }
    let all_digits = |s: &str| s.chars().all(|c| c.is_ascii_digit());
    if !all_digits(euros) || !all_digits(cents) || cents.len() > 2 || euros.len() > 9 {
        return None;
    }
    let euros: Cents = if euros.is_empty() {
        0
    } else {
        euros.parse().ok()?
    };
    let cents: Cents = match cents.len() {
        0 => 0,
        1 => cents.parse::<Cents>().ok()? * 10,
        _ => cents.parse().ok()?,
    };
    Some(euros * 100 + cents)
}

#[cfg(test)]
mod tests {
    use super::parse_euro;

    #[test]
    fn parses_common_inputs() {
        assert_eq!(parse_euro("0,20"), Some(20));
        assert_eq!(parse_euro("0.2"), Some(20));
        assert_eq!(parse_euro("1"), Some(100));
        assert_eq!(parse_euro("2,5 €"), Some(250));
        assert_eq!(parse_euro(" 12,05 "), Some(1205));
        assert_eq!(parse_euro(",50"), Some(50));
        assert_eq!(parse_euro("0"), Some(0));
    }

    #[test]
    fn rejects_everything_else() {
        for bad in [
            "",
            " ",
            "€",
            "-1",
            "1,234",
            "abc",
            "1,2,3",
            "1 2",
            "+1",
            "1e3",
            "9999999999",
        ] {
            assert_eq!(parse_euro(bad), None, "{bad:?}");
        }
    }
}
