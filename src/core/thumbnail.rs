use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use image::imageops::FilterType;

use crate::core::wallpaper;
use crate::error::SwpError;

#[derive(Debug, Clone)]
pub struct ThumbnailData {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

pub fn generate_or_get_thumbnail(
    original_path: &Path,
    max_width: u32,
    max_height: u32,
) -> Result<ThumbnailData, SwpError> {
    let cache_dir = thumbnails_dir()?;
    
    let cache_path = thumbnail_path_for(original_path, max_width, max_height, &cache_dir, "jpg");

    if cache_path.exists() {
        return load_thumbnail_data(&cache_path);
    }

    let image = image::open(original_path).map_err(|e| SwpError::Thumbnail {
        message: format!("Failed to open image {}: {e}", original_path.display()),
    })?;

    let resized = image
        .resize(max_width, max_height, FilterType::Triangle)
        .to_rgb8();

    let (width, height) = resized.dimensions();
    
    let pixels = resized.to_vec();

    let path_clone = cache_path.clone();
    std::thread::spawn(move || {
        let _ = resized.save(&path_clone);
    });

    Ok(ThumbnailData {
        width,
        height,
        pixels,
    })
}

fn thumbnails_dir() -> Result<PathBuf, SwpError> {
    let dir = wallpaper::wallpaper_dir()?.join("thumbnails");
    fs::create_dir_all(&dir).map_err(|e| SwpError::Thumbnail {
        message: format!(
            "Failed to create thumbnails directory {}: {e}",
            dir.display()
        ),
    })?;
    Ok(dir)
}

fn thumbnail_path_for(
    original_path: &Path,
    max_width: u32,
    max_height: u32,
    cache_dir: &Path,
    extension: &str,
) -> PathBuf {
    let canonical = original_path
        .canonicalize()
        .unwrap_or_else(|_| original_path.to_path_buf());

    let mut hasher = DefaultHasher::new();
    canonical.to_string_lossy().hash(&mut hasher);
    max_width.hash(&mut hasher);
    max_height.hash(&mut hasher);
    let digest = hasher.finish();

    // Usando a nova extensão (jpg)
    cache_dir.join(format!("{:x}_{}x{}.{}", digest, max_width, max_height, extension))
}

fn load_thumbnail_data(path: &Path) -> Result<ThumbnailData, SwpError> {
    let thumb = image::open(path).map_err(|e| SwpError::Thumbnail {
        message: format!("Failed to load thumbnail {}: {e}", path.display()),
    })?;
    
    let rgb = thumb.to_rgba8(); 
    let (width, height) = rgb.dimensions();

    Ok(ThumbnailData {
        width,
        height,
        pixels: rgb.into_raw(),
    })
}