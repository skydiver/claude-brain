use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    #[default]
    System,
    DefaultDark,
    DefaultLight,
    SolarizedDark,
    Nord,
    CatppuccinMocha,
    Dracula,
    TokyoNight,
    Cobalt2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceSettings {
    #[serde(default)]
    pub theme: Theme,
    #[serde(default = "default_true")]
    pub filters_sidebar_visible: bool,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: Theme::default(),
            filters_sidebar_visible: true,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub appearance: AppearanceSettings,
}

/// Returns the path to the settings file: ~/.config/claude-brain/settings.json
pub fn settings_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("claude-brain")
        .join("settings.json")
}

/// Reads settings from disk. Returns defaults if file is missing or corrupted.
/// Creates the file with defaults if it doesn't exist.
pub fn load_settings() -> Settings {
    let path = settings_path();
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_else(|e| {
            eprintln!("Warning: corrupted settings file, using defaults: {e}");
            let defaults = Settings::default();
            let _ = save_settings(&defaults);
            defaults
        }),
        Err(_) => {
            let defaults = Settings::default();
            let _ = save_settings(&defaults);
            defaults
        }
    }
}

/// Writes settings to disk. Creates parent directories if needed.
pub fn save_settings(settings: &Settings) -> Result<(), String> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create settings directory: {e}"))?;
    }
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| format!("Failed to write settings: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.appearance.theme, Theme::System);
        assert!(settings.appearance.filters_sidebar_visible);
    }

    #[test]
    fn test_theme_serialization() {
        let theme = Theme::DefaultDark;
        let json = serde_json::to_string(&theme).unwrap();
        assert_eq!(json, "\"default-dark\"");

        let parsed: Theme = serde_json::from_str("\"default-light\"").unwrap();
        assert_eq!(parsed, Theme::DefaultLight);
    }

    #[test]
    fn test_settings_roundtrip() {
        let settings = Settings {
            appearance: AppearanceSettings {
                theme: Theme::DefaultDark,
                filters_sidebar_visible: false,
            },
        };
        let json = serde_json::to_string(&settings).unwrap();
        let parsed: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.appearance.theme, Theme::DefaultDark);
        assert!(!parsed.appearance.filters_sidebar_visible);
    }

    #[test]
    fn test_missing_fields_get_defaults() {
        let json = r#"{"appearance": {"theme": "default-dark"}}"#;
        let settings: Settings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.appearance.theme, Theme::DefaultDark);
        assert!(settings.appearance.filters_sidebar_visible);
    }

    #[test]
    fn test_empty_json_gets_defaults() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.appearance.theme, Theme::System);
        assert!(settings.appearance.filters_sidebar_visible);
    }

    #[test]
    fn test_corrupted_json_is_error() {
        let result: Result<Settings, _> = serde_json::from_str("not json at all");
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");

        let settings = Settings {
            appearance: AppearanceSettings {
                theme: Theme::DefaultLight,
                filters_sidebar_visible: false,
            },
        };

        let json = serde_json::to_string_pretty(&settings).unwrap();
        fs::write(&path, &json).unwrap();

        let loaded: Settings = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(loaded.appearance.theme, Theme::DefaultLight);
        assert!(!loaded.appearance.filters_sidebar_visible);
    }
}
