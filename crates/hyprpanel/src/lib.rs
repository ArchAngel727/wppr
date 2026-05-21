use anyhow::Result;
use std::{os::unix::net::UnixStream, path::Path, process::Command};

pub enum HyprpanelSocketStatus {
    Running,
    NotRunning,
}

pub struct HyprpanelController {}

impl HyprpanelController {
    pub fn is_installed() -> bool {
        Command::new("hyprpanel").arg("--version").output().is_ok()
    }

    pub fn check_daemon_status() -> HyprpanelSocketStatus {
        let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") else {
            return HyprpanelSocketStatus::NotRunning;
        };

        let sock = format!("{runtime}/astal/hyprpanel.sock");
        match UnixStream::connect(sock).is_ok() {
            true => HyprpanelSocketStatus::Running,
            false => HyprpanelSocketStatus::NotRunning,
        }
    }

    pub fn set_wallpaper(path: &Path) -> Result<()> {
        let result = Command::new("hyprpanel").arg("sw").arg(path).output();

        result?.stdout.iter().for_each(|f| {
            print!("{}", *f as char);
        });

        Ok(())
    }
}
