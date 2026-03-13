use thiserror::Error;

#[derive(Debug, Error)]
pub enum BrainError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("invalid entry type: {0}")]
    InvalidEntryType(String),

    #[error("entry not found: id={0}")]
    NotFound(i64),

    #[error("duplicate entry: \"{title}\" (type={entry_type}, existing_id={existing_id})")]
    Duplicate { existing_id: i64, title: String, entry_type: String },

    #[error("migration failed: {0}")]
    Migration(String),
}
