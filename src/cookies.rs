use dioxus::prelude::*;

#[cfg(feature = "web")]
fn html_document() -> Option<web_sys::HtmlDocument> {
    use web_sys::wasm_bindgen::JsCast;

    web_sys::window()?
        .document()?
        .dyn_into::<web_sys::HtmlDocument>()
        .ok()
}

/// Store (key, value) to cookies. No-op on non-web builds
#[allow(unused_variables)]
pub fn set_cookie(key: &str, value: &str) {
    #[cfg(feature = "web")]
    {
        if let Some(document) = html_document() {
            let _ = document.set_cookie(
                &format!(
                    "{key}={value}; Path=/; Max-Age={}; SameSite=Lax",
                    2 * 365 * 24 * 60 * 60
                ), // 2 years as max age
            );
        }
    }
}

/// Get value from cookies. Returns default on failure.
/// Also sets 'key' to default on failure on web builds.
pub fn get_or_insert_cookie<T>(key: &str, default: &str, parse: impl Fn(&str) -> Option<T>) -> T {
    #[cfg(feature = "web")]
    {
        html_document()
            .and_then(|html_document| html_document.cookie().ok())
            .and_then(|string| {
                string.split(';').find_map(|kv| {
                    let (cookie_key, value) = kv.trim().split_once('=')?;
                    cookie_key.eq(key).then(|| parse(value)).flatten()
                })
            })
            .unwrap_or_else(|| {
                set_cookie(key, default);
                parse(default).expect("default should be parseable")
            })
    }
    #[cfg(feature = "server")]
    {
        use dioxus::fullstack::{headers::HeaderMapExt, Cookie, FullstackContext};

        FullstackContext::current()
            .and_then(|context| context.parts_mut().headers.typed_get::<Cookie>())
            .and_then(|cookie| cookie.get(key).and_then(&parse))
            .unwrap_or(parse(default).expect("default should be parseable"))
    }
}
