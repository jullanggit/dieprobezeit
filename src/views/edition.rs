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
                    for i in 1..=data.num_pages {
                        div {id: "edition-page-{i}",
                            style: "display: inline-block;",
                            object {
                                data: "/svgs/{data.edition.date}/-{i}.svg",
                                height: "100%",
                                width: "auto",
                            }
                        }
                    }
                },
                Some(Err(e)) => rsx! { "{lang.read().error_loading_edition()}: {e}" },
                None => rsx! { "{lang.read().loading_edition()}" },
            }
            Feedback { edition_id: id }
        }
    }
}
