use std::{iter, sync::LazyLock};

use crate::db::{
    db,
    entities::{edition, feedback, reads},
};
use dioxus::{
    fullstack::{
        reqwest::{self, Url},
        reqwest_response_to_serverfn_err,
    },
    prelude::*,
};
use sea_orm::{EntityTrait, QuerySelect, prelude::*};
use serde::Deserialize;

static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[derive(Deserialize)]
struct UploadResponse {
    result: String,
}

pub async fn sync_reads_to_kdrive() -> Result<()> {
    let db = db();

    // (edition id, total read time, unique readers)
    let per_edition: Vec<(i32, f32, i64)> = reads::Entity::find()
        .select_only()
        .column(reads::Column::EditionId)
        .expr(reads::Column::ReadTime.sum())
        .expr(Expr::col(reads::Column::ClientId).count_distinct())
        .group_by(reads::Column::EditionId)
        .into_tuple()
        .all(db)
        .await
        .map_err(|err| {
            ServerFnError::new(format!(
                "Failed to get total read times and unique reader counts per edition: {err}"
            ))
        })?;

    let total_unique_readers: Option<i64> = reads::Entity::find()
        .select_only()
        .expr(Expr::col(reads::Column::ClientId).count_distinct())
        .into_tuple()
        .one(db)
        .await
        .map_err(|err| {
            ServerFnError::new(format!("Failed to get total unique reader count: {err}"))
        })?;

    let csv =
        iter::once("Total Unique Readers,Edition ID,Total Read Time,Unique Readers\n".to_string())
            .chain(per_edition.iter().enumerate().map(
                |(i, (edition_id, total_read_time, unique_readers))| {
                    format!(
                        "{},{},{},{}\n",
                        if i == 0 {
                            total_unique_readers.unwrap_or_default()
                        } else {
                            0
                        },
                        edition_id,
                        total_read_time,
                        unique_readers,
                    )
                },
            ))
            .collect::<String>();

    upload_to_kdrive("reads", csv).await
}

pub async fn sync_feedback_to_kdrive() -> Result<()> {
    kdrive_sync_table::<feedback::Entity>("feedback", "Feedback,E-Mail", |feedback| {
        format!(
            "{},{}\n",
            csv_str(feedback.content),
            csv_str(feedback.email.unwrap_or_default())
        )
    })
    .await
}

pub async fn sync_editions_to_kdrive() -> Result<()> {
    kdrive_sync_table::<edition::Entity>("edition", "Date,Title,Views,OldViews,Hidden", |edition| {
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

async fn kdrive_sync_table<Entity: EntityTrait>(
    entity_name: &str,
    columns: &str,
    format_entity: impl Fn(Entity::Model) -> String,
) -> Result<()> {
    let entities = Entity::find().all(db()).await.map_err(|err| {
        ServerFnError::new(format!("Failed to get all {entity_name} entities: {err}"))
    })?;

    let csv = std::iter::once(format!("{columns}\n"))
        .chain(entities.into_iter().map(format_entity))
        .collect::<String>();

    upload_to_kdrive(&format!("{entity_name}.csv"), csv).await
}

/// Escape rust String to a csv string
fn csv_str(string: String) -> String {
    format!("\"{}\"", string.replace('"', "\"\""))
}

async fn upload_to_kdrive(file_name: &str, content: String) -> Result<()> {
    let read = async |file: &str| {
        tokio::fs::read_to_string(format!("kdrive/{file}"))
            .await
            .map_err(|err| ServerFnError::new(format!("Failed to read {file}: {err}")))
    };

    let drive_id = read("drive-id").await?;
    let oauth_token = read("oauth-token").await?;
    let directory_id = read("directory-id").await?;

    let url = Url::parse_with_params(
        &format!(
            "https://api.infomaniak.com/3/drive/{}/upload",
            drive_id.trim()
        ),
        &[
            ("directory_id", directory_id.trim()),
            ("conflict", "version"),
            ("file_name", file_name),
            ("total_size", &content.len().to_string()),
        ],
    )
    .map_err(|err| ServerFnError::new(format!("Failed to construct url: {err}")))?;

    let response = CLIENT
        .post(url)
        .bearer_auth(oauth_token.trim())
        .body(content)
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
