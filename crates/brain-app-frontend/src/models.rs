use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EntryType {
    Learning,
    ProjectContext,
    Gotcha,
}

impl EntryType {
    pub fn label(&self) -> &'static str {
        match self {
            EntryType::Learning => "Learning",
            EntryType::ProjectContext => "Project Context",
            EntryType::Gotcha => "Gotcha",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            EntryType::Learning => "learning",
            EntryType::ProjectContext => "project_context",
            EntryType::Gotcha => "gotcha",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entry {
    pub id: i64,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub title: String,
    pub content: String,
    pub technology: Option<String>,
    pub project: Option<String>,
    pub tags: Option<String>,
    pub source: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl Entry {
    pub fn tags_list(&self) -> Vec<&str> {
        self.tags
            .as_deref()
            .map(|t| {
                t.split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn content_preview(&self, max_len: usize) -> String {
        if self.content.len() <= max_len {
            self.content.clone()
        } else {
            let truncated: String = self.content.chars().take(max_len).collect();
            format!("{truncated}…")
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub entries: Vec<Entry>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtsEntry {
    #[serde(flatten)]
    pub entry: Entry,
    pub rank: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtsResponse {
    pub entries: Vec<FtsEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total: u64,
    pub by_type: HashMap<String, u64>,
    pub recent: Vec<Entry>,
}
