//! Async bridge to synchronous core functions via tokio's spawn_blocking.
//!
//! This module exposes the synchronous core APIs as async functions suitable for
//! use in async contexts (like Iced's Task system). All operations delegate to the
//! blocking core functions via tokio::task::spawn_blocking to avoid freezing the UI.

use crate::error::SwpError;
use std::path::{Path, PathBuf};

/// Apply a specific wallpaper asynchronously.
pub async fn apply_wallpaper_async(path: &Path) -> Result<PathBuf, SwpError> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        crate::core::operations::apply_wallpaper(&path)
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// Apply a random wallpaper asynchronously.
pub async fn random_wallpaper_async() -> Result<PathBuf, SwpError> {
    tokio::task::spawn_blocking(|| {
        crate::core::operations::random_wallpaper()
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// List all available wallpapers asynchronously.
pub async fn list_wallpapers_async() -> Result<Vec<PathBuf>, SwpError> {
    tokio::task::spawn_blocking(|| {
        crate::core::operations::list_wallpapers().map(|(_, paths)| paths)
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// Generate or retrieve a cached thumbnail asynchronously.
pub async fn generate_thumbnail_async(
    path: &Path,
    width: u16,
    height: u16,
) -> Result<crate::core::thumbnail::ThumbnailData, SwpError> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        crate::core::thumbnail::generate_or_get_thumbnail(&path, width as u32, height as u32)
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// List all wallpaper lists asynchronously.
pub async fn list_lists_async() -> Result<Vec<crate::core::lists::ListSummary>, SwpError> {
    tokio::task::spawn_blocking(|| {
        crate::core::lists::list_lists()
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// Get details of a specific wallpaper list asynchronously.
pub async fn get_list_items_async(name: &str) -> Result<Vec<crate::core::lists::ListItem>, SwpError> {
    let name = name.to_string();
    tokio::task::spawn_blocking(move || {
        crate::core::lists::get_list_items(&name)
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// Create a new wallpaper list asynchronously.
pub async fn create_list_async(name: &str) -> Result<(), SwpError> {
    let name = name.to_string();
    tokio::task::spawn_blocking(move || {
        crate::core::lists::create_list(&name)
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// Delete a wallpaper list asynchronously.
pub async fn delete_list_async(name: &str) -> Result<(), SwpError> {
    let name = name.to_string();
    tokio::task::spawn_blocking(move || {
        crate::core::lists::delete_list(&name)
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// Add wallpapers to a list asynchronously.
pub async fn add_to_list_async(list_name: &str, paths: Vec<PathBuf>) -> Result<(), SwpError> {
    let list_name = list_name.to_string();
    tokio::task::spawn_blocking(move || {
        crate::core::lists::add_wallpapers_by_paths(&list_name, &paths).map(|_| ())
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// Remove wallpapers from a list asynchronously.
pub async fn remove_from_list_async(
    list_name: &str,
    paths: Option<Vec<PathBuf>>,
    ids: Option<Vec<uuid::Uuid>>,
) -> Result<(), SwpError> {
    let list_name = list_name.to_string();
    tokio::task::spawn_blocking(move || {
        let paths_slice = paths.as_deref().unwrap_or(&[]);
        let ids_slice = ids.as_deref().unwrap_or(&[]);
        crate::core::lists::remove_wallpapers(&list_name, paths_slice, ids_slice).map(|_| ())
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}

/// Parse a duration interval string (e.g., "30s", "10m", "2h").
pub async fn parse_interval_async(interval: &str) -> Result<std::time::Duration, SwpError> {
    let interval = interval.to_string();
    tokio::task::spawn_blocking(move || {
        crate::core::lists_play::parse_interval(&interval)
    })
    .await
    .map_err(|e| SwpError::GuiLaunch {
        message: format!("Task join error: {}", e),
    })?
}
