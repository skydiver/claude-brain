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

const THEME_CLASSES: &[&str] = &[
    "dark",
    "theme-solarized-dark",
    "theme-nord",
    "theme-catppuccin-mocha",
    "theme-dracula",
    "theme-tokyo-night",
];

fn apply_theme_class(class: Option<&str>) {
    if let Some(el) = window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
    {
        let class_list = el.class_list();
        // Remove all theme classes
        for &cls in THEME_CLASSES {
            let _ = class_list.remove_1(cls);
        }
        // Apply the requested class
        if let Some(cls) = class {
            let _ = class_list.add_1(cls);
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
                apply_theme_class(if is_os_dark() { Some("dark") } else { None });
                // Set up matchMedia listener
                if let Some(win) = window() {
                    if let Ok(Some(mql)) = win.match_media("(prefers-color-scheme: dark)") {
                        let closure = Closure::wrap(
                            Box::new(move |event: web_sys::MediaQueryListEvent| {
                                apply_theme_class(if event.matches() { Some("dark") } else { None });
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
            Theme::DefaultDark => apply_theme_class(Some("dark")),
            Theme::DefaultLight => apply_theme_class(None),
            Theme::SolarizedDark => apply_theme_class(Some("theme-solarized-dark")),
            Theme::Nord => apply_theme_class(Some("theme-nord")),
            Theme::CatppuccinMocha => apply_theme_class(Some("theme-catppuccin-mocha")),
            Theme::Dracula => apply_theme_class(Some("theme-dracula")),
            Theme::TokyoNight => apply_theme_class(Some("theme-tokyo-night")),
        }
    });
}
