use std::path::Path;
use std::process::Command;

pub fn set_wallpaper(path: &Path) -> Result<(), String> {
    if std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default() != "KDE" {
        return Err("Não está rodando no KDE Plasma".into());
    }

    let status = Command::new("plasma-apply-wallpaperimage")
        .arg(path)
        .status()
        .map_err(|_| "plasma-apply-wallpaperimage não encontrado")?;

    if status.success() {
        println!("✔ Wallpaper aplicado: {}", path.display());
        Ok(())
    } else {
        Err("Falha ao aplicar wallpaper".into())
    }
}
