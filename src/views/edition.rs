use crate::{
    components::{view_edition, EditionId},
    i18n,
    views::Feedback,
};
use dioxus::prelude::*;

#[component]
pub fn Edition(id: EditionId) -> Element {
    let data = use_server_future(move || async move { view_edition(id).await })?;

    let lang = i18n::get_lang();

    rsx! {
        div {
            match &*data.read_unchecked() {
                Some(Ok(data)) => rsx! {
                    h3 { class: "text-2xl", "{data.label()}" }
                    div { style: "display: inline-block;",
                        img { src: "/svgs/{data.date}.svg", height: "100%", width: "auto" }
                    }
                },
                Some(Err(e)) => rsx! { "{lang.error_loading_edition()}: {e}" },
                None => rsx! { "{lang.loading_edition()}" },
            }
            Feedback { edition_id: id }
        }
    }
}
