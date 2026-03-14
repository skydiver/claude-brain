use serde::Serialize;
use tauri::State;

use brain_core::models::{Entry, EntryType, ListFilter, Stats};

use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub entries: Vec<Entry>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct FtsResponse {
    pub entries: Vec<FtsEntry>,
}

#[derive(Debug, Serialize)]
pub struct FtsEntry {
    #[serde(flatten)]
    pub entry: Entry,
    pub rank: f64,
}

fn entry_type_from_str(s: &str) -> Option<EntryType> {
    match s {
        "learning" => Some(EntryType::Learning),
        "project_context" => Some(EntryType::ProjectContext),
        "gotcha" => Some(EntryType::Gotcha),
        _ => None,
    }
}

#[tauri::command]
pub fn list_entries(
    state: State<'_, AppState>,
    entry_type: Option<String>,
    technology: Option<String>,
    tags: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<SearchResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let filter = ListFilter {
        entry_type: entry_type.as_deref().and_then(entry_type_from_str),
        technology,
        tags,
        limit,
        offset,
    };
    let (entries, total) = db.list_entries(&filter).map_err(|e| e.to_string())?;
    Ok(SearchResponse {
        entries,
        total: total as usize,
    })
}

#[tauri::command]
pub fn search_entries(
    state: State<'_, AppState>,
    query: String,
    entry_type: Option<String>,
    technology: Option<String>,
    project: Option<String>,
    limit: Option<u32>,
) -> Result<FtsResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let results = db
        .search_entries(
            &query,
            entry_type.as_deref().and_then(entry_type_from_str).as_ref(),
            technology.as_deref(),
            project.as_deref(),
            limit.unwrap_or(20),
        )
        .map_err(|e| e.to_string())?;
    let entries = results
        .into_iter()
        .map(|r| FtsEntry {
            entry: r.entry,
            rank: r.rank,
        })
        .collect();
    Ok(FtsResponse { entries })
}

#[tauri::command]
pub fn get_entry(state: State<'_, AppState>, id: i64) -> Result<Entry, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_entry(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_project_context(
    state: State<'_, AppState>,
    project_path: String,
) -> Result<SearchResponse, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let entries = db
        .get_project_context(&project_path)
        .map_err(|e| e.to_string())?;
    let total = entries.len();
    Ok(SearchResponse { entries, total })
}

#[tauri::command]
pub fn list_technologies(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_technologies().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_tags(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stats(state: State<'_, AppState>) -> Result<Stats, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.stats().map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use brain_core::db::Database;
    use brain_core::models::{EntryType, NewEntry};

    fn test_db() -> Database {
        let db = Database::open_in_memory().unwrap();
        let entry = NewEntry {
            entry_type: EntryType::Learning,
            title: "Test FTS5 triggers".to_string(),
            content: "SQLite FTS5 needs explicit sync triggers".to_string(),
            technology: Some("sqlite".to_string()),
            project: None,
            tags: Some("fts5,search".to_string()),
            source: Some("session".to_string()),
        };
        db.store_entry(&entry).unwrap();
        let entry2 = NewEntry {
            entry_type: EntryType::Gotcha,
            title: "WKWebView sandbox".to_string(),
            content: "Needs network.client entitlement".to_string(),
            technology: Some("swift".to_string()),
            project: Some("/projects/my-app".to_string()),
            tags: Some("macos,sandbox".to_string()),
            source: Some("session".to_string()),
        };
        db.store_entry(&entry2).unwrap();
        db
    }

    #[test]
    fn test_list_entries_no_filter() {
        let db = test_db();
        let (entries, total) = db.list_entries(&Default::default()).unwrap();
        assert_eq!(total, 2);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_list_entries_by_type() {
        let db = test_db();
        let filter = brain_core::models::ListFilter {
            entry_type: Some(EntryType::Learning),
            ..Default::default()
        };
        let (entries, total) = db.list_entries(&filter).unwrap();
        assert_eq!(total, 1);
        assert_eq!(entries[0].title, "Test FTS5 triggers");
    }

    #[test]
    fn test_search_entries() {
        let db = test_db();
        let results = db.search_entries("FTS5", None, None, None, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.title, "Test FTS5 triggers");
    }

    #[test]
    fn test_get_entry() {
        let db = test_db();
        let entry = db.get_entry(1).unwrap();
        assert_eq!(entry.title, "Test FTS5 triggers");
    }

    #[test]
    fn test_list_technologies() {
        let db = test_db();
        let techs = db.list_technologies().unwrap();
        assert_eq!(techs, vec!["sqlite", "swift"]);
    }

    #[test]
    fn test_list_tags() {
        let db = test_db();
        let tags = db.list_tags().unwrap();
        assert!(tags.contains(&"fts5".to_string()));
        assert!(tags.contains(&"macos".to_string()));
    }

    #[test]
    fn test_stats() {
        let db = test_db();
        let stats = db.stats().unwrap();
        assert_eq!(stats.total, 2);
        assert_eq!(*stats.by_type.get("learning").unwrap(), 1);
        assert_eq!(*stats.by_type.get("gotcha").unwrap(), 1);
    }
}
