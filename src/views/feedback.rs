use dioxus::{fullstack::Form, prelude::*};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct FeedbackForm {
    content: String,
    email: Option<String>,
}

#[server]
async fn send_feedback(form: Form<FeedbackForm>) -> Result<()> {
    use crate::db::{db, entities::feedback};
    use sea_orm::{EntityTrait, Set};

    let feedback = feedback::ActiveModel {
        content: Set(form.content.clone()),
        email: Set(form.email.clone()),
        ..Default::default()
    };

    // TODO: check for duplicates
    feedback::Entity::insert(feedback).exec(db()).await?;
    Ok(())

    // TODO: send email
}

#[component]
pub fn Feedback() -> Element {
    rsx! {
        form {
            onsubmit: move |evt: FormEvent| async move {
                evt.prevent_default();
                let values: FeedbackForm = evt.parsed_values().unwrap();

                let _ = send_feedback(Form(values)).await; // TODO: send notification or smth on error
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
