use std::{path::Path, process::Command};

use anyhow::Result;

pub struct MatugenController {}

impl MatugenController {
    pub fn is_installed() -> bool {
        Command::new("matugen").arg("--version").output().is_ok()
    }

    pub fn update_colors(path: &Path) -> Result<()> {
        let result = Command::new("matugen")
            .arg("image")
            .arg(path)
            .args(["--source-color-index", "0"])
            .output();

        result?.stdout.iter().for_each(|f| {
            print!("{}", *f as char);
        });

        Ok(())
    }
}
