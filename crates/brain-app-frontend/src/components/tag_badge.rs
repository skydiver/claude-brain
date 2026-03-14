use leptos::prelude::*;

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
            class=move || format!(
                "inline-flex items-center px-2 py-0.5 rounded text-xs bg-accent/10 text-accent {}",
                if clickable { "cursor-pointer hover:bg-accent/20" } else { "" }
            )
            on:click=move |_| {
                if let Some(ref cb) = on_click_cb {
                    cb.run(tag_for_click.clone());
                }
            }
        >
            {tag_for_display}
        </span>
    }
}
