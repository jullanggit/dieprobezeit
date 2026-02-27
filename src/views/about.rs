use std::ops::Index;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
#[cfg(feature = "server")]
use tokio::sync::RwLock;

use crate::i18n::{self, Language};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Team {
    redaktion: Vec<Person>,
    journalists: Vec<Person>,
}
#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct Person {
    nickname: String,
    profile_picture_url: String,
    roles: Vec<TranslatedRole>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
struct TranslatedRole {
    de: String,
    en: String,
    ch: String,
}
impl Index<Language> for TranslatedRole {
    type Output = String;
    fn index(&self, index: Language) -> &Self::Output {
        match index {
            Language::DE => &self.de,
            Language::CH => &self.ch,
            Language::EN => &self.en,
        }
    }
}

#[cfg(feature = "server")]
pub static TEAM: RwLock<Team> = RwLock::const_new(Team {
    redaktion: Vec::new(),
    journalists: Vec::new(),
});

#[server]
async fn get_team() -> Result<Team> {
    Ok(TEAM.read().await.clone())
}

#[component]
pub fn About() -> Element {
    let lang = i18n::use_lang();
    let team = use_loader(|| async move { get_team().await })?;

    rsx! {
        div {
            h1 { class: "text-4xl", "{lang.read().about_title()" }
            // TODO: "{lang.read().about_text()}"
            People {
                title: lang.read().redaktion_title(),
                people: team.read().redaktion.clone(),
            }
            People {
                title: lang.read().journalists_title(),
                people: team.read().journalists.clone(),
            }
        }
    }
}

#[component]
fn People(title: &'static str, people: Vec<Person>) -> Element {
    let lang = i18n::use_lang();

    rsx! {
        h3 { class: "text-2xl", "{title}" }
        div { class: "flex flex-wrap justify-center",
            for member in people {
                img {
                    class: "rounded-full object-cover",
                    src: member.profile_picture_url.clone(),
                    width: 100,
                    height: 100,
                }
                br {}
                h3 { class: "text-2xl", "{member.nickname}" }
                br {}
                ul {
                    for role in &member.roles {
                        li { "{role[*lang.read()]}" }
                    }
                }
            }
        }
    }
}
