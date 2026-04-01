//! Desktop environment backends for applying wallpapers.
//!
//! Detects the current DE/WM at runtime and delegates to the appropriate
//! setter tool. Supported environments (Arch Linux focus):
//!
//! | Environment          | Tool                         |
//! |----------------------|------------------------------|
//! | KDE Plasma           | `plasma-apply-wallpaperimage`|
//! | GNOME / Cinnamon     | `gsettings`                  |
//! | Hyprland             | `swww` + `swww-daemon`       |
//! | Sway                 | `swaybg`                     |
//! | Generic Wayland      | `swww` + `swww-daemon`       |
//! | X11 (any WM)         | `feh`                        |

use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{BackendError, SwpError};

fn detect_desktop() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default()
        .to_uppercase()
}

pub fn apply(path: &Path) -> Result<(), SwpError> {
    let desktop = detect_desktop();

    if desktop.contains("KDE") {
        apply_kde(path)
    } else if desktop.contains("GNOME")
        || desktop.contains("UNITY")
        || desktop.contains("PANTHEON")
        || desktop.contains("BUDGIE")
        || desktop.contains("CINNAMON")
    {
        apply_gnome(path)
    } else if desktop.contains("HYPRLAND") {
        apply_swww(path)
    } else if desktop.contains("SWAY") {
        apply_sway(path)
    } else if std::env::var("WAYLAND_DISPLAY").is_ok() {
        apply_swww(path).map_err(|_| BackendError::WaylandNoCompatibleBackend.into())
    } else if std::env::var("DISPLAY").is_ok() {
        apply_feh(path)
    } else {
        Err(BackendError::UnknownDesktop {
            desktop: std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default(),
        }
        .into())
    }
}

fn apply_kde(path: &Path) -> Result<(), SwpError> {
    let status = Command::new("plasma-apply-wallpaperimage")
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|source| BackendError::CommandSpawn {
            tool: "plasma-apply-wallpaperimage",
            source,
            help: "Check if plasma-workspace is installed.",
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(BackendError::CommandFailed {
            tool: "plasma-apply-wallpaperimage",
            code: status.code(),
        }
        .into())
    }
}

fn apply_gnome(path: &Path) -> Result<(), SwpError> {
    let uri = format!("file://{}", path.display());

    let status = Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.background",
            "picture-uri",
            &uri,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|source| BackendError::CommandSpawn {
            tool: "gsettings",
            source,
            help: "Check if gsettings is installed and available.",
        })?;

    if !status.success() {
        return Err(BackendError::CommandFailed {
            tool: "gsettings",
            code: status.code(),
        }
        .into());
    }

    let _ = Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.background",
            "picture-uri-dark",
            &uri,
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    Ok(())
}

fn apply_swww(path: &Path) -> Result<(), SwpError> {
    let status = Command::new("swww")
        .args(["img", &path.to_string_lossy()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|source| BackendError::CommandSpawn {
            tool: "swww",
            source,
            help: "Install with `sudo pacman -S swww` and start `swww-daemon`.",
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(BackendError::SwwwFailed {
            code: status.code(),
        }
        .into())
    }
}

fn apply_sway(path: &Path) -> Result<(), SwpError> {
    let _ = Command::new("pkill")
        .args(["-x", "swaybg"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    Command::new("swaybg")
        .args(["-i", &path.to_string_lossy(), "-m", "fill"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|source| BackendError::CommandSpawn {
            tool: "swaybg",
            source,
            help: "Install with `sudo pacman -S swaybg`.",
        })?;

    Ok(())
}

fn apply_feh(path: &Path) -> Result<(), SwpError> {
    let status = Command::new("feh")
        .args(["--bg-fill", &path.to_string_lossy()])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|source| BackendError::CommandSpawn {
            tool: "feh",
            source,
            help: "Install with `sudo pacman -S feh`.",
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(BackendError::CommandFailed {
            tool: "feh",
            code: status.code(),
        }
        .into())
    }
}