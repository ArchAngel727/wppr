use ratatui::layout::Size;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub static CELL_SIZE: [Size; 3] = [Size::new(12, 5), Size::new(20, 7), Size::new(31, 10)];

#[derive(Serialize, Deserialize)]
pub struct OptionConfig {
    #[serde(default)]
    pub current_wallpaper: Option<PathBuf>,
    #[serde(default)]
    pub current_dir: Option<PathBuf>,
    #[serde(default)]
    pub save_dir: Option<PathBuf>,
    #[serde(default)]
    pub cell_size: Option<usize>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub current_wallpaper: PathBuf,
    pub current_dir: PathBuf,
    pub save_dir: PathBuf,
    pub cell_size: usize,
}

impl From<OptionConfig> for Config {
    fn from(value: OptionConfig) -> Self {
        Self {
            current_wallpaper: value.current_wallpaper.unwrap_or_default(),
            current_dir: value.current_dir.unwrap_or_default(),
            save_dir: value.save_dir.unwrap_or_default(),
            cell_size: value.cell_size.unwrap_or(1),
        }
    }
}
