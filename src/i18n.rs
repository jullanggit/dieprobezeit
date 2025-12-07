// target access pattern
//
// let translation = Translation::LANG;
// let word = translation.word;
macro_rules! Translation {
    {[$(($flang:ident, $lang_str:literal)),*], $($key:ident: ($($lang:ident: $trans:literal),*))*} => {
        pub enum Language {
            $($flang),*
        }
        impl Language {
            pub const fn to_str(&self) -> &'static str {
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
        DE: "Willkommen zur monatlichen Ausgabe der Probe Zeit!",
        CH: "Willkomme zur monatliche Usgab vo de Probe Ziit!",
        EN: "Welcome to the monthly Edition of the Probe Zeit")
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
        DE: "Fehler beim laden der Ausgaben",
        CH: "Fehler bim lade vo de Usgabe",
        EN: "Error loading editions")
    error_loading_edition: (
        DE: "Fehler beim laden der Ausgabe",
        CH: "Fehler bim lade vo de Usgab",
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
        CH: "Feedback erfolgreich gsendet",
        EN: "Feedback sent succesfully")
    error_loading_archive: (
        DE: "Fehler beim laden des Archivs",
        CH: "Fehler bim lade vom Archiv",
        EN: "Error loading archive")
    loading_archive: (
        DE: "Archiv wird geladen...",
        CH: "Archiv isch am lade...",
        EN: "Loading archive...")
}
