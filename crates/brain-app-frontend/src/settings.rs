use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::models::Settings;

#[derive(Clone, Copy)]
pub struct SettingsContext {
    pub settings: ReadSignal<Settings>,
    set_settings: WriteSignal<Settings>,
}

impl SettingsContext {
    /// Update settings: sends the full Settings to the backend, updates signal on success.
    pub fn update(&self, new_settings: Settings) {
        let set_settings = self.set_settings;
        spawn_local(async move {
            match api::update_settings(new_settings).await {
                Ok(saved) => set_settings.set(saved),
                Err(e) => {
                    web_sys::console::error_1(&format!("Failed to save settings: {e}").into())
                }
            }
        });
    }
}

/// Initialize settings context. Call from App component.
/// Provides context synchronously with defaults, then updates when backend responds.
pub fn init_settings() {
    let (settings, set_settings) = signal(Settings::default());
    provide_context(SettingsContext {
        settings,
        set_settings,
    });

    // Load from backend and update the signal
    spawn_local(async move {
        match api::get_settings().await {
            Ok(loaded) => set_settings.set(loaded),
            Err(e) => {
                web_sys::console::error_1(&format!("Failed to load settings: {e}").into())
            }
        }
    });
}
