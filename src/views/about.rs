use crate::i18n::{self, Language};
use dioxus::prelude::*;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::ops::Index;
#[cfg(feature = "server")]
use tokio::sync::RwLock;

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Team {
    redaktion: Vec<Person>,
    journalists: Vec<Person>,
    ronnie_middle_names: Vec<String>,
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
    ronnie_middle_names: Vec::new(),
});

#[server]
async fn get_team() -> Result<Team> {
    use rand::seq::SliceRandom;

    let mut rng = rand::rng();

    let mut team = TEAM.read().await.clone();
    team.redaktion.shuffle(&mut rng);
    team.journalists.shuffle(&mut rng);

    team.journalists.iter_mut().map(|journalist| {
        journalist.nickname = journalist.nickname.replace(
            "$ronnieNickName",
            team.ronnie_middle_names
                .choose(&mut rng)
                .ok_or("No middle names for Ronnie")?,
        );
    });

    Ok(team)
}

#[component]
pub fn About() -> Element {
    let lang = i18n::use_lang();
    let team = use_loader(|| async move { get_team().await })?;

    rsx! {
        div {
            h1 { class: "text-4xl", "{lang.read().about_title()}" }
            // TODO: "{lang.read().about_text()}"
            People {
                title: lang.read().redaktion_title(),
                people: team.read().redaktion.clone(),
            }
            br {}
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
        h2 { class: "text-3xl", "{title}" }
        div { class: "flex flex-wrap justify-center gap-x-10 gap-y-6",
            for member in people {
                div { class: "inline-flex flex-col items-center text-center",
                    div { class: "w-24 h-24 rounded-full overflow-hidden",
                        img {
                            class: "w-full h-full object-cover",
                            src: member.profile_picture_url.clone(),
                        }
                    }
                    h3 { class: "text-2xl", "{member.nickname}" }
                    ul {
                        for role in &member.roles {
                            li { "{role[*lang.read()]}" }
                        }
                    }
                }
            }
        }
    }
}
