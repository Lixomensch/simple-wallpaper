use std::fs;
use std::path::{Path, PathBuf};

use rand::RngExt;
use walkdir::WalkDir;

use crate::error::SwpError;

const EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "webp", "bmp", "avif"];

pub fn init() -> Result<(), SwpError> {
    let dir = wallpaper_dir()?;
    fs::create_dir_all(&dir).map_err(|source| SwpError::CreateWallpapersDir { source })?;
    Ok(())
}

pub fn wallpaper_dir() -> Result<PathBuf, SwpError> {
    let home = std::env::var("HOME").map_err(|_| SwpError::HomeEnvMissing)?;
    Ok(PathBuf::from(home).join(".local/share/swp/wallpapers"))
}

fn next_available_path(dir: &Path, file_name: &str) -> PathBuf {
    let candidate = dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("wallpaper");
    let ext = path.extension().and_then(|e| e.to_str());

    let mut counter = 1u32;
    loop {
        let file = match ext {
            Some(ext) => format!("{stem}_{counter}.{ext}"),
            None => format!("{stem}_{counter}"),
        };
        let candidate = dir.join(file);
        if !candidate.exists() {
            return candidate;
        }
        counter += 1;
    }
}

pub fn import_image(source: &Path) -> Result<PathBuf, SwpError> {
    if !source.exists() {
        return Err(SwpError::WallpaperSourceNotFound {
            path: source.to_path_buf(),
        });
    }

    if !source.is_file() {
        return Err(SwpError::WallpaperSourceNotFile {
            path: source.to_path_buf(),
        });
    }

    if !is_image(source) {
        return Err(SwpError::UnsupportedWallpaperFormat {
            path: source.to_path_buf(),
        });
    }

    let dir = wallpaper_dir()?;
    fs::create_dir_all(&dir).map_err(|source| SwpError::CreateWallpapersDir { source })?;

    let file_name = source
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("wallpaper");
    let destination = next_available_path(&dir, file_name);

    fs::copy(source, &destination).map_err(|source_err| SwpError::CopyWallpaper {
        from: source.to_path_buf(),
        to: destination.clone(),
        source: source_err,
    })?;

    Ok(destination)
}

pub fn remove_image(path: &Path) -> Result<(), SwpError> {
    if !path.exists() {
        return Err(SwpError::WallpaperSourceNotFound {
            path: path.to_path_buf(),
        });
    }

    if !path.is_file() {
        return Err(SwpError::WallpaperSourceNotFile {
            path: path.to_path_buf(),
        });
    }

    fs::remove_file(path).map_err(|source| SwpError::RemoveWallpaper {
        path: path.to_path_buf(),
        source,
    })
}

pub fn is_image(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| EXTENSIONS.iter().any(|ext| e.eq_ignore_ascii_case(ext)))
        .unwrap_or(false)
}

fn image_paths(dir: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_entry(|e| e.file_name() != "thumbnails")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.into_path();
            is_image(&path).then_some(path)
        })
}

pub fn list_images(dir: &Path) -> Vec<PathBuf> {
    image_paths(dir).collect()
}

pub fn random_image(dir: &Path) -> Option<PathBuf> {
    let mut rng = rand::rng();
    let mut selected: Option<PathBuf> = None;
    let mut count = 0u64;

    image_paths(dir).for_each(|path| {
        count += 1;
        // Reservoir sampling: select with probability 1/count
        if rng.random_range(0..count) == 0 {
            selected = Some(path);
        }
    });

    selected
}

fn matches_all_words(filename: &str, words: &[String]) -> bool {
    let lower_filename = filename.to_lowercase();
    words.iter().all(|word| lower_filename.contains(word))
}

pub fn find_by_words(dir: &Path, query: &str) -> Vec<PathBuf> {
    let words: Vec<String> = query
        .split_whitespace()
        .map(|part| part.to_lowercase())
        .collect();

    if words.is_empty() {
        return Vec::new();
    }

    image_paths(dir)
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| matches_all_words(n, &words))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{find_by_words, is_image, list_images, next_available_path};

    fn test_dir() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("swp-wallpaper-tests-{stamp}"));
        fs::create_dir_all(&dir).expect("temp test dir should be created");
        dir
    }

    #[test]
    fn is_image_matches_extensions_case_insensitively() {
        assert!(is_image(Path::new("a.JPG")));
        assert!(is_image(Path::new("a.pNg")));
        assert!(!is_image(Path::new("a.txt")));
    }

    #[test]
    fn list_images_ignores_thumbnail_directory() {
        let dir = test_dir();
        let thumbnails = dir.join("thumbnails");
        fs::create_dir_all(&thumbnails).expect("thumbnails dir should be created");
        fs::write(dir.join("kept.jpg"), b"x").expect("file should be writable");
        fs::write(thumbnails.join("ignored.jpg"), b"x").expect("file should be writable");

        let listed = list_images(&dir);
        assert_eq!(listed.len(), 1);
        assert!(listed.iter().any(|path| path.ends_with("kept.jpg")));

        fs::remove_dir_all(&dir).expect("temp test dir should be removable");
    }

    #[test]
    fn find_by_words_matches_all_query_terms() {
        let dir = test_dir();
        fs::write(dir.join("angel-night.png"), b"x").expect("file should be writable");
        fs::write(dir.join("angel-day.png"), b"x").expect("file should be writable");

        let found = find_by_words(&dir, "angel night");
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("angel-night.png"));

        fs::remove_dir_all(&dir).expect("temp test dir should be removable");
    }

    #[test]
    fn next_available_path_adds_incrementing_suffix() {
        let dir = test_dir();
        fs::write(dir.join("city.jpg"), b"x").expect("file should be writable");
        fs::write(dir.join("city_1.jpg"), b"x").expect("file should be writable");

        let path = next_available_path(&dir, "city.jpg");
        assert!(path.ends_with("city_2.jpg"));

        fs::remove_dir_all(&dir).expect("temp test dir should be removable");
    }

    #[test]
    fn remove_image_deletes_existing_file() {
        let dir = test_dir();
        let image = dir.join("remove-me.jpg");
        fs::write(&image, b"x").expect("file should be writable");

        super::remove_image(&image).expect("remove should succeed");
        assert!(!image.exists());

        fs::remove_dir_all(&dir).expect("temp test dir should be removable");
    }

    #[test]
    fn remove_image_fails_for_missing_file() {
        let dir = test_dir();
        let missing = dir.join("missing.jpg");

        let result = super::remove_image(&missing);
        assert!(result.is_err());

        fs::remove_dir_all(&dir).expect("temp test dir should be removable");
    }
}
