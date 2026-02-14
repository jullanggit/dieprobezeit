use crate::{
    components::{EditionId, view_edition},
    i18n,
    track_views::ensure_client_id_set,
    views::Feedback,
};
use dioxus::prelude::*;

#[component]
pub fn Edition(id: EditionId) -> Element {
    let data = use_server_future(move || async move { view_edition(id).await })?;
    use_hook(ensure_client_id_set);

    let lang = i18n::use_lang();

    rsx! {
        div {
            match &*data.read_unchecked() {
                Some(Ok(data)) => rsx! {
                    h3 { class: "text-2xl", "{data.label()}" }
                    div {
                        class: "pdfjs-container",
                        "data-pdf-src": "/pdfs/{data.date}.pdf",
                        "data-edition-id": "{id}",
                        "{lang.read().loading_pdf()}"
                    }
                },
                Some(Err(e)) => rsx! { "{lang.read().error_loading_edition()}: {e}" },
                None => rsx! { "{lang.read().loading_edition()}" },
            }
            Feedback { edition_id: id }
        }
    }
}
