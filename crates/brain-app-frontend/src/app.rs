use leptos::prelude::*;

use crate::pages::browse::BrowsePage;
use crate::theme::init_theme;

#[component]
pub fn App() -> impl IntoView {
    init_theme();

    view! {
        <div class="h-screen flex flex-col bg-white dark:bg-[#1a1a2e] text-[#1a1a2e] dark:text-[#e0e0e0]">
            <BrowsePage />
        </div>
    }
}
