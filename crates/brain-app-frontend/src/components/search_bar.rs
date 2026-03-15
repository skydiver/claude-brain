use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use leptos_icons::Icon;

#[component]
pub fn SearchBar(value: ReadSignal<String>, on_search: Callback<String>) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let (local_value, set_local_value) = signal(value.get_untracked());

    // Debounce timer
    let timeout_handle = StoredValue::new(None::<i32>);

    let on_input = move |ev: leptos::ev::Event| {
        let target: HtmlInputElement = event_target(&ev);
        let val = target.value();
        set_local_value.set(val.clone());

        // Clear existing timeout
        if let Some(handle) = timeout_handle.get_value() {
            web_sys::window().unwrap().clear_timeout_with_handle(handle);
        }

        // Set new 300ms debounce
        let cb = Closure::once(move || {
            on_search.run(val);
        });
        let handle = web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                cb.as_ref().unchecked_ref(),
                300,
            )
            .unwrap();
        cb.forget();
        timeout_handle.set_value(Some(handle));
    };

    // Clear button
    let on_clear = move |_| {
        set_local_value.set(String::new());
        on_search.run(String::new());
        if let Some(el) = input_ref.get() {
            let _ = el.focus();
        }
    };

    view! {
        <div class="relative flex items-center flex-1">
            <span class="absolute left-3 size-4 text-muted-foreground"><Icon icon=icondata::LuSearch /></span>
            <input
                node_ref=input_ref
                type="text"
                placeholder="Search entries... (Cmd+K)"
                class="flex h-9 w-full rounded-md border border-input bg-transparent pl-10 pr-8 py-1 text-sm shadow-xs transition-[color,box-shadow] outline-none placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:ring-2"
                prop:value=move || local_value.get()
                on:input=on_input
            />
            <Show when=move || !local_value.get().is_empty()>
                <Button
                    variant=ButtonVariant::Ghost
                    size=ButtonSize::Sm
                    class="absolute right-1 h-7 w-7 p-0"
                    on:click=on_clear
                >
                    "✕"
                </Button>
            </Show>
        </div>
    }
}
