use dioxus::prelude::*;

const STORAGE_KEY: &str = "lang";
pub const DEFAULT_LANG: Language = Language::DE;

#[cfg(feature = "web")]
fn html_document() -> Option<web_sys::HtmlDocument> {
    use web_sys::wasm_bindgen::JsCast;

    web_sys::window()?
        .document()?
        .dyn_into::<web_sys::HtmlDocument>()
        .ok()
}

/// Store language to cookies. No-op on non-web builds
#[allow(unused_variables)]
pub fn set_lang(language: Language) {
    #[cfg(feature = "web")]
    {
        if let Some(document) = html_document() {
            let _ = document.set_cookie(&format!(
                "{STORAGE_KEY}={}; Path=/; Max-Age=3153600; SameSite=Lax",
                language.to_str()
            ));
        }
    }
}

/// Get language setting from cookies. Returns DEFAULT_LANG on failure.
/// Also sets 'lang' to DEFAULT lang on failure on web builds.
pub fn get_lang() -> Language {
    #[cfg(feature = "web")]
    {
        html_document()
            .and_then(|html_document| html_document.cookie().ok())
            .and_then(|string| {
                string.split(';').find_map(|kv| {
                    let (key, value) = kv.trim().split_once('=')?;
                    key.eq(STORAGE_KEY)
                        .then(|| Language::from_str(value))
                        .flatten()
                })
            })
            .unwrap_or_else(|| {
                set_lang(DEFAULT_LANG);
                DEFAULT_LANG
            })
    }
    #[cfg(feature = "server")]
    {
        use dioxus::fullstack::{headers::HeaderMapExt, Cookie, FullstackContext};

        FullstackContext::current()
            .and_then(|context| context.parts_mut().headers.typed_get::<Cookie>())
            .and_then(|cookie| cookie.get(STORAGE_KEY).and_then(Language::from_str))
            .unwrap_or(DEFAULT_LANG)
    }
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
