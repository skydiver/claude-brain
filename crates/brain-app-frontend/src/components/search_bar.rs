use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;

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
        <div class="relative flex items-center">
            <svg class="absolute left-3 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"/>
            </svg>
            <input
                node_ref=input_ref
                type="text"
                placeholder="Search entries... (Cmd+K)"
                class="w-full pl-10 pr-8 py-2 rounded-lg border border-gray-200 dark:border-gray-700 bg-white dark:bg-[#16213e] focus:outline-none focus:ring-2 focus:ring-accent/50 text-sm"
                prop:value=move || local_value.get()
                on:input=on_input
            />
            <Show when=move || !local_value.get().is_empty()>
                <button
                    class="absolute right-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                    on:click=on_clear
                >
                    "✕"
                </button>
            </Show>
        </div>
    }
}
