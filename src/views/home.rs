use crate::{components::fetch_editions, i18n, Edition};
use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    let editions = use_server_future(move || async move { fetch_editions().await })?;

    let lang = i18n::get_lang();

    rsx! {
        div {
            h1 { class: "text-4xl", "Die Probe Zeit" }
            a { "{lang.welcome()}" }

            h2 { class: "text-3xl", "{lang.newest_edition()}" }

            match &*editions.read_unchecked() {
                Some(Ok(editions)) => {
                    let newest = editions.iter().max_by_key(|element| element.date);
                    match newest {
                        None => rsx! { "{lang.no_edition_found()}" },
                        Some(newest) => rsx! {
                            Edition { id: newest.id }
                        },
                    }
                }
                Some(Err(e)) => rsx! { "{lang.error_loading_editions()}: {e}" },
                None => rsx! { "{lang.loading_editions()}" },
            }
        }
    }
}
