use rusqlite::{Connection, params};
use crate::error::BrainError;
use crate::models::*;
use crate::search::sanitize_fts_query;

const SCHEMA_V1: &str = r#"
CREATE TABLE IF NOT EXISTS schema_version (
    version INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS entries (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    type        TEXT NOT NULL CHECK(type IN ('learning', 'project_context', 'gotcha')),
    title       TEXT NOT NULL,
    content     TEXT NOT NULL,
    technology  TEXT,
    project     TEXT,
    tags        TEXT,
    source      TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
    title, content, technology, tags,
    content='entries',
    content_rowid='id'
);

CREATE INDEX IF NOT EXISTS idx_entries_project ON entries(project);
CREATE INDEX IF NOT EXISTS idx_entries_type ON entries(type);
CREATE INDEX IF NOT EXISTS idx_entries_technology ON entries(technology);

CREATE TRIGGER IF NOT EXISTS entries_ai AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts(rowid, title, content, technology, tags)
    VALUES (new.id, new.title, new.content, new.technology, new.tags);
END;

CREATE TRIGGER IF NOT EXISTS entries_ad AFTER DELETE ON entries BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, title, content, technology, tags)
    VALUES ('delete', old.id, old.title, old.content, old.technology, old.tags);
END;

CREATE TRIGGER IF NOT EXISTS entries_au AFTER UPDATE ON entries BEGIN
    INSERT INTO entries_fts(entries_fts, rowid, title, content, technology, tags)
    VALUES ('delete', old.id, old.title, old.content, old.technology, old.tags);
    INSERT INTO entries_fts(rowid, title, content, technology, tags)
    VALUES (new.id, new.title, new.content, new.technology, new.tags);
END;
"#;

#[derive(Debug)]
pub struct Database {
    pub(crate) conn: Connection,
}

impl Database {
    pub fn open(path: &str) -> Result<Self, BrainError> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self, BrainError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub(crate) fn migrate(&self) -> Result<(), BrainError> {
        let version = self.current_version();
        if version < 1 {
            self.conn
                .execute_batch(SCHEMA_V1)
                .map_err(|e| BrainError::Migration(e.to_string()))?;
            self.conn
                .execute("INSERT INTO schema_version (version) VALUES (?1)", params![1])?;
        }
        Ok(())
    }

    fn current_version(&self) -> i64 {
        self.conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0)
    }

    // --- CRUD ---

    pub fn store_entry(&self, entry: &NewEntry) -> Result<i64, BrainError> {
        // Check for duplicates
        let existing: Option<i64> = self.conn.query_row(
            "SELECT id FROM entries WHERE title = ?1 AND type = ?2 AND COALESCE(project, '') = COALESCE(?3, '')",
            params![entry.title, entry.entry_type.as_str(), entry.project],
            |r| r.get(0),
        ).ok();

        if let Some(existing_id) = existing {
            return Err(BrainError::Duplicate {
                existing_id,
                title: entry.title.clone(),
                entry_type: entry.entry_type.as_str().to_string(),
            });
        }

        let tags = entry.tags.as_deref().map(normalize_tags);

        self.conn.execute(
            "INSERT INTO entries (type, title, content, technology, project, tags, source)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.entry_type.as_str(),
                entry.title,
                entry.content,
                entry.technology,
                entry.project,
                tags,
                entry.source,
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_entry(&self, id: i64) -> Result<Entry, BrainError> {
        self.conn
            .query_row(
                "SELECT id, type, title, content, technology, project, tags, source, created_at, updated_at
                 FROM entries WHERE id = ?1",
                params![id],
                |row| Ok(row_to_entry(row)),
            )
            .map_err(|_| BrainError::NotFound(id))
    }

    pub fn update_entry(&self, id: i64, update: &UpdateEntry) -> Result<Entry, BrainError> {
        // Verify entry exists
        self.get_entry(id)?;

        let mut sets = Vec::new();
        let mut values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref title) = update.title {
            sets.push("title = ?");
            values.push(Box::new(title.clone()));
        }
        if let Some(ref content) = update.content {
            sets.push("content = ?");
            values.push(Box::new(content.clone()));
        }
        if let Some(ref technology) = update.technology {
            sets.push("technology = ?");
            values.push(Box::new(technology.clone()));
        }
        if let Some(ref project) = update.project {
            sets.push("project = ?");
            values.push(Box::new(project.clone()));
        }
        if let Some(ref tags) = update.tags {
            sets.push("tags = ?");
            values.push(Box::new(normalize_tags(tags)));
        }

        if !sets.is_empty() {
            sets.push("updated_at = datetime('now')");
            let sql = format!("UPDATE entries SET {} WHERE id = ?", sets.join(", "));
            values.push(Box::new(id));
            let params: Vec<&dyn rusqlite::types::ToSql> = values.iter().map(|v| v.as_ref()).collect();
            self.conn.execute(&sql, params.as_slice())?;
        }

        self.get_entry(id)
    }

    pub fn delete_entry(&self, id: i64) -> Result<(), BrainError> {
        let changes = self
            .conn
            .execute("DELETE FROM entries WHERE id = ?1", params![id])?;
        if changes == 0 {
            return Err(BrainError::NotFound(id));
        }
        Ok(())
    }

    // --- Search & Query ---

    pub fn search_entries(
        &self,
        query: &str,
        entry_type: Option<&EntryType>,
        technology: Option<&str>,
        project: Option<&str>,
        limit: u32,
    ) -> Result<Vec<SearchResult>, BrainError> {
        let sanitized = sanitize_fts_query(query);
        if sanitized.is_empty() {
            return Ok(Vec::new());
        }
        let mut sql = String::from(
            "SELECT e.id, e.type, e.title, e.content, e.technology, e.project, e.tags, e.source,
                    e.created_at, e.updated_at, entries_fts.rank
             FROM entries_fts
             JOIN entries e ON e.id = entries_fts.rowid
             WHERE entries_fts MATCH ?1"
        );
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        params_vec.push(Box::new(sanitized));

        if let Some(et) = entry_type {
            sql.push_str(" AND e.type = ?");
            params_vec.push(Box::new(et.as_str().to_string()));
        }
        if let Some(tech) = technology {
            sql.push_str(" AND e.technology = ?");
            params_vec.push(Box::new(tech.to_string()));
        }
        if let Some(proj) = project {
            sql.push_str(" AND e.project = ?");
            params_vec.push(Box::new(proj.to_string()));
        }

        sql.push_str(" ORDER BY entries_fts.rank LIMIT ?");
        params_vec.push(Box::new(limit));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|v| v.as_ref()).collect();

        let mut stmt = self.conn.prepare(&sql)?;
        let results = stmt
            .query_map(params_refs.as_slice(), |row| {
                Ok(SearchResult {
                    entry: row_to_entry(row),
                    rank: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
    }

    /// Simple LIKE-based search for interactive search-as-you-type UIs.
    /// Matches substring anywhere in title, content, technology, or tags.
    pub fn search_like(
        &self,
        query: &str,
        entry_type: Option<&EntryType>,
        technology: Option<&str>,
        limit: u32,
    ) -> Result<(Vec<Entry>, u64), BrainError> {
        let query = query.trim();
        if query.is_empty() {
            return Ok((Vec::new(), 0));
        }
        let pattern = format!("%{query}%");

        let mut sql = String::from(
            "SELECT id, type, title, content, technology, project, tags, source, created_at, updated_at
             FROM entries
             WHERE (title LIKE ?1 OR content LIKE ?1 OR technology LIKE ?1 OR tags LIKE ?1)",
        );
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        params_vec.push(Box::new(pattern));

        if let Some(et) = entry_type {
            sql.push_str(" AND type = ?");
            params_vec.push(Box::new(et.as_str().to_string()));
        }
        if let Some(tech) = technology {
            sql.push_str(" AND technology = ?");
            params_vec.push(Box::new(tech.to_string()));
        }

        let count_sql = format!("SELECT COUNT(*) FROM ({sql})");
        sql.push_str(" ORDER BY updated_at DESC LIMIT ?");
        params_vec.push(Box::new(limit));

        let params_refs: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|v| v.as_ref()).collect();

        let total: u64 = {
            let count_params = &params_refs[..params_refs.len() - 1];
            self.conn.query_row(&count_sql, count_params, |row| row.get(0))?
        };

        let mut stmt = self.conn.prepare(&sql)?;
        let entries = stmt
            .query_map(params_refs.as_slice(), |row| Ok(row_to_entry(row)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((entries, total))
    }

    pub fn get_project_context(&self, project_path: &str) -> Result<Vec<Entry>, BrainError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, type, title, content, technology, project, tags, source, created_at, updated_at
             FROM entries WHERE project = ?1
             ORDER BY updated_at DESC"
        )?;
        let results = stmt
            .query_map(params![project_path], |row| Ok(row_to_entry(row)))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(results)
    }

    pub fn list_entries(&self, filter: &ListFilter) -> Result<(Vec<Entry>, u64), BrainError> {
        let mut where_clauses = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(ref et) = filter.entry_type {
            where_clauses.push("type = ?");
            params_vec.push(Box::new(et.as_str().to_string()));
        }
        if let Some(ref tech) = filter.technology {
            where_clauses.push("technology = ?");
            params_vec.push(Box::new(tech.clone()));
        }
        if let Some(ref tags) = filter.tags {
            where_clauses.push("tags LIKE ?");
            params_vec.push(Box::new(format!("%{}%", tags.to_lowercase())));
        }

        let where_sql = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Get total count
        let count_sql = format!("SELECT COUNT(*) FROM entries {where_sql}");
        let count_params: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|v| v.as_ref()).collect();
        let total: u64 = self.conn.query_row(&count_sql, count_params.as_slice(), |r| r.get(0))?;

        // Get paginated results
        let limit = filter.limit.unwrap_or(10).min(50);
        let offset = filter.offset.unwrap_or(0);

        let query_sql = format!(
            "SELECT id, type, title, content, technology, project, tags, source, created_at, updated_at
             FROM entries {where_sql}
             ORDER BY updated_at DESC
             LIMIT ? OFFSET ?"
        );
        params_vec.push(Box::new(limit));
        params_vec.push(Box::new(offset));
        let query_params: Vec<&dyn rusqlite::types::ToSql> =
            params_vec.iter().map(|v| v.as_ref()).collect();

        let mut stmt = self.conn.prepare(&query_sql)?;
        let entries = stmt
            .query_map(query_params.as_slice(), |row| Ok(row_to_entry(row)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok((entries, total))
    }

    // --- Utility queries ---

    pub fn list_technologies(&self) -> Result<Vec<String>, BrainError> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT technology FROM entries WHERE technology IS NOT NULL ORDER BY technology")?;
        let results = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        Ok(results)
    }

    pub fn list_tags(&self) -> Result<Vec<String>, BrainError> {
        let mut stmt = self
            .conn
            .prepare("SELECT DISTINCT tags FROM entries WHERE tags IS NOT NULL AND tags != ''")?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<String>, _>>()?;

        let mut all_tags: Vec<String> = rows
            .iter()
            .flat_map(|t| t.split(',').map(|s| s.trim().to_string()))
            .filter(|t| !t.is_empty())
            .collect();
        all_tags.sort();
        all_tags.dedup();
        Ok(all_tags)
    }

    pub fn stats(&self) -> Result<Stats, BrainError> {
        let total: u64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))?;

        let mut by_type = std::collections::HashMap::new();
        let mut stmt = self
            .conn
            .prepare("SELECT type, COUNT(*) FROM entries GROUP BY type")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
        })?;
        for row in rows {
            let (t, count) = row?;
            by_type.insert(t, count);
        }

        let mut recent_stmt = self.conn.prepare(
            "SELECT id, type, title, content, technology, project, tags, source, created_at, updated_at
             FROM entries ORDER BY created_at DESC LIMIT 5"
        )?;
        let recent = recent_stmt
            .query_map([], |row| Ok(row_to_entry(row)))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Stats { total, by_type, recent })
    }
}

fn row_to_entry(row: &rusqlite::Row) -> Entry {
    Entry {
        id: row.get(0).unwrap(),
        entry_type: EntryType::from_str(row.get::<_, String>(1).unwrap().as_str()).unwrap(),
        title: row.get(2).unwrap(),
        content: row.get(3).unwrap(),
        technology: row.get(4).unwrap(),
        project: row.get(5).unwrap(),
        tags: row.get(6).unwrap(),
        source: row.get(7).unwrap(),
        created_at: row.get(8).unwrap(),
        updated_at: row.get(9).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_in_memory_creates_schema() {
        let db = Database::open_in_memory().unwrap();
        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM entries", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn open_in_memory_creates_fts_table() {
        let db = Database::open_in_memory().unwrap();
        let count: i64 = db
            .conn
            .query_row("SELECT COUNT(*) FROM entries_fts", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn migration_is_idempotent() {
        let db = Database::open_in_memory().unwrap();
        db.migrate().unwrap();
    }

    fn test_entry() -> NewEntry {
        NewEntry {
            entry_type: EntryType::Learning,
            title: "Test title".to_string(),
            content: "Test content".to_string(),
            technology: Some("rust".to_string()),
            project: None,
            tags: Some("testing, Rust".to_string()),
            source: Some("session".to_string()),
        }
    }

    #[test]
    fn store_and_get_entry() {
        let db = Database::open_in_memory().unwrap();
        let id = db.store_entry(&test_entry()).unwrap();
        let entry = db.get_entry(id).unwrap();
        assert_eq!(entry.title, "Test title");
        assert_eq!(entry.content, "Test content");
        assert_eq!(entry.entry_type, EntryType::Learning);
        assert_eq!(entry.technology.as_deref(), Some("rust"));
        assert_eq!(entry.tags.as_deref(), Some("rust,testing"));
    }

    #[test]
    fn get_entry_not_found() {
        let db = Database::open_in_memory().unwrap();
        let result = db.get_entry(999);
        assert!(matches!(result, Err(BrainError::NotFound(999))));
    }

    #[test]
    fn store_duplicate_returns_error() {
        let db = Database::open_in_memory().unwrap();
        db.store_entry(&test_entry()).unwrap();
        let result = db.store_entry(&test_entry());
        assert!(matches!(result, Err(BrainError::Duplicate { .. })));
    }

    #[test]
    fn update_entry_partial() {
        let db = Database::open_in_memory().unwrap();
        let id = db.store_entry(&test_entry()).unwrap();
        let update = UpdateEntry {
            title: Some("Updated title".to_string()),
            ..Default::default()
        };
        db.update_entry(id, &update).unwrap();
        let entry = db.get_entry(id).unwrap();
        assert_eq!(entry.title, "Updated title");
        assert_eq!(entry.content, "Test content");
    }

    #[test]
    fn update_entry_not_found() {
        let db = Database::open_in_memory().unwrap();
        let update = UpdateEntry {
            title: Some("New".to_string()),
            ..Default::default()
        };
        let result = db.update_entry(999, &update);
        assert!(matches!(result, Err(BrainError::NotFound(999))));
    }

    #[test]
    fn delete_entry_removes_it() {
        let db = Database::open_in_memory().unwrap();
        let id = db.store_entry(&test_entry()).unwrap();
        db.delete_entry(id).unwrap();
        assert!(matches!(db.get_entry(id), Err(BrainError::NotFound(_))));
    }

    #[test]
    fn delete_entry_not_found() {
        let db = Database::open_in_memory().unwrap();
        let result = db.delete_entry(999);
        assert!(matches!(result, Err(BrainError::NotFound(999))));
    }

    fn seed_entries(db: &Database) {
        let entries = vec![
            NewEntry {
                entry_type: EntryType::Learning,
                title: "WKWebView needs network entitlement".to_string(),
                content: "Even for local HTML, WKWebView requires network.client entitlement in sandbox".to_string(),
                technology: Some("swift".to_string()),
                project: None,
                tags: Some("macos, sandbox, webview".to_string()),
                source: Some("session".to_string()),
            },
            NewEntry {
                entry_type: EntryType::Gotcha,
                title: "NSRulerView paints over NSTextView".to_string(),
                content: "macOS 14 changed clipsToBounds default. Fill bounds not rect.".to_string(),
                technology: Some("swift".to_string()),
                project: None,
                tags: Some("macos, appkit".to_string()),
                source: Some("session".to_string()),
            },
            NewEntry {
                entry_type: EntryType::ProjectContext,
                title: "Custom Swift compiler path".to_string(),
                content: "Must use DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer swift".to_string(),
                technology: Some("swift".to_string()),
                project: Some("/Users/dev/my-swift-app".to_string()),
                tags: Some("build, xcode".to_string()),
                source: Some("session".to_string()),
            },
            NewEntry {
                entry_type: EntryType::Learning,
                title: "SQLite WAL mode".to_string(),
                content: "WAL mode allows concurrent reads during writes".to_string(),
                technology: Some("sqlite".to_string()),
                project: None,
                tags: Some("database, performance".to_string()),
                source: Some("session".to_string()),
            },
        ];
        for entry in &entries {
            db.store_entry(entry).unwrap();
        }
    }

    #[test]
    fn search_entries_by_keyword() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let results = db.search_entries("WKWebView", None, None, None, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.title, "WKWebView needs network entitlement");
    }

    #[test]
    fn search_entries_filter_by_technology() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let results = db.search_entries("macOS", None, Some("swift"), None, 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_entries_filter_by_type() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let results = db.search_entries("swift", Some(&EntryType::Gotcha), None, None, 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].entry.title, "NSRulerView paints over NSTextView");
    }

    #[test]
    fn search_entries_empty_query() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let results = db.search_entries("nonexistent_term_xyz", None, None, None, 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn get_project_context_returns_scoped_entries() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let results = db.get_project_context("/Users/dev/my-swift-app").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Custom Swift compiler path");
    }

    #[test]
    fn get_project_context_empty_for_unknown_project() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let results = db.get_project_context("/nonexistent/path").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn list_entries_no_filter() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let filter = ListFilter { limit: Some(10), ..Default::default() };
        let (entries, total) = db.list_entries(&filter).unwrap();
        assert_eq!(total, 4);
        assert_eq!(entries.len(), 4);
    }

    #[test]
    fn list_entries_filter_by_type() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let filter = ListFilter {
            entry_type: Some(EntryType::Learning),
            limit: Some(10),
            ..Default::default()
        };
        let (entries, total) = db.list_entries(&filter).unwrap();
        assert_eq!(total, 2);
    }

    #[test]
    fn list_entries_with_offset() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let filter = ListFilter { limit: Some(2), offset: Some(2), ..Default::default() };
        let (entries, total) = db.list_entries(&filter).unwrap();
        assert_eq!(total, 4);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn list_technologies_returns_distinct() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let techs = db.list_technologies().unwrap();
        assert_eq!(techs, vec!["sqlite", "swift"]);
    }

    #[test]
    fn list_tags_returns_distinct() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let tags = db.list_tags().unwrap();
        assert!(tags.contains(&"macos".to_string()));
        assert!(tags.contains(&"sandbox".to_string()));
    }

    #[test]
    fn stats_returns_counts() {
        let db = Database::open_in_memory().unwrap();
        seed_entries(&db);
        let stats = db.stats().unwrap();
        assert_eq!(stats.total, 4);
        assert_eq!(*stats.by_type.get("learning").unwrap(), 2);
        assert_eq!(*stats.by_type.get("gotcha").unwrap(), 1);
        assert_eq!(*stats.by_type.get("project_context").unwrap(), 1);
    }
}
