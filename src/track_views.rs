use crate::cookies::{get_cookie, get_or_insert_cookie};
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
