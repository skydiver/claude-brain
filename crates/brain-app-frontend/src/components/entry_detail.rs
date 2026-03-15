use leptos::prelude::*;

use crate::components::tag_badge::TagBadge;
use crate::components::type_icon::TypeIcon;
use crate::components::ui::scroll_area::ScrollArea;
use crate::components::ui::separator::Separator;
use crate::markdown::render_markdown;
use crate::models::Entry;

#[component]
pub fn EntryDetail(
    entry: ReadSignal<Option<Entry>>,
    on_tag_click: Callback<String>,
    on_tech_click: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="flex-1 bg-background">
            <ScrollArea class="h-full">
                <Show
                    when=move || entry.get().is_some()
                    fallback=|| view! {
                        <div class="flex items-center justify-center h-full text-muted-foreground">
                            <div class="text-center">
                                <p class="text-lg">"Select an entry"</p>
                                <p class="text-sm mt-1">"Choose from the list to view its contents"</p>
                            </div>
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
                            <div class="p-6">
                                // Title
                                <h1 class="text-xl font-bold mb-3 text-foreground">{e.title.clone()}</h1>

                                // Metadata row
                                <div class="flex flex-wrap items-center gap-2 mb-3">
                                    <TypeIcon entry_type=entry_type />
                                    {tech.map(|t| {
                                        let t_clone = t.clone();
                                        view! {
                                            <span
                                                class="text-sm text-primary cursor-pointer hover:underline"
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
                                    <p class="text-xs text-muted-foreground mb-3 font-mono bg-muted px-2 py-1 rounded-md w-fit">{p}</p>
                                })}

                                <Separator />

                                // Rendered content
                                <div
                                    class="prose prose-sm dark:prose-invert max-w-none mt-4"
                                    inner_html=rendered_content
                                />

                                // Timestamps
                                <div class="mt-6 pt-4 border-t border-border text-xs text-muted-foreground flex gap-4">
                                    <span>"Created: " {e.created_at.clone()}</span>
                                    <span>"Updated: " {e.updated_at.clone()}</span>
                                </div>
                            </div>
                        }
                    }}
                </Show>
            </ScrollArea>
        </div>
    }
}
