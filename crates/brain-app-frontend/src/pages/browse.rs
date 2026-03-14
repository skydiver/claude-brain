use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::entry_detail::EntryDetail;
use crate::components::entry_list::EntryList;
use crate::components::search_bar::SearchBar;
use crate::components::sidebar::Sidebar;
use crate::models::{Entry, Stats};

const PAGE_SIZE: u32 = 20;

#[component]
pub fn BrowsePage() -> impl IntoView {
    // Filter state
    let (selected_type, set_selected_type) = signal(None::<String>);
    let (selected_technology, set_selected_technology) = signal(None::<String>);
    let (selected_tags, set_selected_tags) = signal(Vec::<String>::new());
    let (search_query, set_search_query) = signal(String::new());

    // Data state
    let (entries, set_entries) = signal(Vec::<Entry>::new());
    let (total, set_total) = signal(0usize);
    let (selected_entry, set_selected_entry) = signal(None::<Entry>);
    let (technologies, set_technologies) = signal(Vec::<String>::new());
    let (tags, set_tags) = signal(Vec::<String>::new());
    let (app_stats, set_stats) = signal(None::<Stats>);
    let (offset, set_offset) = signal(0u32);

    // Initial data load
    spawn_local(async move {
        if let Ok(techs) = api::list_technologies().await {
            set_technologies.set(techs);
        }
        if let Ok(t) = api::list_tags().await {
            set_tags.set(t);
        }
        if let Ok(s) = api::fetch_stats().await {
            set_stats.set(Some(s));
        }
    });

    // Fetch entries based on current filters
    let fetch_entries = move || {
        let query = search_query.get();
        let entry_type = selected_type.get();
        let tech = selected_technology.get();
        let tag_list = selected_tags.get();
        let current_offset = offset.get();

        spawn_local(async move {
            if !query.is_empty() {
                // FTS search mode
                match api::search_entries(query, entry_type, tech, None, Some(PAGE_SIZE)).await {
                    Ok(resp) => {
                        let search_entries: Vec<Entry> =
                            resp.entries.into_iter().map(|fe| fe.entry).collect();
                        let len = search_entries.len();
                        set_entries.set(search_entries);
                        set_total.set(len); // FTS doesn't return a total
                    }
                    Err(_) => {
                        set_entries.set(vec![]);
                        set_total.set(0);
                    }
                }
            } else {
                // List/filter mode
                let tags_str = if tag_list.is_empty() {
                    None
                } else {
                    Some(tag_list.join(","))
                };
                match api::list_entries(entry_type, tech, tags_str, Some(PAGE_SIZE), Some(current_offset)).await {
                    Ok(resp) => {
                        if current_offset == 0 {
                            set_entries.set(resp.entries);
                        } else {
                            // Append for load-more
                            let mut current = entries.get_untracked();
                            current.extend(resp.entries);
                            set_entries.set(current);
                        }
                        set_total.set(resp.total);
                    }
                    Err(_) => {
                        set_entries.set(vec![]);
                        set_total.set(0);
                    }
                }
            }
        });
    };

    // Trigger fetch on any filter change
    Effect::new(move || {
        let _ = search_query.get();
        let _ = selected_type.get();
        let _ = selected_technology.get();
        let _ = selected_tags.get();
        set_offset.set(0);
        fetch_entries();
    });

    // Search callback
    let on_search = Callback::new(move |query: String| {
        if !query.is_empty() {
            set_selected_type.set(None);
            set_selected_technology.set(None);
            set_selected_tags.set(vec![]);
        }
        set_search_query.set(query);
    });

    // Entry selection
    let on_select_entry = Callback::new(move |id: i64| {
        spawn_local(async move {
            if let Ok(entry) = api::get_entry(id).await {
                set_selected_entry.set(Some(entry));
            }
        });
    });

    // Load more
    let on_load_more = Callback::new(move |_: ()| {
        set_offset.set(offset.get_untracked() + PAGE_SIZE);
        fetch_entries();
    });

    // Tag click from detail → add to filters
    let on_tag_click = Callback::new(move |tag: String| {
        set_search_query.set(String::new());
        let mut current = selected_tags.get_untracked();
        if !current.contains(&tag) {
            current.push(tag);
            set_selected_tags.set(current);
        }
    });

    // Tech click from detail → filter by technology
    let on_tech_click = Callback::new(move |tech: String| {
        set_search_query.set(String::new());
        set_selected_technology.set(Some(tech));
    });

    // Selected ID derived signal
    let selected_id = Memo::new(move |_| selected_entry.get().map(|e| e.id));

    // Stats bar
    let stats_text = move || {
        app_stats
            .get()
            .map(|s| {
                let learning = s.by_type.get("learning").copied().unwrap_or(0);
                let gotcha = s.by_type.get("gotcha").copied().unwrap_or(0);
                let project = s.by_type.get("project_context").copied().unwrap_or(0);
                format!("{learning} learnings · {gotcha} gotchas · {project} project contexts")
            })
            .unwrap_or_default()
    };

    // Keyboard shortcuts
    {
        let closure = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            let key = event.key();
            let meta = event.meta_key() || event.ctrl_key();

            // Cmd+K → focus search
            if meta && key == "k" {
                event.prevent_default();
                if let Some(el) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.query_selector("input[type='text']").ok())
                    .flatten()
                {
                    let _ = el.dyn_ref::<web_sys::HtmlElement>().map(|e| e.focus());
                }
                return;
            }

            // Escape → clear search and deselect
            if key == "Escape" {
                set_search_query.set(String::new());
                set_selected_entry.set(None);
                return;
            }

            // Arrow Up/Down → navigate entry list
            if key == "ArrowDown" || key == "ArrowUp" {
                event.prevent_default();
                let current_entries = entries.get_untracked();
                if current_entries.is_empty() {
                    return;
                }

                let current_id = selected_entry.get_untracked().map(|e| e.id);
                let current_idx =
                    current_id.and_then(|id| current_entries.iter().position(|e| e.id == id));

                let next_idx = match (current_idx, key.as_str()) {
                    (None, _) => 0,
                    (Some(i), "ArrowDown") => (i + 1).min(current_entries.len() - 1),
                    (Some(i), "ArrowUp") => i.saturating_sub(1),
                    _ => return,
                };

                let next_id = current_entries[next_idx].id;
                on_select_entry.run(next_id);
                return;
            }

            // Enter → select first entry if none selected
            if key == "Enter" {
                if let Some(active) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element())
                {
                    if active.tag_name() == "INPUT" {
                        return;
                    }
                }
                if selected_entry.get_untracked().is_some() {
                    return;
                }
                let current_entries = entries.get_untracked();
                if let Some(first) = current_entries.first() {
                    on_select_entry.run(first.id);
                }
            }
        }) as Box<dyn FnMut(_)>);

        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    // Refresh on window focus
    {
        let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if doc.visibility_state() == web_sys::VisibilityState::Visible {
                    fetch_entries();
                    spawn_local(async move {
                        if let Ok(techs) = api::list_technologies().await {
                            set_technologies.set(techs);
                        }
                        if let Ok(t) = api::list_tags().await {
                            set_tags.set(t);
                        }
                        if let Ok(s) = api::fetch_stats().await {
                            set_stats.set(Some(s));
                        }
                    });
                }
            }
        }) as Box<dyn FnMut(_)>);

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .add_event_listener_with_callback("visibilitychange", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    view! {
        <div class="flex flex-col h-screen">
            // Top bar: search + refresh
            <div class="p-3 border-b border-gray-200 dark:border-gray-700 flex items-center gap-2">
                <div class="flex-1">
                    <SearchBar value=search_query on_search=on_search />
                </div>
                <button
                    class="p-2 text-gray-400 hover:text-accent rounded"
                    title="Refresh"
                    on:click=move |_| {
                        fetch_entries();
                        spawn_local(async move {
                            if let Ok(techs) = api::list_technologies().await {
                                set_technologies.set(techs);
                            }
                            if let Ok(t) = api::list_tags().await {
                                set_tags.set(t);
                            }
                            if let Ok(s) = api::fetch_stats().await {
                                set_stats.set(Some(s));
                            }
                        });
                    }
                >
                    "⟳"
                </button>
            </div>

            // Three-pane layout
            <div class="flex flex-1 min-h-0">
                <Sidebar
                    selected_type=selected_type
                    set_selected_type=set_selected_type
                    technologies=technologies
                    selected_technology=selected_technology
                    set_selected_technology=set_selected_technology
                    tags=tags
                    selected_tags=selected_tags
                    set_selected_tags=set_selected_tags
                />
                <EntryList
                    entries=entries
                    total=total
                    selected_id=selected_id
                    on_select=on_select_entry
                    on_load_more=on_load_more
                />
                <EntryDetail
                    entry=selected_entry
                    on_tag_click=on_tag_click
                    on_tech_click=on_tech_click
                />
            </div>

            // Bottom stats bar
            <div class="px-4 py-1.5 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-500 flex justify-between">
                <span>{stats_text}</span>
                <span class="font-mono">"brain.db"</span>
            </div>
        </div>
    }
}
