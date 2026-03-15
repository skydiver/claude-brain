use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn Separator(#[prop(into, optional)] class: String) -> impl IntoView {
    let merged_class = tw_merge!("shrink-0 bg-border w-full h-[1px]", class);

    view! { <div data-name="Separator" class=merged_class role="separator" /> }
}
