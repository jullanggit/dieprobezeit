use crate::{components::fetch_editions, Edition};
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    let editions = use_resource(move || async move { fetch_editions().await });

    rsx! {
        div {
            h1 { "MNG Schüelerziitig" }
            a { "Willkomme zur monatliche MNG Schüelerziitg!" }

            h2 { "Neusti Usgab" }

            match &*editions.read_unchecked() {
                Some(Ok(editions)) => {
                    let newest = editions.iter().max_by_key(|element| element.date);
                    match newest {
                        None => rsx! { "Kei neusti usgab gfunde" },
                        Some(newest) => rsx! {
                            Edition { id: newest.id }
                        },
                    }
                }
                Some(Err(e)) => rsx! {
                "Fehler beim laden der Ausgaben: {e}"
                },
                None => rsx! { "Lade Ausgaben..." },
            }
        }
    }
}
