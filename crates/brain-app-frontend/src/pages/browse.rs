use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::entry_detail::EntryDetail;
use crate::components::entry_list::EntryList;
use crate::components::settings_dialog::SettingsDialog;
use crate::components::sidebar::Sidebar;
use crate::models::{Entry, Stats};
use crate::settings::SettingsContext;
use leptos_icons::Icon;

const PAGE_SIZE: u32 = 20;

#[component]
pub fn BrowsePage() -> impl IntoView {
    // UI state — restore sidebar visibility from settings
    let ctx = expect_context::<SettingsContext>();
    let initial_sidebar = ctx.settings.get_untracked().appearance.filters_sidebar_visible;
    let (filters_sidebar_visible, set_filters_sidebar_visible) = signal(initial_sidebar);

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
                match api::search_entries(query, entry_type, tech, Some(PAGE_SIZE)).await {
                    Ok(resp) => {
                        set_entries.set(resp.entries);
                        set_total.set(resp.total);
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
        // Track all filter signals so this re-runs when any changes
        let query = search_query.get();
        let entry_type = selected_type.get();
        let tech = selected_technology.get();
        let tag_list = selected_tags.get();

        // Reset pagination on filter change
        set_offset.set(0);

        // Fetch with current values directly (not via closure that re-reads signals)
        spawn_local(async move {
            if !query.is_empty() {
                match api::search_entries(query, entry_type, tech, Some(PAGE_SIZE)).await {
                    Ok(resp) => {
                        set_entries.set(resp.entries);
                        set_total.set(resp.total);
                    }
                    Err(_) => {
                        set_entries.set(vec![]);
                        set_total.set(0);
                    }
                }
            } else {
                let tags_str = if tag_list.is_empty() {
                    None
                } else {
                    Some(tag_list.join(","))
                };
                match api::list_entries(entry_type, tech, tags_str, Some(PAGE_SIZE), Some(0)).await {
                    Ok(resp) => {
                        set_entries.set(resp.entries);
                        set_total.set(resp.total);
                    }
                    Err(_) => {
                        set_entries.set(vec![]);
                        set_total.set(0);
                    }
                }
            }
        });
    });

    // Search callback
    let on_search = Callback::new(move |query: String| {
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

    // Refresh all data
    let on_refresh = Callback::new(move |_: ()| {
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
                    on_refresh.run(());
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
            // Titlebar (draggable, overlays native titlebar)
            {
                let titlebar_ref = NodeRef::<leptos::html::Div>::new();
                Effect::new(move || {
                    if let Some(el) = titlebar_ref.get() {
                        use wasm_bindgen::closure::Closure;
                        use wasm_bindgen::JsCast;

                        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
                            // Don't drag if clicking a button
                            if let Some(target) = e.target() {
                                if let Some(el) = target.dyn_ref::<web_sys::HtmlElement>() {
                                    if el.closest("button").ok().flatten().is_some() {
                                        return;
                                    }
                                }
                            }
                            if e.buttons() == 1 {
                                // Call Tauri's startDragging via JS interop
                                if let Some(window) = web_sys::window() {
                                    if let Ok(tauri) = js_sys::Reflect::get(&window, &"__TAURI__".into()) {
                                        if let Ok(win_mod) = js_sys::Reflect::get(&tauri, &"window".into()) {
                                            if let Ok(get_current) = js_sys::Reflect::get(&win_mod, &"getCurrentWindow".into()) {
                                                if let Ok(get_fn) = get_current.dyn_into::<js_sys::Function>() {
                                                    if let Ok(app_win) = get_fn.call0(&wasm_bindgen::JsValue::NULL) {
                                                        if let Ok(drag_fn) = js_sys::Reflect::get(&app_win, &"startDragging".into()) {
                                                            if let Ok(drag) = drag_fn.dyn_into::<js_sys::Function>() {
                                                                let _ = drag.call0(&app_win);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }) as Box<dyn FnMut(_)>);

                        let _ = el.add_event_listener_with_callback(
                            "mousedown",
                            closure.as_ref().unchecked_ref(),
                        );
                        closure.forget();
                    }
                });

                view! {
                    <div
                        node_ref=titlebar_ref
                        class="h-[38px] shrink-0 border-b border-border flex items-center pl-[90px] pr-3 select-none cursor-default"
                    >
                        <span class="text-xs font-semibold text-muted-foreground">"ClaudeBrain"</span>
                        <div class="mx-3 h-4 w-px bg-border"></div>
                        {
                            let settings_open = move || {
                                use_context::<crate::components::settings_dialog::SettingsOpen>()
                                    .map(|s| s.0.get())
                                    .unwrap_or(false)
                            };
                            view! {
                                <button
                                    class=move || format!(
                                        "p-1 rounded transition-colors mr-auto {}",
                                        if settings_open() {
                                            "text-muted-foreground opacity-20 cursor-default"
                                        } else if filters_sidebar_visible.get() {
                                            "bg-muted text-foreground"
                                        } else {
                                            "text-muted-foreground hover:bg-muted"
                                        }
                                    )
                                    title="Toggle filters"
                                    on:click=move |e: web_sys::MouseEvent| {
                                        e.stop_propagation();
                                        if settings_open() { return; }
                                        let new_state = !filters_sidebar_visible.get_untracked();
                                        set_filters_sidebar_visible.set(new_state);
                                        let mut settings = ctx.settings.get_untracked();
                                        settings.appearance.filters_sidebar_visible = new_state;
                                        ctx.update(settings);
                                    }
                                >
                                    <span class="size-3.5"><Icon icon=icondata::LuPanelLeft /></span>
                                </button>
                                <button
                                    class=move || format!(
                                        "p-1 rounded transition-colors {}",
                                        if settings_open() {
                                            "text-muted-foreground opacity-20 cursor-default"
                                        } else {
                                            "text-muted-foreground hover:bg-muted"
                                        }
                                    )
                                    title="Refresh"
                                    on:click=move |e: web_sys::MouseEvent| {
                                        e.stop_propagation();
                                        if settings_open() { return; }
                                        on_refresh.run(());
                                    }
                                >
                                    <span class="size-3.5"><Icon icon=icondata::LuRefreshCw /></span>
                                </button>
                            }
                        }
                        <SettingsDialog />
                    </div>
                }
            }

            // Three-pane layout
            <div class="flex flex-1 min-h-0">
                <div class=move || format!(
                    "transition-all duration-300 ease-in-out overflow-hidden {}",
                    if filters_sidebar_visible.get() { "w-[200px] min-w-[200px]" } else { "w-0 min-w-0" }
                )>
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
                </div>
                <EntryList
                    entries=entries
                    total=total
                    selected_id=selected_id
                    on_select=on_select_entry
                    on_load_more=on_load_more
                    search_value=search_query
                    on_search=on_search
                />
                <EntryDetail
                    entry=selected_entry
                    on_tag_click=on_tag_click
                    on_tech_click=on_tech_click
                />
            </div>

            // Bottom stats bar
            <div class="px-4 py-1.5 border-t border-border text-xs text-muted-foreground flex justify-between bg-card">
                <span>{stats_text}</span>
                <span class="font-mono">"brain.db"</span>
            </div>
        </div>
    }
}
