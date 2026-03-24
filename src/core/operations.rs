use std::path::{Path, PathBuf};

use crate::backends;
use crate::core::wallpaper;

pub fn apply_wallpaper(path: &Path) -> Result<PathBuf, String> {
    let img = path.to_path_buf();
    backends::apply(&img)?;
    Ok(img)
}

pub fn random_wallpaper() -> Result<PathBuf, String> {
    let dir = wallpaper::wallpaper_dir()?;
    let img = wallpaper::random_image(&dir)
        .ok_or("No image found. Add images to `swp path` first.")?;
    backends::apply(&img)?;
    Ok(img)
}

pub fn list_wallpapers() -> Result<(PathBuf, Vec<PathBuf>), String> {
    let dir = wallpaper::wallpaper_dir()?;
    let mut images = wallpaper::list_images(&dir);
    images.sort();
    Ok((dir, images))
}

pub fn wallpaper_directory() -> Result<PathBuf, String> {
    wallpaper::wallpaper_dir()
}