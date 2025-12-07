use crate::{i18n, Route};
use dioxus::prelude::*;

const NAVBAR_CSS: Asset = asset!("/assets/styling/navbar.css");

/// The Navbar component that will be rendered on all pages of our app since every page is under the layout.
///
///
/// This layout component wraps the UI of [Route::Home] and [Route::Blog] in a common navbar. The contents of the Home and Blog
/// routes will be rendered under the outlet inside this component
#[component]
pub fn Navbar() -> Element {
    let mut lang = i18n::use_lang();

    rsx! {
        document::Link { rel: "stylesheet", href: NAVBAR_CSS }

        div { id: "navbar",
            div {id: "navbar-links"
            Link { to: Route::Home {}, "Home" }
            Link { to: Route::Archiv {}, "{lang.read().archive()}" }
            Link { to: Route::Feedback {}, "Feedback" }
            }

            div { id: "language-selector",
                select {
                    value: "{lang.read().to_str()}",
                    onchange: move |event| {
                        let value = event.value();
                        if let Some(language) = i18n::Language::from_str(&value) {
                            i18n::set_lang(language);
                            lang.set(language);
                        }
                    },

                    for variant in i18n::Language::variants() {
                        option {
                            value: "{variant.to_str()}",
                            "{variant.display_name()}"
                        }
                    }
                }
            }
        }


        // The `Outlet` component is used to render the next component inside the layout. In this case, it will render either
        // the [`Home`] or [`Blog`] component depending on the current route.
        Outlet::<Route> {}
    }
}
