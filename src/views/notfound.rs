use crate::i18n;
use dioxus::prelude::*;

#[component]
pub fn NotFound(segments: Vec<String>) -> Element {
    let lang = i18n::use_lang();
    let not_found = lang.read().page_not_found();
    let (page, not_found) = {
        let first_space = not_found.find(' ').unwrap_or(5);
        // include space in both sub strings
        (&not_found[0..=first_space], &not_found[first_space..])
    };

    rsx! {
        div {
            h1 {
                "{page}"
                for segment in segments {
                    "{segment}/"
                }
                "{not_found}"
            }
        }
    }
}
