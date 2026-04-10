use std::{path::Path, process::Command};

use anyhow::Result;

pub struct AwwwControlle {}

impl AwwwControlle {
    pub fn is_installed() -> bool {
        Command::new("awww").arg("--version").output().is_ok()
    }

    pub fn set_wallpaper(path: &Path) -> Result<()> {
        let result = Command::new("awww")
            .arg("img")
            .arg(path)
            .args(["-t", "random"])
            .args(["--transition-fps", "60"])
            .args(["--transition-duration", "1"])
            .output();

        result?.stdout.iter().for_each(|f| {
            print!("{}", *f as char);
        });

        Ok(())
    }
}

/*
data.stdout.iter().for_each(|f| {
    print!("{}", *f as char);
});
*/
