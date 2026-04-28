mod app;
mod cli;
mod config;
mod config_manager;
mod local_image;
mod online_image;
mod scraper;

use crate::app::App;
use crate::cli::Cli;
use crate::config::Config;
use crate::config_manager::ConfigManager;

use anyhow::{Result, anyhow};
use awww::AwwwController;
use clap::Parser;
use matugen::MatugenController;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    compile_error!("AW HELL NAH I AINT RUNNING ON {}", target_os);

    if !AwwwController::is_installed() {
        return Err(anyhow!("awww is not installed"));
    }

    if !MatugenController::is_installed() {
        return Err(anyhow!("matugen is not installed"));
    }

    let cli = Cli::parse();

    let config_path = PathBuf::from(if let Some(home) = home::home_dir() {
        format!("{}/.config/wppr/config.json", home.display())
    } else {
        return Err(anyhow!("Could not find home dir"));
    });

    let config = ConfigManager::load_config(&config_path)?;
    let mut app = App::new(&config_path, config, cli);

    app.menu().await?;

    Ok(())
}
