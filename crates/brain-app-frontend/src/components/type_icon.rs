use leptos::prelude::*;

use crate::models::EntryType;

#[component]
pub fn TypeIcon(entry_type: EntryType) -> impl IntoView {
    let (label, color_class) = match entry_type {
        EntryType::Learning => ("Learning", "bg-type-learning/20 text-type-learning"),
        EntryType::Gotcha => ("Gotcha", "bg-type-gotcha/20 text-type-gotcha"),
        EntryType::ProjectContext => ("Project", "bg-type-project/20 text-type-project"),
    };

    view! {
        <span class=format!("inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {color_class}")>
            {label}
        </span>
    }
}
