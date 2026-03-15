use leptos::prelude::*;

use crate::components::ui::checkbox::Checkbox;
use crate::components::ui::scroll_area::ScrollArea;
use crate::components::ui::separator::Separator;

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
        <aside class="w-[200px] min-w-[200px] border-r border-border bg-card h-full">
            <ScrollArea class="h-full">
                <div class="p-4 space-y-4">
                    // Type filter
                    <div>
                        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2">"Type"</h3>
                        <div class="space-y-1">
                            {type_options.into_iter().map(|(label, value)| {
                                let value_clone = value.clone();
                                let value_clone2 = value.clone();
                                let value_for_click = value.clone();
                                view! {
                                    <button
                                        class=move || {
                                            let selected = selected_type.get() == value_clone;
                                            format!(
                                                "flex items-center gap-2 w-full text-sm px-2 py-1 rounded-md transition-colors {}",
                                                if selected {
                                                    "bg-primary/10 text-primary font-medium"
                                                } else {
                                                    "text-foreground hover:bg-muted"
                                                }
                                            )
                                        }
                                        on:click=move |_| set_selected_type.set(value_for_click.clone())
                                    >
                                        <span class=move || {
                                            let selected = selected_type.get() == value_clone2;
                                            format!(
                                                "size-2 rounded-full {}",
                                                if selected { "bg-primary" } else { "bg-muted-foreground/30" }
                                            )
                                        }></span>
                                        {label}
                                    </button>
                                }
                            }).collect_view()}
                        </div>
                    </div>

                    <Separator />

                    // Technology filter (single-select)
                    <div>
                        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2">"Technology"</h3>
                        <div class="space-y-1">
                            <Show when=move || technologies.get().is_empty()>
                                <p class="text-xs text-muted-foreground">"None"</p>
                            </Show>
                            <For
                                each=move || technologies.get()
                                key=|tech| tech.clone()
                                children=move |tech: String| {
                                    let tech_clone = tech.clone();
                                    let tech_clone2 = tech.clone();
                                    let tech_for_click = tech.clone();
                                    view! {
                                        <button
                                            class=move || {
                                                let selected = selected_technology.get().as_deref() == Some(&tech_clone);
                                                format!(
                                                    "flex items-center gap-2 w-full text-sm px-2 py-1 rounded-md transition-colors {}",
                                                    if selected {
                                                        "bg-primary/10 text-primary font-medium"
                                                    } else {
                                                        "text-foreground hover:bg-muted"
                                                    }
                                                )
                                            }
                                            on:click=move |_| {
                                                let current = selected_technology.get();
                                                if current.as_deref() == Some(&tech_for_click) {
                                                    set_selected_technology.set(None);
                                                } else {
                                                    set_selected_technology.set(Some(tech_for_click.clone()));
                                                }
                                            }
                                        >
                                            <span class=move || {
                                                let selected = selected_technology.get().as_deref() == Some(&tech_clone2);
                                                format!(
                                                    "size-2 rounded-full {}",
                                                    if selected { "bg-primary" } else { "bg-muted-foreground/30" }
                                                )
                                            }></span>
                                            {tech.clone()}
                                        </button>
                                    }
                                }
                            />
                        </div>
                    </div>

                    <Separator />

                    // Tags filter
                    <div>
                        <h3 class="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2">"Tags"</h3>
                        <div class="space-y-1">
                            <Show when=move || tags.get().is_empty()>
                                <p class="text-xs text-muted-foreground">"None"</p>
                            </Show>
                            <For
                                each=move || tags.get()
                                key=|tag| tag.clone()
                                children=move |tag: String| {
                                    let tag_clone = tag.clone();
                                    let is_selected = move || selected_tags.get().contains(&tag_clone);
                                    let tag_for_click = tag.clone();
                                    view! {
                                        <label class="flex items-center gap-2 text-sm cursor-pointer px-2 py-0.5 rounded-md hover:bg-muted transition-colors">
                                            <Checkbox
                                                checked=Signal::derive(is_selected)
                                                on_checked_change=Callback::new(move |_checked: bool| {
                                                    let mut current = selected_tags.get();
                                                    if current.contains(&tag_for_click) {
                                                        current.retain(|t| t != &tag_for_click);
                                                    } else {
                                                        current.push(tag_for_click.clone());
                                                    }
                                                    set_selected_tags.set(current);
                                                })
                                            />
                                            {tag.clone()}
                                        </label>
                                    }
                                }
                            />
                        </div>
                    </div>
                </div>
            </ScrollArea>
        </aside>
    }
}
