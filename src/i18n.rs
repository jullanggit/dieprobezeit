use crate::cookies::{get_or_insert_cookie, set_cookie};
use dioxus::prelude::*;

const STORAGE_KEY: &str = "lang";
pub const DEFAULT_LANG: Language = Language::DE;

/// Store language to cookies. No-op on non-web builds
#[allow(unused_variables)]
pub fn set_lang(language: Language) {
    set_cookie(STORAGE_KEY, language.to_str());
}

/// Get language setting from cookies. Returns DEFAULT_LANG on failure.
/// Also sets 'lang' to DEFAULT lang on failure on web builds.
pub fn get_lang() -> Language {
    get_or_insert_cookie(STORAGE_KEY, DEFAULT_LANG.to_str(), Language::from_str)
}

pub fn use_lang() -> Signal<Language> {
    use_context::<Signal<Language>>()
}

// target access pattern
//
// let translation = Translation::LANG;
// let word = translation.word();
macro_rules! Translation {
    {[$(($flang:ident, $lang_str:literal)),*], $($key:ident: ($($lang:ident: $trans:literal),*))*} => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum Language {
            $($flang),*
        }
        impl Language {
            pub const fn to_str(self) -> &'static str {
                match self {
                    $(
                        Self::$flang => $lang_str,
                    )*
                }
            }
            pub fn from_str(str: &str) -> Option<Self> {
                match str {
                    $(
                        $lang_str => Some(Self::$flang),
                    )*
                    _ => None
                }
            }
            pub const fn display_name(&self) -> &'static str {
                match self {
                    $(
                        Self::$flang => stringify!($flang),
                    )*
                }
            }
            pub const fn variants() -> [Self; [$(Self::$flang),*].len()] {
                [$(Self::$flang),*]
            }
            $(
                pub const fn $key(&self) -> &'static str {
                    match self {
                        $(
                            Language::$lang => $trans
                        ),*
                    }
                }
            )*
        }
    }
}

Translation! {
    [(DE, "de"), (CH, "ch"), (EN, "en")],
    welcome: (
        DE: "Willkommen zu der Probe Zeit!",
        CH: "Willkomme zur Probe Ziit!",
        EN: "Welcome to the Probe Zeit")
    newest_edition: (
        DE: "Neuste Ausgabe",
        CH: "Neusti Usgab",
        EN: "Newest Edition")
    loading_editions: (
        DE: "Ausgaben werden geladen...",
        CH: "Usgabe sind am lade...",
        EN: "Loading editions...")
    loading_edition: (
        DE: "Ausgabe wird geladen...",
        CH: "Usgab isch am lade...",
        EN: "Loading edition...")
    error_loading_editions: (
        DE: "Fehler beim Laden der Ausgaben",
        CH: "Fehler bim Lade vo de Usgabe",
        EN: "Error loading editions")
    error_loading_edition: (
        DE: "Fehler beim Laden der Ausgabe",
        CH: "Fehler bim Lade vo de Usgab",
        EN: "Error loading edition")
    loading_pdf: (
        DE: "PDF wird geladen...",
        CH: "PDF isch am lade...",
        EN: "Loading PDF...")
    no_edition_found: (
        DE: "Keine Ausgabe gefunden",
        CH: "Kei Usgab gfunde",
        EN: "No edition found")
    archive: (
        DE: "Archiv",
        CH: "Archiv",
        EN: "Archive")
    page_not_found: (
        DE: "Seite nicht gefunden",
        CH: "Siite nöd gfunde",
        EN: "Page not found")
    optional_email: (
        DE: "Optional: Email für weiteren Kontakt",
        CH: "Optional: Email für wiitere Kontakt",
        EN: "Optional: Email for further contact")
    send: (
        DE: "Senden",
        CH: "Sende",
        EN: "Send")
    feedback_sent: (
        DE: "Feedback erfolgreich gesendet",
        CH: "Feedback erfolgriich gsendet",
        EN: "Feedback sent succesfully")
    archive_title: (
        DE: "Archiv aller Ausgaben",
        CH: "Archiv vo allne Usgabe",
        EN: "Archive of all editions")
    error_loading_archive: (
        DE: "Fehler beim Laden des Archivs",
        CH: "Fehler bim Lade vom Archiv",
        EN: "Error loading archive")
    loading_archive: (
        DE: "Archiv wird geladen...",
        CH: "Archiv isch am lade...",
        EN: "Loading archive...")
}
