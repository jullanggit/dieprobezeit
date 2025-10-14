use dioxus::prelude::*;

use crate::components::{fetch_edition_data, EditionId};

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Edition(id: EditionId) -> Element {
    let data = use_resource(move || async move { fetch_edition_data(id).await });

    rsx! {
        div {
            match &*data.read_unchecked() {
                Some(Ok(_data)) => rsx! { "TODO" },
                Some(Err(e)) => rsx! {
                "Fehler beim laden der Ausgabe: {e}"
                },
                None => rsx! { "Lade Ausgabe..." },
            }
        }
    }
}
