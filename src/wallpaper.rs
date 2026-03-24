use std::fs;
use std::path::{Path, PathBuf};

use rand::prelude::IndexedRandom;
use walkdir::WalkDir;

use crate::error::SwpError;

const EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "avif"];

pub fn init() -> Result<(), SwpError> {
    let dir = wallpaper_dir()?;
    fs::create_dir_all(&dir)
        .map_err(|source| SwpError::CreateWallpapersDir { source })?;
    Ok(())
}

pub fn wallpaper_dir() -> Result<PathBuf, SwpError> {
    let home = std::env::var("HOME").map_err(|_| SwpError::HomeEnvMissing)?;
    Ok(PathBuf::from(home).join(".local/share/simple-wallpaper/wallpapers"))
}

pub fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

pub fn list_images(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
    .filter_entry(|e| e.file_name() != "thumbnails")
        .filter_map(Result::ok)
        .map(|e| e.path().to_path_buf())
        .filter(|p| is_image(p))
        .collect()
}

pub fn random_image(dir: &Path) -> Option<PathBuf> {
    let images = list_images(dir);
    let mut rng = rand::rng();
    images.choose(&mut rng).cloned()
}

fn matches_all_words(filename: &str, words: &[&str]) -> bool {
    let lower_filename = filename.to_lowercase();
    words.iter().all(|w| lower_filename.contains(&w.to_lowercase()))
}

pub fn find_by_words(dir: &Path, query: &str) -> Vec<PathBuf> {
    let words: Vec<&str> = query.split_whitespace().collect();

    if words.is_empty() {
        return Vec::new();
    }

    list_images(dir)
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| matches_all_words(n, &words))
        })
        .collect()
}