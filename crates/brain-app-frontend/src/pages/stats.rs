use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::api;
use crate::components::type_icon::TypeIcon;
use crate::models::Stats;

#[component]
pub fn StatsPage() -> impl IntoView {
    let (stats, set_stats) = signal(None::<Stats>);

    spawn_local(async move {
        if let Ok(s) = api::fetch_stats().await {
            set_stats.set(Some(s));
        }
    });

    view! {
        <div class="p-6 max-w-2xl mx-auto">
            <h1 class="text-2xl font-bold mb-6">"Brain Stats"</h1>
            <Show when=move || stats.get().is_some()>
                {move || {
                    let s = stats.get().unwrap();
                    let learning = *s.by_type.get("learning").unwrap_or(&0);
                    let gotcha = *s.by_type.get("gotcha").unwrap_or(&0);
                    let project = *s.by_type.get("project_context").unwrap_or(&0);

                    view! {
                        <div class="grid grid-cols-3 gap-4 mb-8">
                            <div class="p-4 rounded-lg bg-surface text-center">
                                <div class="text-3xl font-bold text-type-learning">{learning}</div>
                                <div class="text-sm text-gray-500 mt-1">"Learnings"</div>
                            </div>
                            <div class="p-4 rounded-lg bg-surface text-center">
                                <div class="text-3xl font-bold text-type-gotcha">{gotcha}</div>
                                <div class="text-sm text-gray-500 mt-1">"Gotchas"</div>
                            </div>
                            <div class="p-4 rounded-lg bg-surface text-center">
                                <div class="text-3xl font-bold text-type-project">{project}</div>
                                <div class="text-sm text-gray-500 mt-1">"Project Contexts"</div>
                            </div>
                        </div>

                        <h2 class="text-lg font-semibold mb-3">"Recent Entries"</h2>
                        <div class="space-y-2">
                            {s.recent.into_iter().map(|entry| {
                                let entry_type = entry.entry_type.clone();
                                view! {
                                    <div class="p-3 rounded-lg bg-surface flex items-center gap-3">
                                        <TypeIcon entry_type=entry_type />
                                        <div class="flex-1 min-w-0">
                                            <div class="text-sm font-medium truncate">{entry.title.clone()}</div>
                                            <div class="text-xs text-gray-400">{entry.created_at.clone()}</div>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>
                    }
                }}
            </Show>
        </div>
    }
}
