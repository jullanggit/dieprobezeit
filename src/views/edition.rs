use crate::{
    components::{EditionId, ViewEdition, view_edition},
    i18n,
    track_views::ensure_client_id_set,
    views::Feedback,
};
use dioxus::prelude::*;

#[component]
pub fn Edition(id: EditionId) -> Element {
    let data = use_server_future(move || async move { view_edition(id).await })?;
    use_hook(ensure_client_id_set);

    let lang = i18n::use_lang();

    track_reads(data, id);

    rsx! {
        div {
            match &*data.read_unchecked() {
                Some(Ok(data)) => rsx! {
                    h3 { class: "text-2xl", "{data.edition.label()}" }
                    for i in 1..=data.num_pages {
                        div {
                            id: "edition-page-{i}",
                            style: "display: inline-block; background-color: white; margin-right: 5px",
                            object {
                                data: "/svgs/{data.edition.date}/{i}.svg",
                                height: "auto",
                                width: "100%",
                            }
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

#[cfg(feature = "web")]
fn track_reads(data: Resource<Result<ViewEdition, ServerFnError>>, edition_id: EditionId) {
    use std::rc::Rc;
    use web_sys::{
        Element, IntersectionObserver, IntersectionObserverEntry, IntersectionObserverInit,
        js_sys::{
            self,
            wasm_bindgen::{JsValue, prelude::Closure},
        },
        wasm_bindgen::JsCast,
        window,
    };

    let mut visibilities = use_signal(|| Vec::new());
    let mut acc = use_signal(|| Vec::new());

    let mut observer = use_signal(|| None);
    let mut _observer_callback = use_signal(|| None);

    // initialize
    let mut initialized = use_signal(|| false);
    use_effect(move || {
        if initialized() {
            return;
        }
        let Some(Ok(data)) = &*data.read_unchecked() else {
            return;
        };
        let num_pages = data.num_pages as usize;
        visibilities.set(vec![0.0; num_pages]);
        acc.set(vec![0.0; num_pages]);

        let callback = Closure::new(move |entries: js_sys::Array, _obs: IntersectionObserver| {
            for entry in entries {
                let entry: IntersectionObserverEntry = entry.unchecked_into();
                let target: Element = entry.target();

                if let Some(id) = target.get_attribute("id")
                    && let Some(page_str) = id.strip_prefix("edition-page-")
                    && let Ok(page) = page_str.parse::<usize>()
                {
                    visibilities
                        .get_mut(page - 1)
                        .map(|mut visibility| *visibility = entry.intersection_ratio());
                }
            }
        });

        let thresholds = js_sys::Array::new();
        for i in 0..=100 {
            thresholds.push(&JsValue::from_f64(i as f64 / 100.0));
        }

        let mut init = IntersectionObserverInit::new();
        init.threshold(&thresholds.into());

        let obs = IntersectionObserver::new_with_options(callback.as_ref().unchecked_ref(), &init)
            .expect("IntersectionObserver should be available");

        let doc = window()
            .expect("Window should be available")
            .document()
            .expect("Document should be available");
        for page in 1..=num_pages {
            if let Some(element) = doc.get_element_by_id(&format!("edition-page-{page}")) {
                obs.observe(&element);
            }
        }

        observer.set(Some(obs));
        _observer_callback.set(Some(Rc::new(callback))); // keep callback alive

        initialized.set(true);
    });

    let now_ms = || {
        window()
            .and_then(|window| window.performance())
            .map(|performance| performance.now())
            .unwrap_or(0.0)
    };

    let mut last_sample = use_signal(now_ms);
    let mut last_flush = use_signal(now_ms);
    let mut interval_handle = use_signal(|| None);
    let mut _tick = use_signal(|| None);

    // initialize ticks
    use_effect(move || {
        if !initialized() || interval_handle().is_some() {
            return;
        }

        let tick = Closure::new(move || {
            let now = now_ms();
            let delta_t = now - last_sample();
            last_sample.set(now);

            if window()
                .and_then(|window| window.document())
                .map(|document| document.hidden() || !document.has_focus().unwrap_or(true))
                .unwrap_or_default()
            {
                return;
            }

            let visibilities = visibilities();

            let total = visibilities.iter().sum::<f64>();
            if total <= 0.0 || delta_t <= 0.0 {
                return;
            }

            for (visibility, mut acc_elem) in visibilities.iter().zip(acc.iter_mut()) {
                *acc_elem += delta_t * (*visibility / total)
            }

            if now - last_flush() >= 5000.0 {
                use crate::track_views::record_read_times;

                last_flush.set(now);

                // Take a snapshot and reset accumulator
                let snapshot = acc()
                    .into_iter()
                    .map(|float| float as f32)
                    .collect::<Vec<_>>();
                for mut value in acc.iter_mut() {
                    *value = 0.0;
                }

                if snapshot.iter().all(|value| *value == 0.0) {
                    return;
                }

                spawn(async move {
                    record_read_times(edition_id, snapshot).await;
                });
            }
        });

        let handle = window()
            .unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                tick.as_ref().unchecked_ref(),
                250,
            )
            .expect("setInterval should work");

        interval_handle.set(Some(handle));
        _tick.set(Some(Rc::new(tick))); // keep callback alive
    });

    // cleanup
    use_drop(move || {
        if let Some(handle) = interval_handle() {
            let _ = window().unwrap().clear_interval_with_handle(handle);
        }
        if let Some(obs) = observer() {
            obs.disconnect();
        }
    });
}

#[cfg(not(feature = "web"))]
fn track_reads(data: Resource<Result<ViewEdition, ServerFnError>>, id: EditionId) {}
