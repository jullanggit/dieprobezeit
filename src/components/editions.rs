use crate::db::{db, entities::edition};
use dioxus::prelude::*;
use sea_orm::EntityTrait;
use std::fmt::Display;

pub type EditionId = i32;

// Server functions let us define public APIs on the server that can be called like a normal async function from the client.
// Each server function needs to be annotated with the `#[server]` attribute, accept and return serializable types, and return
// a `Result` with the error type [`ServerFnError`].
//
// When the server function is called from the client, it will just serialize the arguments, call the API, and deserialize the
// response.
#[server]
pub async fn fetch_editions() -> Result<Vec<edition::Model>, ServerFnError> {
    edition::Entity::find()
        .all(db())
        .await
        .map_err(|err| ServerFnError::ServerError(format!("{err}")))
}
