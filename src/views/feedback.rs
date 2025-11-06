use crate::EditionId;
use dioxus::{fullstack::Form, prelude::*};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct FeedbackForm {
    content: String,
    email: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct FeedbackRequest {
    form: FeedbackForm,
    edition_id: Option<i32>,
}

#[server]
async fn send_feedback(data: FeedbackRequest) -> Result<()> {
    use crate::db::{db, entities::feedback};
    use sea_orm::{EntityTrait, Set};

    let feedback = feedback::ActiveModel {
        content: Set(data.form.content.clone()),
        email: Set(data.form.email.clone()),
        edition_id: Set(data.edition_id),
        ..Default::default()
    };

    // TODO: check for duplicates
    feedback::Entity::insert(feedback).exec(db()).await?;
    Ok(())

    // TODO: send email
}

#[component]
pub fn Feedback(edition_id: Option<EditionId>) -> Element {
    rsx! {
        form {
            onsubmit: move |evt: FormEvent| async move {
                evt.prevent_default();
                let form: FeedbackForm = evt.parsed_values().unwrap();

                let _ = send_feedback(FeedbackRequest{form, edition_id}).await; // TODO: send notification or smth on error
            },
            label { "Feedback" }
            br {}
            textarea {
                id: "content", name: "content",
                style: "color: black;"
            }
            br {}
            label { "Optional: Email für weiteren Kontakt" }
            br {}
            input {
                type: "text", id: "email", name: "email",
                style: "color: black;"
            }
            br {}
            button { "Senden" }
        }
    }
}
