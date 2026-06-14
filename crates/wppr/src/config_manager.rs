use crate::app::App;
use crate::config::Config;

use anyhow::Result;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};
use tracing::error;

pub struct ConfigManager {}

impl ConfigManager {
    pub fn save_config(app: &App) -> Result<()> {
        if !app.config_path.exists()
            && let Some(dir) = app.config_path.parent()
        {
            fs::create_dir_all(dir)
                .inspect_err(|e| error!("Failed to create config dir: {e:#}"))?;
        }

        let mut file = File::create(app.config_path)
            .inspect_err(|e| error!("Failed to create config file: {e:#}"))?;
        file.write_all(
            &serde_json::to_vec_pretty(&app.config)
                .inspect_err(|e| error!("Failed to parse json: {e:#}"))?,
        )
        .inspect_err(|e| error!("Failed to write config file: {e:#}"))?;

        Ok(())
    }

    pub fn load_config(path: &Path) -> Result<Config> {
        let default_config = r#"{
        "current_wallpaper": "",
        "current_dir": "",
        "save_dir": ""
    }"#;

        if let Some(dir) = path.parent()
            && !dir.exists()
        {
            fs::create_dir_all(dir)
                .inspect_err(|e| error!("Failed to create config dir: {e:#}"))?;
        }

        if path.exists() {
            Ok(serde_json::from_str(
                fs::read_to_string(path)
                    .inspect_err(|e| error!("Failed to read config: {e:#}"))?
                    .as_str(),
            )
            .inspect_err(|e| error!("Failed to parse config as json: {e:#}"))?)
        } else {
            let mut file = File::create(path)?;
            file.write_all(default_config.as_bytes())
                .inspect_err(|e| error!("Failed to write config: {e:#}"))?;

            Ok(serde_json::from_str(default_config)
                .inspect_err(|e| error!("Failed to parse default config as json: {e:#}"))?)
        }
    }
}
