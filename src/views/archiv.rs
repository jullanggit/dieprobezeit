use crate::{components::fetch_editions, Route};
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Archiv() -> Element {
    let editions = use_resource(move || async move {
        fetch_editions().await.map(|mut editions| {
            editions.sort_by(|a, b| a.date.cmp(&b.date).reverse());
            editions
        })
    });

    rsx! {
        div {
            h1 { class: "text-4xl", "Archiv aller Ausgaben" }

            match &*editions.read_unchecked() {
                Some(Ok(editions)) => rsx! {
                    for edition in editions {
                        Link { to: Route::Edition { id: edition.id }, "{edition.label()}" }
                        br {}
                    }
                },
                Some(Err(e)) => rsx! { "Fehler beim laden des Archivs: {e}" },
                None => rsx! { "Lade Archiv..." },
            }
        }
    }
}
