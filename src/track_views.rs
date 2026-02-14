use crate::{
    components::EditionId,
    cookies::{get_cookie, get_or_insert_cookie},
};
use dioxus::prelude::*;
use sea_orm::{
    prelude::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set,
};
use std::str::FromStr;
use time::{Duration, PrimitiveDateTime, UtcDateTime};
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

/// Record read times from a client.
/// Merge any reads within 30 minutes.
/// Cap read time at 3 minutes per page. This resets every day to permit re-reads.
#[post("/api/record-read-times")]
pub async fn record_read_times(
    edition_id: EditionId,
    page_times: Vec<f32>,
) -> Result<(), ServerFnError> {
    use crate::db::{db, entities::reads};

    let db = db();
    let client_id = get_client_id().map_or(NO_ID, |client_id| client_id.0);

    let cap_cutoff = UtcDateTime::now() - Duration::days(1);
    let merge_cutoff = UtcDateTime::now() - Duration::minutes(30);

    for (page, time) in page_times
        .iter()
        .enumerate()
        .filter(|(_, time)| **time != 0.)
    {
        // all entities with matching client id edition id and page number
        let matching_entities = reads::Entity::find()
            .filter(reads::Column::ClientId.eq(client_id))
            .filter(reads::Column::EditionId.eq(edition_id))
            .filter(reads::Column::PageNumber.eq(page as i32));

        // Reads in the last day
        let read_time_within_day: f32 = matching_entities
            .clone()
            .filter(
                reads::Column::Timestamp
                    .gt(PrimitiveDateTime::new(cap_cutoff.date(), cap_cutoff.time())),
            )
            .select_only()
            .expr(reads::Column::ReadTime.sum())
            .into_tuple()
            .one(db)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?
            .unwrap_or(0.0);

        let three_minutes_ms = 3. * 60. * 1000.;
        if read_time_within_day >= three_minutes_ms {
            continue;
        }

        // try to find an existing entry within the merge cutoff
        let merge_entity = matching_entities
            .filter(reads::Column::Timestamp.gt(PrimitiveDateTime::new(
                merge_cutoff.date(),
                merge_cutoff.time(),
            )))
            .order_by_desc(reads::Column::Timestamp)
            .one(db)
            .await
            .map_err(|err| ServerFnError::new(err.to_string()))?;

        match merge_entity {
            Some(entity) => {
                let mut active: reads::ActiveModel = entity.into();
                let current_read_time = active.read_time.take().unwrap_or(0.0);
                active.read_time = Set(current_read_time + *time);

                active
                    .update(db)
                    .await
                    .map_err(|err| ServerFnError::new(err.to_string()))?;
            }
            None => {
                let entity = reads::ActiveModel {
                    client_id: Set(client_id),
                    edition_id: Set(edition_id),
                    page_number: Set(page as i32),
                    read_time: Set(*time),
                    ..Default::default()
                };
                reads::Entity::insert(entity)
                    .exec(db)
                    .await
                    .map_err(|err| ServerFnError::new(err.to_string()))?;
            }
        }
    }

    Ok(())
}
