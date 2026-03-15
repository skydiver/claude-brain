use leptos::prelude::*;
use tw_merge::tw_merge;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[component]
pub fn ScrollArea(children: Children, #[prop(into, optional)] class: String) -> impl IntoView {
    let merged_class = tw_merge!("relative overflow-hidden", class);
    let viewport_ref = NodeRef::<leptos::html::Div>::new();
    let (show_top, set_show_top) = signal(false);
    let (show_bottom, set_show_bottom) = signal(false);

    Effect::new(move || {
        if let Some(el) = viewport_ref.get() {
            let el_clone = el.clone();
            // Check initial state
            let can_scroll = el.scroll_height() > el.client_height();
            set_show_bottom.set(can_scroll);

            let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
                let scroll_top = el_clone.scroll_top();
                let scroll_height = el_clone.scroll_height();
                let client_height = el_clone.client_height();

                set_show_top.set(scroll_top > 0);
                set_show_bottom.set(scroll_top + client_height < scroll_height - 1);
            }) as Box<dyn FnMut(_)>);

            let _ = el.add_event_listener_with_callback("scroll", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });

    view! {
        <div data-name="ScrollArea" class=merged_class>
            // Top shadow
            <div class=move || format!(
                "absolute top-0 left-0 right-0 h-6 bg-gradient-to-b from-black/40 to-transparent z-10 pointer-events-none transition-opacity duration-200 {}",
                if show_top.get() { "opacity-100" } else { "opacity-0" }
            )></div>
            <div node_ref=viewport_ref class="size-full rounded-[inherit] overflow-auto">
                {children()}
            </div>
            // Bottom shadow
            <div class=move || format!(
                "absolute bottom-0 left-0 right-0 h-6 bg-gradient-to-t from-black/40 to-transparent z-10 pointer-events-none transition-opacity duration-200 {}",
                if show_bottom.get() { "opacity-100" } else { "opacity-0" }
            )></div>
        </div>
    }
}
