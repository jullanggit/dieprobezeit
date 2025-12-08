use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::db::db;
#[cfg(feature = "server")]
use sea_orm::{sea_query::Expr, ColumnTrait, EntityTrait, QueryFilter};

use crate::db::entities::edition;

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
        .map_err(|err| ServerFnError::new(format!("{err}")))
}

#[server]
pub async fn view_edition(id: i32) -> Result<edition::Model, ServerFnError> {
    let db = db();

    // increment view count
    let _ = edition::Entity::update_many()
        .col_expr(
            edition::Column::Views,
            Expr::col(edition::Column::Views).add(1),
        )
        .filter(edition::Column::Id.eq(id))
        .exec(db)
        .await; // ignore error, TODO: log instead

    // get updated entity
    edition::Entity::find_by_id(id)
        .one(db)
        .await
        .map_err(|err| ServerFnError::new(format!("{err}")))?
        .ok_or(ServerFnError::new(format!("Edition {id} not found")))
}
