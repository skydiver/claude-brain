use leptos::prelude::*;
use leptos_icons::Icon;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::models::Theme;
use crate::settings::SettingsContext;

/// Signal indicating whether the settings dialog is open.
/// Provided via context so titlebar buttons can check it.
#[derive(Clone, Copy)]
pub struct SettingsOpen(pub ReadSignal<bool>);

#[component]
pub fn SettingsDialog() -> impl IntoView {
    let ctx = expect_context::<SettingsContext>();
    let (open, set_open) = signal(false);
    provide_context(SettingsOpen(open));
    let (active_category, _set_active_category) = signal("appearance".to_string());

    // Close on Escape
    {
        let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
            if e.key() == "Escape" && open.get_untracked() {
                set_open.set(false);
            }
        }) as Box<dyn FnMut(_)>);

        web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
            .unwrap();
        closure.forget();
    }

    let current_theme_value = move || match ctx.settings.get().appearance.theme {
        Theme::System => "system",
        Theme::DefaultDark => "default-dark",
        Theme::DefaultLight => "default-light",
    }
    .to_string();

    let on_theme_change = Callback::new(move |theme: Theme| {
        let mut settings = ctx.settings.get_untracked();
        settings.appearance.theme = theme;
        ctx.update(settings);
    });

    view! {
        // Trigger button — matches titlebar button style exactly
        <button
            class=move || format!(
                "p-1 rounded transition-colors {}",
                if open.get() {
                    "text-muted-foreground opacity-20 cursor-default"
                } else {
                    "text-muted-foreground hover:bg-muted"
                }
            )
            title="Settings"
            on:click=move |e: web_sys::MouseEvent| {
                e.stop_propagation();
                if !open.get_untracked() {
                    set_open.set(true);
                }
            }
        >
            <span class="size-3.5"><Icon icon=icondata::LuSettings /></span>
        </button>

        // Backdrop
        <Show when=move || open.get()>
            <div
                class="fixed inset-x-0 top-[37px] bottom-0 z-[60] bg-black/70"
                on:mousedown=move |e: web_sys::MouseEvent| {
                    e.stop_propagation();
                    e.prevent_default();
                    // Only close if mousedown is directly on the backdrop itself
                    if let Some(target) = e.target() {
                        if let Some(current) = e.current_target() {
                            if target == current {
                                set_open.set(false);
                            }
                        }
                    }
                }
            >

            // Dialog — inside backdrop to block interaction with app below
            <div
                class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 z-[100] bg-background border border-border rounded-2xl shadow-lg max-w-[600px] w-[calc(100%-2rem)]"
                on:mousedown=move |e: web_sys::MouseEvent| e.stop_propagation()
            >
                <div class="flex min-h-[400px]">
                    // Category sidebar
                    <div class="w-[180px] border-r border-border p-2 space-y-0.5">
                        <button class=move || {
                            format!(
                                "w-full flex items-center gap-2 px-3 py-2 rounded-md text-sm transition-colors {}",
                                if active_category.get() == "appearance" {
                                    "bg-muted text-foreground"
                                } else {
                                    "text-muted-foreground hover:bg-muted/50 hover:text-foreground"
                                },
                            )
                        }>
                            <span class="size-4">
                                <Icon icon=icondata::LuPalette />
                            </span>
                            "Appearance"
                        </button>
                    </div>

                    // Content panel
                    <div class="flex-1 p-6">
                        <div class="flex justify-between items-center mb-6">
                            <h2 class="text-lg font-semibold">
                                {move || match active_category.get().as_str() {
                                    "appearance" => "Appearance",
                                    _ => "Appearance",
                                }}
                            </h2>
                            <button
                                class="p-1 rounded-sm text-muted-foreground hover:text-foreground transition-colors"
                                on:click=move |_| set_open.set(false)
                            >
                                <span class="size-4"><Icon icon=icondata::LuX /></span>
                            </button>
                        </div>

                        // Appearance settings
                        <Show when=move || active_category.get() == "appearance">
                            // Theme setting
                            <div class="flex items-center justify-between py-3">
                                <div>
                                    <div class="text-sm font-medium">"Theme"</div>
                                    <div class="text-xs text-muted-foreground">
                                        "Choose the app color scheme"
                                    </div>
                                </div>
                                <ThemeSelect
                                    value=Signal::derive(current_theme_value)
                                    on_change=on_theme_change
                                />
                            </div>

                        </Show>
                    </div>
                </div>
            </div>
            </div>
        </Show>
    }
}

#[component]
fn ThemeSelect(
    value: Signal<String>,
    on_change: Callback<Theme>,
) -> impl IntoView {
    let (dropdown_open, set_dropdown_open) = signal(false);

    let display_label = move || match value.get().as_str() {
        "default-dark" => "Default Dark",
        "default-light" => "Default Light",
        _ => "System",
    };

    let make_option = move |val: &'static str, label: &'static str, theme: Theme| {
        let is_selected = move || value.get() == val;
        view! {
            <button
                type="button"
                class=move || format!(
                    "w-full flex items-center gap-2 px-2 py-1.5 text-sm rounded-sm transition-colors {}",
                    if is_selected() {
                        "bg-accent text-accent-foreground"
                    } else {
                        "text-popover-foreground hover:bg-accent hover:text-accent-foreground"
                    }
                )
                on:click={
                    let theme = theme.clone();
                    move |e: web_sys::MouseEvent| {
                        e.stop_propagation();
                        on_change.run(theme.clone());
                        set_dropdown_open.set(false);
                    }
                }
            >
                <span class=move || if is_selected() { "size-4" } else { "size-4 opacity-0" }>
                    <Icon icon=icondata::LuCheck />
                </span>
                {label}
            </button>
        }
    };

    view! {
        <div class="relative w-[160px]">
            <button
                type="button"
                class="w-full h-9 px-3 inline-flex items-center justify-between text-sm rounded-md border border-input bg-background hover:bg-accent hover:text-accent-foreground transition-colors"
                on:click=move |e: web_sys::MouseEvent| {
                    e.stop_propagation();
                    set_dropdown_open.update(|v| *v = !*v);
                }
            >
                <span class="truncate">{display_label}</span>
                <span class="size-4 text-muted-foreground">
                    <Icon icon=icondata::LuChevronDown />
                </span>
            </button>

            <Show when=move || dropdown_open.get()>
                <div
                    class="fixed inset-0 z-[90]"
                    on:mousedown=move |_| set_dropdown_open.set(false)
                />
                <div class="absolute top-[calc(100%+4px)] left-0 w-full z-[100] p-1 rounded-md border border-border bg-card shadow-md">
                    {make_option("system", "System", Theme::System)}
                    {make_option("default-dark", "Default Dark", Theme::DefaultDark)}
                    {make_option("default-light", "Default Light", Theme::DefaultLight)}
                </div>
            </Show>
        </div>
    }
}
