mod app;
mod cli;
mod config;
mod config_manager;
mod image_buffer;
mod local_image;
mod online_image;
mod scraper;
mod ui;

use crate::app::App;
use crate::cli::Cli;
use crate::config::Config;
use crate::config_manager::ConfigManager;

use anyhow::{Result, anyhow};
use clap::Parser;
use std::path::PathBuf;
use tokio::fs;
use tracing::error;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // TODO: add seperate configs for debug and release
    let file_appender = tracing_appender::rolling::daily("logs", "wppr");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let home_dir = if let Some(home) = home::home_dir() {
        home
    } else {
        let e = anyhow!("{}", "Could not find home dir");
        error!("{}", e);
        return Err(e);
    };

    let config_path = PathBuf::from(format!("{}/.config/wppr/config.json", home_dir.display()));
    let config = ConfigManager::load_config(&config_path)?;
    let mut app = App::new(&config_path, config, cli);

    if !app.config.save_dir.exists()
        && let Some(home) = home::home_dir()
    {
        let dir_path = PathBuf::from(format!("{}/Pictures/wppr", home.display()));
        fs::create_dir_all(&dir_path).await?;
        app.config.save_dir = dir_path;
    }

    match app.menu().await {
        Ok(()) => {}
        Err(e) => {
            error!("{}", e);
            return Err(e);
        }
    }

    Ok(())
}
