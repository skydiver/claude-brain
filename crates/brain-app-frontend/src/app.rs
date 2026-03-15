use leptos::prelude::*;

use crate::pages::browse::BrowsePage;
use crate::settings::init_settings;
use crate::theme::init_theme;

#[component]
pub fn App() -> impl IntoView {
    // Provide settings context synchronously, then load from backend
    init_settings();
    // Theme reads from SettingsContext — defaults to System until backend responds
    init_theme();

    view! {
        <div class="h-screen flex flex-col bg-background text-foreground">
            <BrowsePage />
        </div>
    }
}
