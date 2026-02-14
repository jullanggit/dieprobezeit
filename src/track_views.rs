use crate::components::EditionId;
use crate::cookies::{get_cookie, get_or_insert_cookie};
use dioxus::prelude::*;
use sea_orm::EntityTrait;
use sea_orm::Set;
use std::str::FromStr;
use uuid::Uuid;

pub const NO_ID: Uuid = Uuid::nil();

/// A Client UUID used for view deduplication
pub struct ClientId(pub Uuid);
impl ClientId {
    /// Creates a new random Client ID. May fail if window or crypto couldn't be accessed.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
    pub fn from_str(str: &str) -> Option<Self> {
        Some(Self(Uuid::from_str(str).ok()?))
    }
}

const STORAGE_KEY: &str = "client_id";
pub fn ensure_client_id_set() {
    get_or_insert_cookie(
        STORAGE_KEY,
        || ClientId::new().to_string(),
        ClientId::from_str,
    );
}
pub fn get_client_id() -> Option<ClientId> {
    get_cookie(STORAGE_KEY, ClientId::from_str)
}

#[post("/api/record-read-times")]
pub async fn record_read_times(
    edition_id: EditionId,
    page_times: Vec<f32>,
) -> Result<(), ServerFnError> {
    use crate::db::{db, entities::reads};

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
