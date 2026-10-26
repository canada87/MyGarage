mod due;
mod error;
mod ics;
mod mailer;
mod models;
mod routes;
mod scheduler;
mod settings;
mod static_files;

use std::{net::SocketAddr, path::PathBuf, str::FromStr};

use anyhow::Context;
use axum::{extract::DefaultBodyLimit, Router};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub docs_dir: PathBuf,
    pub base_url: String,
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "mygarage=info,tower_http=warn".into()))
        .init();

    let data_dir = PathBuf::from(env_or("MYGARAGE_DATA_DIR", "./data"));
    let docs_dir = PathBuf::from(env_or("MYGARAGE_DOCS_DIR", "./documents"));
    let listen = env_or("MYGARAGE_LISTEN", "0.0.0.0:8080");
    let base_url = env_or("MYGARAGE_BASE_URL", "http://localhost:8080").trim_end_matches('/').to_string();

    std::fs::create_dir_all(&data_dir).with_context(|| format!("creating {}", data_dir.display()))?;
    std::fs::create_dir_all(&docs_dir).with_context(|| format!("creating {}", docs_dir.display()))?;

    let db_path = data_dir.join("mygarage.db");
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", db_path.display()))?
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .foreign_keys(true);
    let db = SqlitePoolOptions::new().max_connections(5).connect_with(options).await?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let state = AppState { db, docs_dir, base_url };

    tokio::spawn(scheduler::run(state.clone()));

    let app = Router::new()
        .nest("/api", routes::router())
        .fallback(static_files::serve)
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr: SocketAddr = listen.parse().context("invalid MYGARAGE_LISTEN")?;
    tracing::info!("MyGarage listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
