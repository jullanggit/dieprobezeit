use crate::{
    components::{view_edition, EditionId},
    i18n,
    views::Feedback,
};
use dioxus::prelude::*;

#[component]
pub fn Edition(id: EditionId) -> Element {
    let data = use_server_future(move || async move { view_edition(id).await })?;

    let lang = i18n::use_lang();

    rsx! {
        div {
            match &*data.read_unchecked() {
                Some(Ok(data)) => rsx! {
                    h3 { class: "text-2xl", "{data.label()}" }
                    div { style: "display: inline-block;",
                        iframe {
                            src: "/html/{data.date}.html",
                            width: "100%",
                            border: "none",
                            overflow: "hidden",
                            scrolling: "no",

                    // run once when the iframe element is mounted

                            // Get the web_sys::Element for this iframe
                            // Treat it as an HtmlIFrameElement

                            // Attach an onload handler so we run *after*
                            // the pdf2htmlEX page has fully loaded & laid out
                            // Set the iframe's height in px

                            // Leak the closure so it lives as long as the iframe
                            onmounted: move |mounted| {
                                #[cfg(feature = "web")]
                                {
                                    let element = mounted.data().downcast::<web_sys::Element>().cloned().unwrap();


                                    let iframe: HtmlIFrameElement = element.unchecked_into();
                                    let closure = Closure::<
                                        dyn FnMut(),
                                    >::new(move || {
                                        if let Some(doc) = iframe.content_document() {
                                            if let Some(body) = doc.body() {
                                                let height = body.scroll_height();
                                                let _ = iframe
                                                    .style()
                                                    .set_property("height", &format!("{height}px"));
                                            }
                                        }
                                    });
                                    iframe.set_onload(Some(closure.as_ref().unchecked_ref()));
                                    closure.forget();
                                }
                            },
                        }
                    }
                },
                Some(Err(e)) => rsx! { "{lang.read().error_loading_edition()}: {e}" },
                None => rsx! { "{lang.read().loading_edition()}" },
            }
            Feedback { edition_id: id }
        }
    }
}
