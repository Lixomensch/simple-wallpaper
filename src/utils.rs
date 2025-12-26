use std::path::Path;

const EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "avif"];

pub fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}
