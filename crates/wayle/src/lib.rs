use anyhow::Result;
use std::{path::Path, process::Command};

pub struct WayleController {}

impl WayleController {
    pub fn is_installed() -> bool {
        Command::new("wayle").arg("--version").output().is_ok()
    }

    pub fn is_running() -> bool {
        let output = Command::new("wayle").args(["panel", "status"]).output();

        match output {
            Ok(output) => {
                let mut msg = String::new();

                for c in output.stdout {
                    msg.push(c as char);
                }

                msg.trim() == "Panel is running"
            }
            Err(e) => {
                println!("{}", e);
                false
            }
        }
    }

    pub fn set_wallpaper(path: &Path) -> Result<()> {
        let output = Command::new("wayle")
            .args(["wallpaper", "set"])
            .arg(path)
            .output();

        if output.is_ok() {
            Ok(())
        } else {
            Err(output.err().unwrap().into())
        }
    }
}
