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
    let cookie = format!(
        "{key}={value}; Path=/; Max-Age={}; SameSite=Lax",
        2 * 365 * 24 * 60 * 60 // 2 years as max age
    );

    #[cfg(feature = "web")]
    {
        if let Some(document) = html_document() {
            let _ = document.set_cookie(&cookie);
        }
    }
    #[cfg(feature = "server")]
    {
        use std::str::FromStr;

        use dioxus::fullstack::{
            headers::{HeaderName, HeaderValue},
            FullstackContext,
        };
        if let (Some(context), Ok(header_name), Ok(header_value)) = (
            FullstackContext::current(),
            HeaderName::from_str("Set-Cookie"),
            HeaderValue::from_str(&cookie),
        ) {
            context.add_response_header(header_name, header_value);
        }
    }
}
pub fn get_cookie<T>(key: &str, parse: impl Fn(&str) -> Option<T>) -> Option<T> {
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
    }
    #[cfg(feature = "server")]
    {
        use dioxus::fullstack::{headers::HeaderMapExt, Cookie, FullstackContext};

        FullstackContext::current()
            .and_then(|context| context.parts_mut().headers.typed_get::<Cookie>())
            .and_then(|cookie| cookie.get(key).and_then(&parse))
    }
}

/// Get value from cookies. Returns default on failure.
/// Also sets 'key' to default on failure on web builds.
pub fn get_or_insert_cookie<T>(
    key: &str,
    default: impl Fn() -> String,
    parse: impl Fn(&str) -> Option<T>,
) -> T {
    get_cookie(key, &parse).unwrap_or_else(|| {
        set_cookie(key, &default());
        parse(&default()).expect("default should be parseable")
    })
}
