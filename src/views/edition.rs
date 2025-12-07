use crate::{
    components::{view_edition, EditionId},
    i18n,
    views::Feedback,
};
use dioxus::prelude::*;

#[component]
pub fn Edition(id: EditionId) -> Element {
    let data = use_server_future(move || async move { view_edition(id).await })?;

    let lang = i18n::use_lang();

    rsx! {
        div {
            match &*data.read_unchecked() {
                Some(Ok(data)) => rsx! {
                    h3 { class: "text-2xl", "{data.edition.label()}" }
                    div { class: "edition-html",
                        style: "display: inline-block;",
                        dangerous_inner_html: "{data.html}",
                    }
                },
                Some(Err(e)) => rsx! { "{lang.read().error_loading_edition()}: {e}" },
                None => rsx! { "{lang.read().loading_edition()}" },
            }
            Feedback { edition_id: id }
        }
    }
}
