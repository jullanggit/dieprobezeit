use std::sync::LazyLock;

use crate::db::{db, entities::feedback};
use dioxus::{
    fullstack::{
        reqwest::{self, Url},
        reqwest_response_to_serverfn_err,
    },
    prelude::*,
};
use sea_orm::EntityTrait;
use serde::Deserialize;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Deserialize)]
struct UploadResponse {
    result: String,
}

#[server]
pub async fn sync_feedback_to_kdrive() -> Result<()> {
    let read = async |file: &str| {
        tokio::fs::read_to_string(file)
            .await
            .map_err(|err| ServerFnError::new(format!("Failed to read {file}: {err}")))
    };

    let drive_id = read("drive-id").await?;
    let oauth_token = read("oauth-token").await?;
    let directory_id = read("directory-id").await?;

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

    let url = Url::parse_with_params(
        &format!("https://api.infomaniak.com/3/drive/{drive_id}/upload"),
        &[
            ("directory_id", directory_id),
            ("conflict", "version".into()),
            ("file_name", "feedback.csv".into()),
            ("total_size", csv.len().to_string()),
        ],
    )
    .map_err(|err| ServerFnError::new(format!("Failed to construct url: {err}")))?;

    let response = CLIENT
        .post(url)
        .bearer_auth(oauth_token)
        .body(csv)
        .send()
        .await
        .map_err(reqwest_response_to_serverfn_err)?;

    let json: UploadResponse = response.json().await?;

    if &json.result == "success" {
        Ok(())
    } else {
        Err(ServerFnError::new("Failed to upload file").into())
    }
}
