use leptos::prelude::*;

use crate::components::type_icon::TypeIcon;
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::scroll_area::ScrollArea;
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
        <div class="w-[280px] min-w-[280px] border-r border-border flex flex-col bg-card">
            <ScrollArea class="flex-1">
                <Show
                    when=move || !entries.get().is_empty()
                    fallback=|| view! {
                        <div class="p-6 text-center text-muted-foreground text-sm">
                            <p class="font-medium">"No entries found"</p>
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
                                        "p-3 border-b border-border cursor-pointer transition-colors {}",
                                        if is_selected() { "bg-primary/10 border-l-2 border-l-primary" } else { "hover:bg-muted/50" }
                                    )
                                    on:click=move |_| on_select.run(id)
                                >
                                    <div class="flex items-center gap-2 mb-1">
                                        <TypeIcon entry_type=entry_type />
                                        <Show when=move || !tech_for_check.is_empty()>
                                            <span class="text-xs text-muted-foreground">{tech.clone()}</span>
                                        </Show>
                                    </div>
                                    <h4 class="text-sm font-medium truncate text-foreground">{entry.title.clone()}</h4>
                                    <p class="text-xs text-muted-foreground mt-0.5 line-clamp-2">{preview}</p>
                                </div>
                            }
                        }
                    />
                </Show>
            </ScrollArea>

            // Footer with count and load-more
            <div class="p-2 border-t border-border text-xs text-muted-foreground text-center">
                <span>{showing_text}</span>
                <Show when=has_more>
                    <Button
                        variant=ButtonVariant::Link
                        size=ButtonSize::Sm
                        class="ml-1 h-auto p-0 text-xs"
                        on:click=move |_| on_load_more.run(())
                    >
                        "Load more"
                    </Button>
                </Show>
            </div>
        </div>
    }
}
