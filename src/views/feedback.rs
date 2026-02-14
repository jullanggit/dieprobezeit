use crate::{EditionId, i18n};
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

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
    let mut submitted = use_signal(|| false);
    let mut content = use_signal(String::new);
    let mut email = use_signal(String::new);

    let lang = i18n::use_lang();

    // hide feedback if it was submitted
    rsx! {
        if !*submitted.read() {
            form {
                onsubmit: move |evt: FormEvent| async move {
                    evt.prevent_default();
                    let form: FeedbackForm = evt.parsed_values().unwrap();

                    // TODO: display error or smth if it is not ok
                    if send_feedback(FeedbackRequest {
                            form,
                            edition_id,
                        })
                        .await
                        .is_ok()
                    {
                        content.set(String::new());
                        email.set(String::new());
                        submitted.set(true);
                    }
                },
                label { "Feedback" }
                br {}
                textarea {
                    id: "content",
                    name: "content",
                    style: "color: black;",
                    value: "{content}",
                    oninput: move |evt| content.set(evt.value()),
                }
                br {}
                label { "{lang.read().optional_email()}" }
                br {}
                input {
                    r#type: "text",
                    id: "email",
                    name: "email",
                    style: "color: black;",
                    value: "{email}",
                    oninput: move |evt| email.set(evt.value()),
                }
                br {}
                button { "{lang.read().send()}" }
            }
        } else {
            "{lang.read().feedback_sent()}"
        }
    }
}
