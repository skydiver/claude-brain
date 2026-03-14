use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

pub fn init_theme() {
    let Some(win) = window() else { return };

    let apply_theme = move |dark: bool| {
        if let Some(doc) = window().and_then(|w| w.document()) {
            if let Some(el) = doc.document_element() {
                let class_list = el.class_list();
                if dark {
                    let _ = class_list.add_1("dark");
                } else {
                    let _ = class_list.remove_1("dark");
                }
            }
        }
    };

    // Check initial theme
    let is_dark = win
        .match_media("(prefers-color-scheme: dark)")
        .ok()
        .flatten()
        .map(|mql| mql.matches())
        .unwrap_or(false);

    apply_theme(is_dark);

    // Listen for changes
    if let Ok(Some(mql)) = win.match_media("(prefers-color-scheme: dark)") {
        let closure =
            Closure::wrap(
                Box::new(move |event: web_sys::MediaQueryListEvent| {
                    apply_theme(event.matches());
                }) as Box<dyn FnMut(_)>,
            );

        let _ = mql.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref());
        closure.forget(); // leak intentionally — lives for app lifetime
    }
}
