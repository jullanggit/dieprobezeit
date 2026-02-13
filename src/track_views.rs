use crate::{
    components::EditionId,
    cookies::{get_cookie, get_or_insert_cookie, set_cookie},
};
use std::{collections::HashMap, str::FromStr};
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
    pub fn from_str(str: &str) -> Option<Self> {
        Some(Self(Uuid::from_str(str).ok()?))
    }
}

type EditionProgress = f32;
pub struct EditionProgresses(HashMap<EditionId, EditionProgress>);

const STORAGE_KEY: &str = "client_id";
pub fn set_client_id() {
    set_cookie(STORAGE_KEY, &ClientId::new().0.to_string());
}
pub fn get_client_id() -> Option<ClientId> {
    get_cookie(STORAGE_KEY, ClientId::from_str)
}
