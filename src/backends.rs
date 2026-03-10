//! Desktop environment backends for applying wallpapers.
//!
//! Detects the current DE/WM at runtime and delegates to the appropriate
//! setter tool.  Supported environments (Arch Linux focus):
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
use std::process::Command;

fn detect_desktop() -> String {
    std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default()
        .to_uppercase()
}

pub fn apply(path: &Path) -> Result<(), String> {
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
        apply_swww(path).map_err(|_| {
            "Ambiente Wayland detectado, mas nenhum backend compatível encontrado. \
             Instale swww e inicie swww-daemon."
                .to_string()
        })
    } else if std::env::var("DISPLAY").is_ok() {
        apply_feh(path)
    } else {
        Err(format!(
            "Ambiente de desktop não reconhecido \
             (XDG_CURRENT_DESKTOP='{}'). \
             Ambientes suportados: KDE, GNOME/Cinnamon, Hyprland (swww), \
             Sway (swaybg), X11 (feh).",
            std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default()
        ))
    }
}

fn apply_kde(path: &Path) -> Result<(), String> {
    let status = Command::new("plasma-apply-wallpaperimage")
        .arg(path)
        .status()
        .map_err(|e| {
            format!(
                "Falha ao executar plasma-apply-wallpaperimage: {e}\n\
                 Verifique se plasma-workspace está instalado."
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "plasma-apply-wallpaperimage falhou (código {:?})",
            status.code()
        ))
    }
}

fn apply_gnome(path: &Path) -> Result<(), String> {
    let uri = format!("file://{}", path.display());

    let status = Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.background",
            "picture-uri",
            &uri,
        ])
        .status()
        .map_err(|e| format!("Falha ao executar gsettings: {e}"))?;

    if !status.success() {
        return Err(format!(
            "gsettings falhou (código {:?})",
            status.code()
        ));
    }

    let _ = Command::new("gsettings")
        .args([
            "set",
            "org.gnome.desktop.background",
            "picture-uri-dark",
            &uri,
        ])
        .status();

    Ok(())
}

fn apply_swww(path: &Path) -> Result<(), String> {
    let status = Command::new("swww")
        .args(["img", &path.to_string_lossy()])
        .status()
        .map_err(|e| {
            format!(
                "Falha ao executar swww: {e}\n\
                 Instale com `sudo pacman -S swww` e inicie `swww-daemon`."
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "swww falhou (código {:?}). \
             Certifique-se que swww-daemon está rodando.",
            status.code()
        ))
    }
}

fn apply_sway(path: &Path) -> Result<(), String> {
    let _ = Command::new("pkill").args(["-x", "swaybg"]).status();

    Command::new("swaybg")
        .args(["-i", &path.to_string_lossy(), "-m", "fill"])
        .spawn()
        .map_err(|e| {
            format!(
                "Falha ao executar swaybg: {e}\n\
                 Instale com `sudo pacman -S swaybg`."
            )
        })?;

    Ok(())
}

fn apply_feh(path: &Path) -> Result<(), String> {
    let status = Command::new("feh")
        .args(["--bg-fill", &path.to_string_lossy()])
        .status()
        .map_err(|e| {
            format!(
                "Falha ao executar feh: {e}\n\
                 Instale com `sudo pacman -S feh`."
            )
        })?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("feh falhou (código {:?})", status.code()))
    }
}
