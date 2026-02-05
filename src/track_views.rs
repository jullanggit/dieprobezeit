use crate::{
    components::EditionId,
    cookies::{get_or_insert_cookie, set_cookie},
};
use std::collections::HashMap;
use uuid::{uuid, Uuid};

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
}

type EditionProgress = f32;
pub struct EditionProgresses(HashMap<EditionId, EditionProgress>);

const STORAGE_KEY: &str = "client_id";
pub fn set_client_id(client_id: ClientId) {
    set_cookie(STORAGE_KEY, &client_id.0.to_string());
}
pub fn get_client_id(client_id: ClientId) {
    get_or_insert_cookie(STORAGE_KEY, DEFAULT_LANG.to_str(), Language::from_str)
}
