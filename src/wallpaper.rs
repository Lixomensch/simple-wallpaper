use std::fs;
use std::path::{Path, PathBuf};

use rand::prelude::IndexedRandom;
use walkdir::WalkDir;

use crate::kde::set_wallpaper;
use crate::utils::is_image;

pub fn wallpaper_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").map_err(|_| "HOME não definida")?;
    let dir = PathBuf::from(home)
        .join(".local/share/simple-wallpaper/wallpapers");

    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn list_images(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .filter(|p| is_image(p))
        .collect()
}

pub fn find_by_name(dir: &Path, name: &str) -> Option<PathBuf> {
    list_images(dir)
        .into_iter()
        .find(|p| p.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n == name)
            .unwrap_or(false))
}

pub fn random_image(dir: &Path) -> Option<PathBuf> {
    let images = list_images(dir);
    let mut rng = rand::rng();
    images.choose(&mut rng).cloned()
}

pub fn set(path: &PathBuf) -> Result<(), String> {
    let dir = wallpaper_dir()?;

    let img = if path.exists() {
        path.clone()
    } else {
        find_by_name(&dir, &path.to_string_lossy())
            .ok_or("Imagem não encontrada na pasta de wallpapers")?
    };

    set_wallpaper(&img)
}

pub fn random() -> Result<(), String> {
    let dir = wallpaper_dir()?;
    let img = random_image(&dir).ok_or("Nenhuma imagem encontrada na pasta de wallpapers")?;
    set_wallpaper(&img)
}
