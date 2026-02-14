use std::sync::LazyLock;

use crate::db::{
    db,
    entities::{edition, feedback},
};
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

pub async fn sync_feedback_to_kdrive() -> Result<()> {
    kdrive_sync_inner::<feedback::Entity>("feedback", "Feedback,E-Mail", |feedback| {
        format!(
            "{},{}\n",
            csv_str(feedback.content),
            csv_str(feedback.email.unwrap_or_default())
        )
    })
    .await
}

pub async fn sync_editions_to_kdrive() -> Result<()> {
    kdrive_sync_inner::<edition::Entity>("edition", "Date,Title,Views,OldViews,Hidden", |edition| {
        format!(
            "{},{},{},{},{}\n",
            edition.date,
            csv_str(edition.title.unwrap_or_default()),
            edition.views,
            edition.old_views,
            edition.hidden,
        )
    })
    .await
}

async fn kdrive_sync_inner<Entity: EntityTrait>(
    entity_name: &str,
    columns: &str,
    format_entity: impl Fn(Entity::Model) -> String,
) -> Result<()> {
    let read = async |file: &str| {
        tokio::fs::read_to_string(format!("kdrive/{file}"))
            .await
            .map_err(|err| ServerFnError::new(format!("Failed to read {file}: {err}")))
    };

    let drive_id = read("drive-id").await?;
    let oauth_token = read("oauth-token").await?;
    let directory_id = read("directory-id").await?;

    let entities = Entity::find().all(db()).await.map_err(|err| {
        ServerFnError::new(format!("Failed to get all {entity_name} entities: {err}"))
    })?;

    let csv = std::iter::once(format!("{columns}\n"))
        .chain(entities.into_iter().map(format_entity))
        .collect::<String>();

    let url = Url::parse_with_params(
        &format!(
            "https://api.infomaniak.com/3/drive/{}/upload",
            drive_id.trim()
        ),
        &[
            ("directory_id", directory_id.trim()),
            ("conflict", "version"),
            ("file_name", &format!("{entity_name}.csv")),
            ("total_size", &csv.len().to_string()),
        ],
    )
    .map_err(|err| ServerFnError::new(format!("Failed to construct url: {err}")))?;

    let response = CLIENT
        .post(url)
        .bearer_auth(oauth_token.trim())
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

/// Escape rust String to a csv string
fn csv_str(string: String) -> String {
    format!("\"{}\"", string.replace('"', "\"\""))
}
