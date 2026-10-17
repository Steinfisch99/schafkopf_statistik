//! Tarif während des Abends ändern.

use dioxus::prelude::*;
use schafkopf_core::{Table, Tariff};

use super::common::{Notice, TariffFields};
use crate::draft::TariffDraft;
use crate::{storage, Screen};

#[component]
pub fn TariffScreen(
    table: Signal<Option<Table>>,
    screen: Signal<Screen>,
    notice: Signal<Option<Notice>>,
) -> Element {
    let mut table = table;
    let mut screen = screen;
    let mut notice = notice;
    let draft = use_signal(|| {
        let tariff = table
            .read()
            .as_ref()
            .map_or_else(Tariff::default, |t| t.tariff.clone());
        TariffDraft::from_tariff(&tariff)
    });

    let save = move |_| {
        let tariff = match draft.read().to_tariff() {
            Ok(t) => t,
            Err(e) => return notice.set(Some(Notice::error(e))),
        };
        let mut guard = table.write();
        let Some(t) = guard.as_mut() else { return };
        t.set_tariff(tariff);
        let saved = storage::save(t, None);
        drop(guard);
        match saved {
            Ok(_) => notice.set(Some(Notice::ok(
                "Tarif gespeichert, er gilt ab dem nächsten Spiel",
            ))),
            Err(e) => notice.set(Some(Notice::error(format!(
                "Speichern fehlgeschlagen: {e}"
            )))),
        }
        screen.set(Screen::Table);
    };

    rsx! {
        h2 { "Tarif ändern" }
        p { class: "hint", "Gilt ab dem nächsten Spiel. Schon eingetragene Spiele behalten ihre Beträge." }
        TariffFields { draft }
        div { class: "actions",
            button { class: "primary big", r#type: "button", onclick: save, "Speichern" }
            button { class: "secondary", r#type: "button", onclick: move |_| screen.set(Screen::Table), "Abbrechen" }
        }
    }
}
