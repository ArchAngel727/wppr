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
use crossterm::{
    cursor, execute,
    terminal::{LeaveAlternateScreen, disable_raw_mode},
};
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

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        error!("PANIC: {panic_info}\n{backtrace}");

        let _ = disable_raw_mode();
        let _ = execute!(std::io::stderr(), LeaveAlternateScreen, cursor::Show);
        original_hook(panic_info);
    }));
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
    install_panic_hook();
    setup_db().await?;

    let config_path = if cfg!(debug_assertions) {
        PathBuf::from("./config.json")
    } else {
        let mut path = dirs::config_dir()
            .ok_or_else(|| anyhow!("config dir not found"))
            .inspect_err(|e| error!("{e:#}"))?;
        path.push("wppr/config.json");

        path
    };

    let option_config =
        ConfigManager::load_config(&config_path).inspect_err(|e| error!("{e:#}"))?;

    let mut picture_dir = dirs::picture_dir()
        .ok_or_else(|| anyhow!("picture dir not found"))
        .inspect_err(|e| error!("{e:#}"))?;
    picture_dir.push("wppr/");

    fs::create_dir_all(&picture_dir)
        .await
        .inspect_err(|e| error!("Failed to create save dir {e:#}"))?;

    let mut app = App::new(&config_path, option_config.into(), cli);

    if app.config.save_dir.as_os_str() == "" {
        app.config.save_dir = picture_dir;
    }

    app.menu().await.inspect_err(|e| error!("{e:#}"))?;

    ConfigManager::save_config(&app)?;

    Ok(())
}
