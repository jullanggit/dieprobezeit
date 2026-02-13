use crate::{
    components::{view_edition, EditionId},
    db::{db, entities::reads},
    i18n,
    track_views::{get_client_id, set_client_id, ClientId, NO_ID},
    views::Feedback,
};
use dioxus::prelude::*;
use sea_orm::{EntityTrait, Set};

#[post("/api/record-read-times")]
pub async fn record_read_times(
    edition_id: EditionId,
    page_times: Vec<f32>,
) -> Result<(), ServerFnError> {
    let db = db();

    let client_id = get_client_id().map_or(NO_ID, |client_id| client_id.0);

    let entities = page_times
        .iter()
        .enumerate()
        .filter(|(_, time)| **time != 0.)
        .map(|(page, time)| reads::ActiveModel {
            client_id: Set(client_id),
            edition_id: Set(edition_id),
            page_number: Set(page as i32),
            read_time: Set(*time),
            ..Default::default()
        });
    reads::Entity::insert_many(entities)
        .exec(db)
        .await
        .map_err(|err| ServerFnError::new(err.to_string()))?;

    Ok(())
}

#[component]
pub fn Edition(id: EditionId) -> Element {
    let data = use_server_future(move || async move { view_edition(id).await })?;

    let lang = i18n::use_lang();

    // TODO: actually make sure that this cookie is set.
    if get_client_id().is_none() {
        set_client_id();
    }

    rsx! {
        div {
            match &*data.read_unchecked() {
                Some(Ok(data)) => rsx! {
                    h3 { class: "text-2xl", "{data.label()}" }
                    div {
                        class: "pdfjs-container",
                        "data-pdf-src": "/pdfs/{data.date}.pdf",
                        "edition-id": "{id}",
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
