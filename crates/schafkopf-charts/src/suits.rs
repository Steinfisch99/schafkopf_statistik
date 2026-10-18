//! Symbole der vier Farben als SVG-Text: Eichel, Gras, Herz und Schellen.
//!
//! Jede Farbe hat eine eigene Form (Eichel mit Hütchen, Blatt, Herz, Schellen als Kugel mit gelbem, rotem und
//! grünem Feld wie auf dem Blatt), die Farbe allein trägt also keine Aussage und das Symbol bleibt auch bei Farbschwäche eindeutig. Jede Form hat einen
//! dunklen Rand, damit sie auf hellem und dunklem Grund zu sehen ist.

/// Eine der vier Farben des bayerischen Blatts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuitShape {
    Eichel,
    Gras,
    Herz,
    Schellen,
}

impl SuitShape {
    pub const ALL: [SuitShape; 4] = [
        SuitShape::Eichel,
        SuitShape::Gras,
        SuitShape::Herz,
        SuitShape::Schellen,
    ];

    /// Name der Farbe, auch als Beschreibung für Screenreader.
    pub fn label(self) -> &'static str {
        match self {
            SuitShape::Eichel => "Eichel",
            SuitShape::Gras => "Gras",
            SuitShape::Herz => "Herz",
            SuitShape::Schellen => "Schellen",
        }
    }

    /// Die Zeichnung ohne umgebendes `<svg>`, auf einer Fläche von 24 mal 24.
    fn shapes(self) -> &'static str {
        match self {
            SuitShape::Eichel => concat!(
                r##"<path d="M12 22c-3.6 0-5.8-3.2-5.8-6.8v-3.7h11.6v3.7C17.8 18.8 15.6 22 12 22z" "##,
                r##"fill="#D9A441" stroke="#7A5314" stroke-width="1" stroke-linejoin="round"/>"##,
                r##"<path d="M4.8 11.8C4.8 7.8 8 5.2 12 5.2s7.2 2.6 7.2 6.6z" "##,
                r##"fill="#6B4423" stroke="#3E2713" stroke-width="1" stroke-linejoin="round"/>"##,
                r##"<path d="M12 5.2V2.4" stroke="#3E2713" stroke-width="1.6" stroke-linecap="round"/>"##,
            ),
            SuitShape::Gras => concat!(
                r##"<path d="M4.5 19.5C4.5 10 9.5 4.2 20 4c0 10.4-5.6 15.5-15.5 15.5z" "##,
                r##"fill="#43A047" stroke="#1B5E20" stroke-width="1" stroke-linejoin="round"/>"##,
                r##"<path d="M4.5 19.5 15 9" stroke="#1B5E20" stroke-width="1.2" stroke-linecap="round"/>"##,
            ),
            SuitShape::Herz => concat!(
                r##"<path d="M12 21.2S3.4 15.9 3.4 9.9a4.6 4.6 0 0 1 8.6-2.3 4.6 4.6 0 0 1 8.6 2.3c0 6-8.6 11.3-8.6 11.3z" "##,
                r##"fill="#E53935" stroke="#8E1A17" stroke-width="1" stroke-linejoin="round"/>"##,
            ),
            SuitShape::Schellen => concat!(
                r##"<path d="M10.2 3.4a1.8 1.8 0 0 1 3.6 0z" fill="#F6C90E" stroke="#3A2A00" stroke-width="1" stroke-linejoin="round"/>"##,
                r##"<path d="M12 22.2c-1.3 0-2-.6-2-1.5h4c0 .9-.7 1.5-2 1.5z" fill="#2E8B3E" stroke="#1B5E20" stroke-width="1" stroke-linejoin="round"/>"##,
                r##"<path d="M3.2 9A9.5 9.5 0 0 1 20.8 9z" fill="#F6C90E"/>"##,
                r##"<path d="M3.2 9h17.6l.2 6.5H3z" fill="#D62828"/>"##,
                r##"<path d="M3 15.5h18A9.5 9.5 0 0 1 3 15.5z" fill="#2E8B3E"/>"##,
                r##"<circle cx="7" cy="12.25" r="1.7" fill="#7A1212"/>"##,
                r##"<circle cx="12" cy="12.25" r="1.7" fill="#7A1212"/>"##,
                r##"<circle cx="17" cy="12.25" r="1.7" fill="#7A1212"/>"##,
                r##"<path d="M12 20.4c0-2.2-2.4-3.6-5.4-3.4M12 20.4c0-2.2 2.4-3.6 5.4-3.4" fill="none" stroke="#1B5E20" stroke-width="1" stroke-linecap="round"/>"##,
                r##"<circle cx="12" cy="12.5" r="9.5" fill="none" stroke="#3A2A00" stroke-width="1.2"/>"##,
            ),
        }
    }
}

/// Das Symbol einer Farbe als `<svg>`. Es füllt die Fläche, in die man es setzt.
pub fn suit_icon_svg(suit: SuitShape) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 24 24\" role=\"img\" \
         aria-label=\"{label}\" style=\"width:100%;height:100%\">{shapes}</svg>",
        label = suit.label(),
        shapes = suit.shapes(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_suit_has_an_icon_with_its_name() {
        for suit in SuitShape::ALL {
            let svg = suit_icon_svg(suit);
            assert!(svg.starts_with("<svg") && svg.ends_with("</svg>"));
            assert!(svg.contains(&format!("aria-label=\"{}\"", suit.label())));
            assert!(svg.contains("<path"), "{suit:?} hat eine Form");
        }
    }

    #[test]
    fn the_four_icons_differ_in_shape_and_not_only_in_colour() {
        let shapes: Vec<&str> = SuitShape::ALL.iter().map(|s| s.shapes()).collect();
        for (i, a) in shapes.iter().enumerate() {
            for b in &shapes[i + 1..] {
                assert_ne!(a, b);
            }
        }
        // Die Pfaddaten (ohne Farben) sind pro Farbe verschieden.
        let paths = |s: &str| -> Vec<String> {
            s.split("d=\"")
                .skip(1)
                .map(|p| p.split('"').next().unwrap().to_owned())
                .collect()
        };
        let all: Vec<Vec<String>> = shapes.iter().map(|s| paths(s)).collect();
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn the_icons_are_well_formed() {
        for suit in SuitShape::ALL {
            let svg = suit_icon_svg(suit);
            let open = svg.matches('<').count();
            let close = svg.matches('>').count();
            assert_eq!(open, close, "{suit:?}: Klammern passen nicht");
            assert!(!svg.contains("NaN"));
            assert_eq!(svg.matches("<svg").count(), 1);
        }
    }
}
