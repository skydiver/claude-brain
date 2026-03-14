use leptos::prelude::*;

#[component]
pub fn Sidebar(
    selected_type: ReadSignal<Option<String>>,
    set_selected_type: WriteSignal<Option<String>>,
    technologies: ReadSignal<Vec<String>>,
    selected_technology: ReadSignal<Option<String>>,
    set_selected_technology: WriteSignal<Option<String>>,
    tags: ReadSignal<Vec<String>>,
    selected_tags: ReadSignal<Vec<String>>,
    set_selected_tags: WriteSignal<Vec<String>>,
) -> impl IntoView {
    let type_options = vec![
        ("All", None),
        ("Learning", Some("learning".to_string())),
        ("Gotcha", Some("gotcha".to_string())),
        ("Project Context", Some("project_context".to_string())),
    ];

    view! {
        <aside class="w-[200px] min-w-[200px] border-r border-gray-200 dark:border-gray-700 p-4 overflow-y-auto">
            // Type filter
            <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-2">"Type"</h3>
            <div class="space-y-1 mb-4">
                {type_options.into_iter().map(|(label, value)| {
                    let value_clone = value.clone();
                    let is_selected = move || selected_type.get() == value_clone;
                    let value_for_click = value.clone();
                    view! {
                        <label class="flex items-center gap-2 text-sm cursor-pointer hover:text-accent">
                            <input
                                type="radio"
                                name="entry_type"
                                class="accent-accent"
                                prop:checked=is_selected
                                on:change=move |_| set_selected_type.set(value_for_click.clone())
                            />
                            {label}
                        </label>
                    }
                }).collect_view()}
            </div>

            // Technology filter
            <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-2">"Technology"</h3>
            <div class="space-y-1 mb-4">
                <Show when=move || technologies.get().is_empty()>
                    <p class="text-xs text-gray-400">"None"</p>
                </Show>
                <For
                    each=move || technologies.get()
                    key=|tech| tech.clone()
                    children=move |tech: String| {
                        let tech_clone = tech.clone();
                        let is_selected = move || selected_technology.get().as_deref() == Some(&tech_clone);
                        let tech_for_click = tech.clone();
                        view! {
                            <label class="flex items-center gap-2 text-sm cursor-pointer hover:text-accent">
                                <input
                                    type="checkbox"
                                    class="accent-accent"
                                    prop:checked=is_selected
                                    on:change=move |_| {
                                        let current = selected_technology.get();
                                        if current.as_deref() == Some(&tech_for_click) {
                                            set_selected_technology.set(None);
                                        } else {
                                            set_selected_technology.set(Some(tech_for_click.clone()));
                                        }
                                    }
                                />
                                {tech.clone()}
                            </label>
                        }
                    }
                />
            </div>

            // Tags filter
            <h3 class="text-xs font-semibold uppercase tracking-wider text-gray-500 dark:text-gray-400 mb-2">"Tags"</h3>
            <div class="space-y-1">
                <Show when=move || tags.get().is_empty()>
                    <p class="text-xs text-gray-400">"None"</p>
                </Show>
                <For
                    each=move || tags.get()
                    key=|tag| tag.clone()
                    children=move |tag: String| {
                        let tag_clone = tag.clone();
                        let is_selected = move || selected_tags.get().contains(&tag_clone);
                        let tag_for_click = tag.clone();
                        view! {
                            <label class="flex items-center gap-2 text-sm cursor-pointer hover:text-accent">
                                <input
                                    type="checkbox"
                                    class="accent-accent"
                                    prop:checked=is_selected
                                    on:change=move |_| {
                                        let mut current = selected_tags.get();
                                        if current.contains(&tag_for_click) {
                                            current.retain(|t| t != &tag_for_click);
                                        } else {
                                            current.push(tag_for_click.clone());
                                        }
                                        set_selected_tags.set(current);
                                    }
                                />
                                {tag.clone()}
                            </label>
                        }
                    }
                />
            </div>
        </aside>
    }
}
