use brain_core::db::Database;
use brain_core::error::BrainError;
use brain_core::models::{EntryType, NewEntry, UpdateEntry, ListFilter};
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};

// --- Request types ---

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StatsRequest {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct StoreEntryRequest {
    #[schemars(description = "Entry type: 'learning', 'project_context', or 'gotcha'")]
    pub r#type: String,
    #[schemars(description = "Short descriptive title for the entry")]
    pub title: String,
    #[schemars(description = "Full content of the knowledge entry (markdown formatted)")]
    pub content: String,
    #[schemars(description = "Technology this relates to (e.g., 'swift', 'rust', 'sqlite')")]
    pub technology: Option<String>,
    #[schemars(description = "Absolute path to the project root this entry is scoped to")]
    pub project: Option<String>,
    #[schemars(description = "Comma-separated tags for categorization")]
    pub tags: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct UpdateEntryRequest {
    #[schemars(description = "ID of the entry to update")]
    pub id: i64,
    #[schemars(description = "New title (omit to keep current)")]
    pub title: Option<String>,
    #[schemars(description = "New content in markdown (omit to keep current)")]
    pub content: Option<String>,
    #[schemars(description = "New technology (omit to keep current)")]
    pub technology: Option<String>,
    #[schemars(description = "New project path (omit to keep current)")]
    pub project: Option<String>,
    #[schemars(description = "New tags (omit to keep current)")]
    pub tags: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DeleteEntryRequest {
    #[schemars(description = "ID of the entry to delete")]
    pub id: i64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchEntriesRequest {
    #[schemars(description = "Search query (keywords to match against title, content, technology, tags)")]
    pub query: String,
    #[schemars(description = "Filter by entry type: 'learning', 'project_context', or 'gotcha'")]
    pub r#type: Option<String>,
    #[schemars(description = "Filter by technology (e.g., 'swift', 'rust')")]
    pub technology: Option<String>,
    #[schemars(description = "Filter by project path")]
    pub project: Option<String>,
    #[schemars(description = "Max results to return (default 10, max 50)")]
    pub limit: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetProjectContextRequest {
    #[schemars(description = "Absolute path to the project root")]
    pub project_path: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetEntryRequest {
    #[schemars(description = "ID of the entry to retrieve")]
    pub id: i64,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListEntriesRequest {
    #[schemars(description = "Filter by entry type: 'learning', 'project_context', or 'gotcha'")]
    pub r#type: Option<String>,
    #[schemars(description = "Filter by technology")]
    pub technology: Option<String>,
    #[schemars(description = "Filter by tag")]
    pub tags: Option<String>,
    #[schemars(description = "Max results (default 10, max 50)")]
    pub limit: Option<u32>,
    #[schemars(description = "Offset for pagination")]
    pub offset: Option<u32>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListTechnologiesRequest {}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ListTagsRequest {}

// --- Server ---

#[derive(Debug, Clone)]
pub struct BrainServer {
    db: std::sync::Arc<std::sync::Mutex<Database>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl BrainServer {
    pub fn new(db: Database) -> Self {
        Self {
            db: std::sync::Arc::new(std::sync::Mutex::new(db)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Get summary statistics about stored knowledge entries")]
    fn stats(&self, Parameters(_): Parameters<StatsRequest>) -> String {
        let db = self.db.lock().unwrap();
        match db.stats() {
            Ok(stats) => serde_json::to_string_pretty(&stats).unwrap(),
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(description = "Store a new knowledge entry in the brain")]
    fn store_entry(&self, Parameters(req): Parameters<StoreEntryRequest>) -> String {
        let db = self.db.lock().unwrap();
        let entry_type = match EntryType::from_str(&req.r#type) {
            Ok(t) => t,
            Err(e) => return format!("{{\"error\": \"{e}\"}}"),
        };
        let size_warning = if req.content.len() > 10_240 {
            Some(format!(
                "Warning: content is {}KB, which may impact context window usage",
                req.content.len() / 1024
            ))
        } else {
            None
        };
        let new_entry = NewEntry {
            entry_type,
            title: req.title,
            content: req.content,
            technology: req.technology,
            project: req.project,
            tags: req.tags,
            source: Some("session".to_string()),
        };
        match db.store_entry(&new_entry) {
            Ok(id) => {
                let entry = db.get_entry(id).unwrap();
                let mut result = serde_json::to_value(&entry).unwrap();
                if let Some(warning) = size_warning {
                    result
                        .as_object_mut()
                        .unwrap()
                        .insert("warning".to_string(), serde_json::Value::String(warning));
                }
                serde_json::to_string_pretty(&result).unwrap()
            }
            Err(BrainError::Duplicate { existing_id, .. }) => {
                let existing = db.get_entry(existing_id).unwrap();
                let mut result = serde_json::to_value(&existing).unwrap();
                result.as_object_mut().unwrap().insert(
                    "note".to_string(),
                    serde_json::Value::String("Duplicate entry already exists".to_string()),
                );
                serde_json::to_string_pretty(&result).unwrap()
            }
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(description = "Update an existing knowledge entry (partial update — only provided fields are changed)")]
    fn update_entry(&self, Parameters(req): Parameters<UpdateEntryRequest>) -> String {
        let db = self.db.lock().unwrap();
        let update = UpdateEntry {
            title: req.title,
            content: req.content,
            technology: req.technology,
            project: req.project,
            tags: req.tags,
        };
        match db.update_entry(req.id, &update) {
            Ok(entry) => serde_json::to_string_pretty(&entry).unwrap(),
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(description = "Delete a knowledge entry by ID")]
    fn delete_entry(&self, Parameters(req): Parameters<DeleteEntryRequest>) -> String {
        let db = self.db.lock().unwrap();
        match db.delete_entry(req.id) {
            Ok(()) => format!("{{\"deleted\": {}}}", req.id),
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(description = "Full-text search across all knowledge entries. Returns results ranked by relevance.")]
    fn search_entries(&self, Parameters(req): Parameters<SearchEntriesRequest>) -> String {
        let db = self.db.lock().unwrap();
        let entry_type = req.r#type.as_deref().map(EntryType::from_str).transpose();
        let entry_type = match entry_type {
            Ok(t) => t,
            Err(e) => return format!("{{\"error\": \"{e}\"}}"),
        };
        let limit = req.limit.unwrap_or(10).min(50);
        match db.search_entries(
            &req.query,
            entry_type.as_ref(),
            req.technology.as_deref(),
            req.project.as_deref(),
            limit,
        ) {
            Ok(results) => {
                let total = results.len();
                serde_json::to_string_pretty(&serde_json::json!({
                    "results": results,
                    "total": total
                }))
                .unwrap()
            }
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(description = "Get all knowledge entries scoped to a specific project directory")]
    fn get_project_context(&self, Parameters(req): Parameters<GetProjectContextRequest>) -> String {
        let db = self.db.lock().unwrap();
        match db.get_project_context(&req.project_path) {
            Ok(entries) => {
                let total = entries.len();
                serde_json::to_string_pretty(&serde_json::json!({
                    "results": entries,
                    "total": total
                }))
                .unwrap()
            }
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(description = "Get a single knowledge entry by its ID")]
    fn get_entry(&self, Parameters(req): Parameters<GetEntryRequest>) -> String {
        let db = self.db.lock().unwrap();
        match db.get_entry(req.id) {
            Ok(entry) => serde_json::to_string_pretty(&entry).unwrap(),
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(description = "List knowledge entries with optional filters and pagination")]
    fn list_entries(&self, Parameters(req): Parameters<ListEntriesRequest>) -> String {
        let db = self.db.lock().unwrap();
        let entry_type = req.r#type.as_deref().map(EntryType::from_str).transpose();
        let entry_type = match entry_type {
            Ok(t) => t,
            Err(e) => return format!("{{\"error\": \"{e}\"}}"),
        };
        let filter = ListFilter {
            entry_type,
            technology: req.technology,
            tags: req.tags,
            limit: req.limit,
            offset: req.offset,
        };
        match db.list_entries(&filter) {
            Ok((entries, total)) => {
                serde_json::to_string_pretty(&serde_json::json!({
                    "results": entries,
                    "total": total
                }))
                .unwrap()
            }
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(description = "List all distinct technologies stored in the knowledge base")]
    fn list_technologies(&self, Parameters(_): Parameters<ListTechnologiesRequest>) -> String {
        let db = self.db.lock().unwrap();
        match db.list_technologies() {
            Ok(techs) => {
                serde_json::to_string_pretty(&serde_json::json!({ "technologies": techs })).unwrap()
            }
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }

    #[tool(description = "List all distinct tags used across knowledge entries")]
    fn list_tags(&self, Parameters(_): Parameters<ListTagsRequest>) -> String {
        let db = self.db.lock().unwrap();
        match db.list_tags() {
            Ok(tags) => {
                serde_json::to_string_pretty(&serde_json::json!({ "tags": tags })).unwrap()
            }
            Err(e) => format!("{{\"error\": \"{e}\"}}"),
        }
    }
}

#[tool_handler]
impl ServerHandler for BrainServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            instructions: Some(
                "claude-brain: persistent knowledge store for Claude Code".to_string(),
            ),
            ..Default::default()
        }
    }
}
