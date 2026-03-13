mod server;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing_subscriber::{self, EnvFilter};
use server::BrainServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let db_path = std::env::var("BRAIN_DB_PATH")
        .unwrap_or_else(|_| {
            let dir = dirs::home_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join(".config")
                .join("claude-brain");
            std::fs::create_dir_all(&dir).ok();
            dir.join("brain.db").to_string_lossy().to_string()
        });

    tracing::info!("Opening database at {db_path}");
    let db = brain_core::db::Database::open(&db_path)?;

    let service = BrainServer::new(db).serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
