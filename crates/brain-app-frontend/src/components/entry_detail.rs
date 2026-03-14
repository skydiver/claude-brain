use leptos::prelude::*;

use crate::components::tag_badge::TagBadge;
use crate::components::type_icon::TypeIcon;
use crate::markdown::render_markdown;
use crate::models::Entry;

#[component]
pub fn EntryDetail(
    entry: ReadSignal<Option<Entry>>,
    on_tag_click: Callback<String>,
    on_tech_click: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="flex-1 overflow-y-auto p-6">
            <Show
                when=move || entry.get().is_some()
                fallback=|| view! {
                    <div class="flex items-center justify-center h-full text-gray-400">
                        <p>"Select an entry to view its contents"</p>
                    </div>
                }
            >
                {move || {
                    let e = entry.get().unwrap();
                    let tags = e.tags_list().into_iter().map(String::from).collect::<Vec<_>>();
                    let rendered_content = render_markdown(&e.content);
                    let entry_type = e.entry_type.clone();
                    let tech = e.technology.clone();
                    let project = e.project.clone();

                    view! {
                        <div>
                            // Title
                            <h1 class="text-xl font-bold mb-3">{e.title.clone()}</h1>

                            // Metadata row
                            <div class="flex flex-wrap items-center gap-2 mb-3">
                                <TypeIcon entry_type=entry_type />
                                {tech.map(|t| {
                                    let t_clone = t.clone();
                                    view! {
                                        <span
                                            class="text-sm text-accent cursor-pointer hover:underline"
                                            on:click=move |_| on_tech_click.run(t_clone.clone())
                                        >
                                            {t.clone()}
                                        </span>
                                    }
                                })}
                                {tags.into_iter().map(|tag| {
                                    let cb = on_tag_click.clone();
                                    view! { <TagBadge tag=tag.clone() on_click=cb /> }
                                }).collect_view()}
                            </div>

                            // Project path (if any)
                            {project.map(|p| view! {
                                <p class="text-xs text-gray-400 mb-3 font-mono">{p}</p>
                            })}

                            // Separator
                            <hr class="border-gray-200 dark:border-gray-700 mb-4" />

                            // Rendered content
                            <div
                                class="prose prose-sm dark:prose-invert max-w-none"
                                inner_html=rendered_content
                            />

                            // Timestamps
                            <div class="mt-6 pt-4 border-t border-gray-200 dark:border-gray-700 text-xs text-gray-400 flex gap-4">
                                <span>"Created: " {e.created_at.clone()}</span>
                                <span>"Updated: " {e.updated_at.clone()}</span>
                            </div>
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}
