//! Das Symbol einer Farbe (Eichel, Gras, Herz, Schellen) neben dem Namen.

use dioxus::prelude::*;
use schafkopf_charts::{suit_icon_svg, SuitShape};
use schafkopf_core::Suit;

/// Die Form zu einer Farbe der Spiellogik.
pub fn shape_of(suit: Suit) -> SuitShape {
    match suit {
        Suit::Eichel => SuitShape::Eichel,
        Suit::Gras => SuitShape::Gras,
        Suit::Herz => SuitShape::Herz,
        Suit::Schellen => SuitShape::Schellen,
    }
}

/// Ein kleines Symbol in der Höhe der Schrift.
#[component]
pub fn SuitIcon(suit: Suit) -> Element {
    let svg = suit_icon_svg(shape_of(suit));
    rsx! {
        span { class: "suit-icon", dangerous_inner_html: "{svg}" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_suit_maps_to_the_shape_with_the_same_name() {
        for suit in Suit::ALL {
            assert_eq!(shape_of(suit).label(), suit.label());
        }
    }
}
