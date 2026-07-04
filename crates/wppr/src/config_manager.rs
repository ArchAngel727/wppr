use crate::{app::App, config::OptionConfig};

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

    pub fn load_config(path: &Path) -> Result<OptionConfig> {
        let default_config = concat!(
            "{\n",
            r#"    "current_wallpaper": "","#,
            "\n",
            r#"    "current_dir": "","#,
            "\n",
            r#"    "save_dir": "","#,
            "\n",
            r#"    "cell_size": 0"#,
            "\n",
            "}"
        );

        if let Some(dir) = path.parent()
            && !dir.exists()
        {
            fs::create_dir_all(dir)
                .inspect_err(|e| error!("Failed to create config dir: {e:#}"))?;
        }

        if path.exists() {
            let str =
                fs::read_to_string(path).inspect_err(|e| error!("Failed to read config: {e:#}"))?;

            let option_config: OptionConfig = serde_json::from_str(&str)
                .inspect_err(|e| error!("Failed to parse config as json: {e:#}"))?;

            Ok(option_config)
        } else {
            let mut file = File::create(path)?;
            file.write_all(default_config.as_bytes())
                .inspect_err(|e| error!("Failed to write config: {e:#}"))?;

            Ok(serde_json::from_str(default_config)
                .inspect_err(|e| error!("Failed to parse default config as json: {e:#}"))?)
        }
    }
}
