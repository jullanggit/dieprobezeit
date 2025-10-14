use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn Home() -> Element {
    rsx! {
        div {
            h1 { "MNG Schüelerziitig" }
            a { "Willkomme zur Monatliche MNG Schüelerziitg!" }
        }
    }
}
