use std::collections::HashMap;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

pub type EditionId = u64;

#[derive(Deserialize, Serialize)]
pub struct EditionMetaData {
    pub id: EditionId,
    pub date: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct EditionData {}

// Server functions let us define public APIs on the server that can be called like a normal async function from the client.
// Each server function needs to be annotated with the `#[server]` attribute, accept and return serializable types, and return
// a `Result` with the error type [`ServerFnError`].
//
// When the server function is called from the client, it will just serialize the arguments, call the API, and deserialize the
// response.
#[server]
pub async fn fetch_editions_metadata() -> Result<Vec<EditionMetaData>, ServerFnError> {
    Ok(vec![
        EditionMetaData {
            id: 0,
            date: "01.01.2001".to_string(),
        },
        EditionMetaData {
            id: 1,
            date: "30.05.2025".to_string(),
        },
    ])
}

#[server]
pub async fn fetch_edition_data(id: EditionId) -> Result<EditionData, ServerFnError> {
    let editions: HashMap<EditionId, EditionData> =
        HashMap::from_iter([(0, EditionData {}), (1, EditionData {})]);

    editions
        .get(&id)
        .ok_or(ServerFnError::ServerError(format!(
            "Edition with id {id} not found."
        )))
        .map(Clone::clone)
}
