use dioxus::prelude::*;

use crate::components::{view_edition, EditionId};

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Edition(id: EditionId) -> Element {
    let data = use_resource(move || async move { view_edition(id).await });

    rsx! {
        div {
            match &*data.read_unchecked() {
                Some(Ok(_data)) => rsx! {
                    "TODO id: {id}, views: {_data.views}"
                },
                Some(Err(e)) => rsx! {
                "Fehler beim laden der Ausgabe: {e}"
                },
                None => rsx! { "Lade Ausgabe..." },
            }
        }
    }
}
