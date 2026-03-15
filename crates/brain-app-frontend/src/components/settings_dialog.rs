use leptos::prelude::*;
use leptos_icons::Icon;

use crate::components::ui::button::ButtonVariant;
use crate::components::ui::dialog::{Dialog, DialogBody, DialogContent, DialogTrigger};
use crate::components::ui::select::{
    Select, SelectContent, SelectGroup, SelectOption, SelectTrigger, SelectValue,
};
use crate::components::ui::separator::Separator;
use crate::components::ui::switch::Switch;
use crate::models::Theme;
use crate::settings::SettingsContext;

#[component]
pub fn SettingsDialog() -> impl IntoView {
    let ctx = expect_context::<SettingsContext>();
    let (active_category, _set_active_category) = signal("appearance".to_string());

    // Theme change handler
    let on_theme_change = Callback::new(move |value: Option<String>| {
        let Some(theme_str) = value else { return };
        let theme = match theme_str.as_str() {
            "default-dark" => Theme::DefaultDark,
            "default-light" => Theme::DefaultLight,
            _ => Theme::System,
        };
        let mut settings = ctx.settings.get_untracked();
        settings.appearance.theme = theme;
        ctx.update(settings);
    });

    // Sidebar toggle handler
    let on_sidebar_toggle = Callback::new(move |checked: bool| {
        let mut settings = ctx.settings.get_untracked();
        settings.appearance.sidebar_visible = checked;
        ctx.update(settings);
    });

    let current_theme_value = move || match ctx.settings.get().appearance.theme {
        Theme::System => "system",
        Theme::DefaultDark => "default-dark",
        Theme::DefaultLight => "default-light",
    }
    .to_string();

    view! {
        <Dialog>
            <DialogTrigger
                class="p-1 rounded text-muted-foreground hover:bg-muted transition-colors [&_svg]:size-3.5 h-auto border-0 shadow-none"
                variant=ButtonVariant::Ghost
            >
                <Icon icon=icondata::LuSettings />
            </DialogTrigger>
            <DialogContent class="max-w-[600px] p-0">
                <DialogBody class="p-0 gap-0">
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
                            <h2 class="text-lg font-semibold mb-6">
                                {move || match active_category.get().as_str() {
                                    "appearance" => "Appearance",
                                    _ => "Appearance",
                                }}
                            </h2>

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
                                    <Select
                                        default_value=current_theme_value()
                                        on_change=on_theme_change
                                    >
                                        <SelectTrigger class="w-[160px]">
                                            <SelectValue placeholder="Select theme" />
                                        </SelectTrigger>
                                        <SelectContent class="w-[160px]">
                                            <SelectGroup>
                                                <SelectOption value="system">"System"</SelectOption>
                                                <SelectOption value="default-dark">
                                                    "Default Dark"
                                                </SelectOption>
                                                <SelectOption value="default-light">
                                                    "Default Light"
                                                </SelectOption>
                                            </SelectGroup>
                                        </SelectContent>
                                    </Select>
                                </div>

                                <Separator />

                                // Sidebar visible setting
                                <div class="flex items-center justify-between py-3">
                                    <div>
                                        <div class="text-sm font-medium">"Sidebar Visible"</div>
                                        <div class="text-xs text-muted-foreground">
                                            "Show the sidebar on startup"
                                        </div>
                                    </div>
                                    <Switch
                                        checked=ctx.settings.get_untracked().appearance.sidebar_visible
                                        on_change=on_sidebar_toggle
                                    />
                                </div>
                            </Show>
                        </div>
                    </div>
                </DialogBody>
            </DialogContent>
        </Dialog>
    }
}
