mod app;
mod cli;
mod config;
mod config_manager;
mod db_manager;
mod image_buffer;
mod image_processor;
mod local_image;
mod online_image;
mod scraper;
mod ui;

use crate::{
    cli::Cli,
    config::Config,
    config_manager::ConfigManager,
    {app::App, db_manager::DBManager},
};

use anyhow::{Result, anyhow};
use clap::Parser;
use std::path::PathBuf;
use tokio::fs;
use tracing::error;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

fn setup_log() -> WorkerGuard {
    let log_path = if cfg!(debug_assertions) {
        PathBuf::from("./logs/")
    } else {
        dirs::cache_dir()
            .expect("Could not find cache dir")
            .join("wppr/logs")
    };
    let file_appender = tracing_appender::rolling::daily(log_path, "wppr.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    _guard
}

async fn setup_db() -> Result<()> {
    let mut conn = DBManager::get_db_connection().await?;

    sqlx::migrate!("../../migrations/")
        .run(&mut conn)
        .await
        .inspect_err(|e| error!("{}", e))
        .expect("Failed to run database migration");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let _guard = setup_log();
    setup_db().await?;

    let home_dir = home::home_dir()
        .ok_or_else(|| anyhow!("home dir not found"))
        .inspect_err(|e| error!("{e:#}"))?;

    let config_path = PathBuf::from(format!("{}/.config/wppr/config.json", home_dir.display()));
    let option_config =
        ConfigManager::load_config(&config_path).inspect_err(|e| error!("{e:#}"))?;

    let mut app = App::new(&config_path, option_config.into(), cli);

    if !app.config.save_dir.exists() {
        let dir_path = PathBuf::from(format!("{}/Pictures/wppr", home_dir.display()));

        fs::create_dir_all(&dir_path)
            .await
            .inspect_err(|e| error!("Failed to create save dir {e:#}"))?;

        app.config.save_dir = dir_path;
    }

    app.menu().await.inspect_err(|e| error!("{e:#}"))?;

    ConfigManager::save_config(&app)?;

    Ok(())
}
