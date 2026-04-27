use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub current_wallpaper: PathBuf,
    pub current_dir: PathBuf,
    pub save_dir: PathBuf,
}
