use crate::components::fetch_editions;
use atom_syndication::{EntryBuilder, FeedBuilder, FixedDateTime, LinkBuilder, Person};
use dioxus::{
    fullstack::response::{IntoResponse, Response},
    prelude::*,
    server::http::header,
};

const URL: &str = "https://dieprobezeit.ch";

#[get("/feed.xml")]
async fn atom_feed() -> Result<Response> {
    let author = Person {
        name: "Die Probe Zeit Team".into(),
        // TODO: email
        email: None,
        uri: None,
    };

    let editions = fetch_editions().await?;

    let time_to_chrono = |date| FixedDateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", date));

    let newest = editions
        .iter()
        .map(|model| model.date)
        .max()
        .map_or_else(|| Ok(FixedDateTime::default()), time_to_chrono)?;

    let entries = editions
        .into_iter()
        .map(|model| {
            // avoid confusig rust with double ? later on
            let updated = match time_to_chrono(model.date) {
                Ok(u) => u,
                Err(e) => return Err(e),
            };
            Ok(EntryBuilder::default()
                .title(model.label())
                .id(model.id.to_string())
                .link(
                    LinkBuilder::default()
                        .href(format!("{URL}/editions/{}", model.id))
                        .build(),
                )
                .updated(updated)
                .author(author.clone())
                .build())
        })
        .collect::<Vec<_>>();

    let mut feed = FeedBuilder::default();
    feed.title("Die Probe Zeit")
        .id("urn:uuid:dieprobezeit")
        .updated(newest)
        // TODO: add
        //  - icon
        //  - logo
        //  - website link
        .author(author);

    for entry in entries {
        feed.entry(entry?);
    }

    Ok((
        [(header::CONTENT_TYPE, "application/atom+xml; charset=utf-8")],
        feed.build().to_string(),
    )
        .into_response())
}
