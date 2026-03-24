use std::path::{Path, PathBuf};

use crate::backends;
use crate::core::wallpaper;
use crate::error::{QueryError, SwpError};

pub fn apply_wallpaper(path: &Path) -> Result<PathBuf, SwpError> {
    let img = path.to_path_buf();
    backends::apply(&img)?;
    Ok(img)
}

pub fn random_wallpaper() -> Result<PathBuf, SwpError> {
    let dir = wallpaper::wallpaper_dir()?;
    let img = wallpaper::random_image(&dir)
        .ok_or(QueryError::NoImagesFound)?;
    backends::apply(&img)?;
    Ok(img)
}

pub fn list_wallpapers() -> Result<(PathBuf, Vec<PathBuf>), SwpError> {
    let dir = wallpaper::wallpaper_dir()?;
    let mut images = wallpaper::list_images(&dir);
    images.sort();
    Ok((dir, images))
}

pub fn wallpaper_directory() -> Result<PathBuf, SwpError> {
    wallpaper::wallpaper_dir()
}