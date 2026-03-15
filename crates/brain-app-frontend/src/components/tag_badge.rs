use leptos::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};

#[component]
pub fn TagBadge(
    tag: String,
    #[prop(optional)] on_click: Option<Callback<String>>,
) -> impl IntoView {
    let tag_for_click = tag.clone();
    let tag_for_display = tag.clone();
    let on_click_cb = on_click.clone();
    let clickable = on_click.is_some();

    view! {
        <span
            class=move || if clickable { "cursor-pointer" } else { "" }
            on:click=move |_| {
                if let Some(ref cb) = on_click_cb {
                    cb.run(tag_for_click.clone());
                }
            }
        >
            <Badge variant=BadgeVariant::Outline>{tag_for_display}</Badge>
        </span>
    }
}
