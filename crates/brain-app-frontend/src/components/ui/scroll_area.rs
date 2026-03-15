use leptos::prelude::*;
use tw_merge::tw_merge;

#[component]
pub fn ScrollArea(children: Children, #[prop(into, optional)] class: String) -> impl IntoView {
    let merged_class = tw_merge!("relative overflow-hidden", class);

    view! {
        <div data-name="ScrollArea" class=merged_class>
            <div class="size-full rounded-[inherit] overflow-auto">
                {children()}
            </div>
        </div>
    }
}
