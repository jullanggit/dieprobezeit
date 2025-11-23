use crate::{components::fetch_editions, Route};
use dioxus::prelude::*;

const ARCHIV_CSS: Asset = asset!("/assets/styling/archiv.css");
const RSS_ICON: Asset = asset!("/assets/rss.png");

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Archiv() -> Element {
    let editions = use_server_future(move || async move {
        fetch_editions().await.map(|mut editions| {
            editions.sort_by(|a, b| a.date.cmp(&b.date).reverse());
            editions
        })
    })?;

    rsx! {
        document::Link { rel: "stylesheet", href: ARCHIV_CSS }

        div { id: "archiv",
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

            div {id: "feed-link",
                style: "margin-top: 2em; text-align: center;",
                Link {
                    // use External so the Router doesn't hijack the link
                    to: NavigationTarget::<crate::Route>::External("/feed.xml".to_string()),
                    img {
                        src: RSS_ICON,
                        alt: "Atom Feed",
                        width: 30,
                        height: 30,
                    }
                }
            }
        }
    }
}
