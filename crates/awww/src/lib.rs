use std::{
    env,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Result;

#[derive(PartialEq)]
pub enum AwwwSocketStatus {
    Running,
    NotRunning,
}

pub struct AwwwController {}

impl AwwwController {
    pub fn is_installed() -> bool {
        Command::new("awww").arg("--version").output().is_ok()
    }

    // TODO: Do a real implementation
    fn get_socket_path() -> PathBuf {
        let display = env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-1".into());
        let runtime = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp/awww".into());
        PathBuf::from(runtime).join(format!("{}-awww-daemon.sock", display))
    }

    pub fn check_daemon_status() -> AwwwSocketStatus {
        match UnixStream::connect(AwwwController::get_socket_path()).is_ok() {
            true => AwwwSocketStatus::Running,
            false => AwwwSocketStatus::NotRunning,
        }
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
