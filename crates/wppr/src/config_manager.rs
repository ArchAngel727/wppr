use crate::app::App;
use crate::config::Config;

use anyhow::Result;
use std::{
    fs::{self as stdfs, File},
    io::Write,
    path::Path,
};

pub struct ConfigManager {}

impl ConfigManager {
    pub fn save_config(app: &App) -> Result<()> {
        if !app.config_path.exists()
            && let Some(dir) = app.config_path.parent()
        {
            stdfs::create_dir_all(dir)?;
        }

        let mut file = File::create(app.config_path)?;
        file.write_all(&serde_json::to_vec_pretty(&app.config)?)?;

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
            stdfs::create_dir_all(dir)?;
        }

        if !path.exists() {
            let mut file = File::create(path)?;
            file.write_all(default_config.as_bytes())?;

            Ok(serde_json::from_str(default_config)?)
        } else {
            Ok(serde_json::from_str(stdfs::read_to_string(path)?.as_str())?)
        }
    }
}
