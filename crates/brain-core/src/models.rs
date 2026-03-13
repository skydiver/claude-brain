use crate::error::BrainError;
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
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Learning => "learning",
            Self::ProjectContext => "project_context",
            Self::Gotcha => "gotcha",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, BrainError> {
        match s {
            "learning" => Ok(Self::Learning),
            "project_context" => Ok(Self::ProjectContext),
            "gotcha" => Ok(Self::Gotcha),
            other => Err(BrainError::InvalidEntryType(other.to_string())),
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewEntry {
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub title: String,
    pub content: String,
    pub technology: Option<String>,
    pub project: Option<String>,
    pub tags: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateEntry {
    pub title: Option<String>,
    pub content: Option<String>,
    pub technology: Option<String>,
    pub project: Option<String>,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    #[serde(flatten)]
    pub entry: Entry,
    pub rank: f64,
}

#[derive(Debug, Clone, Default)]
pub struct ListFilter {
    pub entry_type: Option<EntryType>,
    pub technology: Option<String>,
    pub tags: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stats {
    pub total: u64,
    pub by_type: HashMap<String, u64>,
    pub recent: Vec<Entry>,
}

/// Normalize tags: lowercase, trim, deduplicate, sort, rejoin with commas.
pub fn normalize_tags(raw: &str) -> String {
    let mut tags: Vec<String> = raw
        .split(',')
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    tags.sort();
    tags.dedup();
    tags.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_type_roundtrip() {
        assert_eq!(EntryType::from_str("learning").unwrap(), EntryType::Learning);
        assert_eq!(EntryType::from_str("project_context").unwrap(), EntryType::ProjectContext);
        assert_eq!(EntryType::from_str("gotcha").unwrap(), EntryType::Gotcha);
        assert_eq!(EntryType::Learning.as_str(), "learning");
        assert_eq!(EntryType::ProjectContext.as_str(), "project_context");
        assert_eq!(EntryType::Gotcha.as_str(), "gotcha");
    }

    #[test]
    fn entry_type_invalid() {
        assert!(EntryType::from_str("invalid").is_err());
    }

    #[test]
    fn normalize_tags_basic() {
        assert_eq!(normalize_tags("Swift, macOS , SANDBOX"), "macos,sandbox,swift");
    }

    #[test]
    fn normalize_tags_dedup() {
        assert_eq!(normalize_tags("rust, Rust, RUST"), "rust");
    }

    #[test]
    fn normalize_tags_empty() {
        assert_eq!(normalize_tags(""), "");
        assert_eq!(normalize_tags("  ,  , "), "");
    }
}
