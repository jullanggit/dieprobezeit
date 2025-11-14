use dioxus::prelude::*;

/// The Home page component that will be rendered when the current route is `[Route::Home]`
#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    rsx! {
        div {
            h1 {
                "Page "
                for segment in segments {
                    "{segment}/"
                }
                " not Found"
            }
        }
    }
}
