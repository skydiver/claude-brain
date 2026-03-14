#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod state;

use brain_core::db::Database;
use state::AppState;

fn resolve_db_path() -> String {
    if let Ok(path) = std::env::var("BRAIN_DB_PATH") {
        return path;
    }
    let config_dir = dirs::config_dir()
        .expect("Could not determine config directory")
        .join("claude-brain");
    config_dir.join("brain.db").to_string_lossy().to_string()
}

fn main() {
    let db_path = resolve_db_path();
    let db =
        Database::open(&db_path).unwrap_or_else(|_| panic!("Failed to open database at {db_path}"));
    let app_state = AppState::new(db);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::list_entries,
            commands::search_entries,
            commands::get_entry,
            commands::get_project_context,
            commands::list_technologies,
            commands::list_tags,
            commands::stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
