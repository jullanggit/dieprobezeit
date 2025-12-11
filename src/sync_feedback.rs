use std::iter::chain;

use dioxus::prelude::*;
use sea_orm::EntityTrait;

use crate::db::{db, entities::feedback};

#[server]
pub async fn sync_feedback_to_kdrive() -> Result<()> {
    let drive_id = tokio::fs::read_to_string("drive-id")
        .await
        .map_err(|err| "Failed to retrieve drive id: {err}");
    let oath_token = tokio::fs::read_to_string("oauth-token")
        .await
        .map_err(|err| "Failed to retrieve drive id: {err}");

    let feedbacks = feedback::Entity::find()
        .all(db())
        .await
        .map_err(|err| ServerFnError::new(format!("Failed to get all feedback entities: {err}")))?;

    let csv = std::iter::once("Feedback,E-Mail\n".into())
        .chain(feedbacks.into_iter().map(|feedback| {
            format!(
                "{},{}\n",
                feedback.content,
                feedback.email.unwrap_or_default()
            )
        }))
        .collect::<String>();

    Ok(())
}
