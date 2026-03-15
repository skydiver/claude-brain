use std::cell::RefCell;
use std::rc::Rc;

use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::window;

use crate::models::Theme;
use crate::settings::SettingsContext;

type ListenerHandle =
    Option<(Closure<dyn FnMut(web_sys::MediaQueryListEvent)>, web_sys::MediaQueryList)>;

fn apply_dark_class(dark: bool) {
    if let Some(el) = window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let class_list = el.class_list();
        if dark {
            let _ = class_list.add_1("dark");
        } else {
            let _ = class_list.remove_1("dark");
        }
    }
}

fn is_os_dark() -> bool {
    window()
        .and_then(|w| w.match_media("(prefers-color-scheme: dark)").ok().flatten())
        .map(|mql| mql.matches())
        .unwrap_or(false)
}

pub fn init_theme() {
    let ctx = expect_context::<SettingsContext>();

    // Rc<RefCell> because Closure is not Clone (can't use StoredValue)
    let listener_handle: Rc<RefCell<ListenerHandle>> = Rc::new(RefCell::new(None));

    // React to theme changes
    let handle = listener_handle.clone();
    Effect::new(move || {
        let theme = ctx.settings.get().appearance.theme;

        // Remove existing listener if any
        {
            let mut guard = handle.borrow_mut();
            if let Some((closure, mql)) = guard.take() {
                let _ = mql.remove_event_listener_with_callback(
                    "change",
                    closure.as_ref().unchecked_ref(),
                );
            }
        }

        match theme {
            Theme::System => {
                apply_dark_class(is_os_dark());
                // Set up matchMedia listener
                if let Some(win) = window() {
                    if let Ok(Some(mql)) = win.match_media("(prefers-color-scheme: dark)") {
                        let closure = Closure::wrap(
                            Box::new(move |event: web_sys::MediaQueryListEvent| {
                                apply_dark_class(event.matches());
                            })
                                as Box<dyn FnMut(_)>,
                        );
                        let _ = mql.add_event_listener_with_callback(
                            "change",
                            closure.as_ref().unchecked_ref(),
                        );
                        *handle.borrow_mut() = Some((closure, mql));
                    }
                }
            }
            Theme::DefaultDark => apply_dark_class(true),
            Theme::DefaultLight => apply_dark_class(false),
        }
    });
}
