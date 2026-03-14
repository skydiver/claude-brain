use leptos::prelude::*;

use crate::components::type_icon::TypeIcon;
use crate::models::Entry;

#[component]
pub fn EntryList(
    entries: ReadSignal<Vec<Entry>>,
    total: ReadSignal<usize>,
    selected_id: Memo<Option<i64>>,
    on_select: Callback<i64>,
    on_load_more: Callback<()>,
) -> impl IntoView {
    let has_more = move || entries.get().len() < total.get();
    let showing_text = move || {
        let count = entries.get().len();
        let t = total.get();
        if t == 0 {
            "No entries".to_string()
        } else {
            format!("Showing {count} of {t}")
        }
    };

    view! {
        <div class="w-[280px] min-w-[280px] border-r border-gray-200 dark:border-gray-700 flex flex-col">
            <div class="flex-1 overflow-y-auto">
                <Show
                    when=move || !entries.get().is_empty()
                    fallback=|| view! {
                        <div class="p-4 text-center text-gray-400 text-sm">
                            <p>"No entries found"</p>
                            <p class="mt-1 text-xs">"Try broadening your search or filters"</p>
                        </div>
                    }
                >
                    <For
                        each=move || entries.get()
                        key=|entry| entry.id
                        children=move |entry: Entry| {
                            let id = entry.id;
                            let is_selected = move || selected_id.get() == Some(id);
                            let preview = entry.content_preview(80);
                            let entry_type = entry.entry_type.clone();
                            let tech = entry.technology.clone().unwrap_or_default();
                            let tech_for_check = tech.clone();

                            view! {
                                <div
                                    class=move || format!(
                                        "p-3 border-b border-gray-100 dark:border-gray-800 cursor-pointer hover:bg-gray-50 dark:hover:bg-[#1a1a3e] {}",
                                        if is_selected() { "bg-accent/10" } else { "" }
                                    )
                                    on:click=move |_| on_select.run(id)
                                >
                                    <div class="flex items-center gap-2 mb-1">
                                        <TypeIcon entry_type=entry_type />
                                        <Show when=move || !tech_for_check.is_empty()>
                                            <span class="text-xs text-gray-400">{tech.clone()}</span>
                                        </Show>
                                    </div>
                                    <h4 class="text-sm font-medium truncate">{entry.title.clone()}</h4>
                                    <p class="text-xs text-gray-500 dark:text-gray-400 mt-0.5 line-clamp-2">{preview}</p>
                                </div>
                            }
                        }
                    />
                </Show>
            </div>

            // Footer with count and load-more
            <div class="p-2 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-500 text-center">
                <span>{showing_text}</span>
                <Show when=has_more>
                    <button
                        class="ml-2 text-accent hover:underline"
                        on:click=move |_| on_load_more.run(())
                    >
                        "Load more"
                    </button>
                </Show>
            </div>
        </div>
    }
}
