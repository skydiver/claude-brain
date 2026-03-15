use leptos::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::models::EntryType;

#[component]
pub fn TypeIcon(entry_type: EntryType) -> impl IntoView {
    let (label, variant) = match entry_type {
        EntryType::Learning => ("Learning", BadgeVariant::Info),
        EntryType::Gotcha => ("Gotcha", BadgeVariant::Warning),
        EntryType::ProjectContext => ("Project", BadgeVariant::Success),
    };

    view! {
        <Badge variant=variant>{label}</Badge>
    }
}
