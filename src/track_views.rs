use crate::components::EditionId;
use std::collections::HashMap;

/// A Client UUID used for view deduplication
pub struct ClientId(pub String);
impl ClientId {
    /// Creates a new random Client ID. May fail if window or crypto couldn't be accessed.
    #[cfg(feature = "web")]
    pub fn new() -> Option<Self> {
        Some(Self(web_sys::window()?.crypto().ok()?.random_uuid()))
    }
}

type EditionProgress = f32;
pub struct EditionProgresses(HashMap<EditionId, EditionProgress>);
