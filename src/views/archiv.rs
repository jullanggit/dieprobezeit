use crate::{components::fetch_editions, i18n, Route};
use dioxus::prelude::*;

const ARCHIV_CSS: Asset = asset!("/assets/styling/archiv.css");
const RSS_ICON: Asset = asset!("/assets/rss.png");

#[component]
pub fn Archiv() -> Element {
    let editions = use_server_future(move || async move {
        fetch_editions().await.map(|mut editions| {
            editions.sort_by(|a, b| a.date.cmp(&b.date).reverse());
            editions
        })
    })?;

    let lang = i18n::get_lang();

    rsx! {
        document::Link { rel: "stylesheet", href: ARCHIV_CSS }

        div { id: "archiv",
            h1 { class: "text-4xl", "{lang.archive_title()}" }

            match &*editions.read_unchecked() {
                Some(Ok(editions)) => rsx! {
                    for edition in editions {
                        Link { to: Route::Edition { id: edition.id }, "{edition.label()}" }
                        br {}
                    }
                },
                Some(Err(e)) => rsx! { "{lang.error_loading_archive()}: {e}" },
                None => rsx! { "{lang.loading_archive()}" },
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
