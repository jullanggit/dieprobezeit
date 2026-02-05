use uuid::Uuid;

use crate::components::EditionId;
use std::collections::HashMap;

/// A Client UUID used for view deduplication
pub struct ClientId(pub Uuid);
impl ClientId {
    /// Creates a new random Client ID. May fail if window or crypto couldn't be accessed.
    #[cfg(feature = "web")]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

type EditionProgress = f32;
pub struct EditionProgresses(HashMap<EditionId, EditionProgress>);
