use crate::error::SwpError;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

static ASYNC_BRIDGE_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

fn runtime_handle() -> tokio::runtime::Handle {
    ASYNC_BRIDGE_RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("failed to create async_bridge runtime")
        })
        .handle()
        .clone()
}

async fn run_blocking<T, F>(work: F) -> Result<T, SwpError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, SwpError> + Send + 'static,
{
    runtime_handle()
        .spawn_blocking(work)
        .await
        .map_err(|e| SwpError::GuiLaunch {
            message: format!("Task join error: {}", e),
        })?
}

pub async fn apply_wallpaper_async(path: &Path) -> Result<PathBuf, SwpError> {
    let path = path.to_path_buf();

    run_blocking(move || crate::core::operations::apply_wallpaper(&path)).await
}

pub async fn random_wallpaper_async() -> Result<PathBuf, SwpError> {
    run_blocking(crate::core::operations::random_wallpaper).await
}

pub async fn list_wallpapers_async() -> Result<Vec<PathBuf>, SwpError> {
    run_blocking(|| crate::core::operations::list_wallpapers().map(|(_, paths)| paths)).await
}

pub async fn generate_thumbnail_async(
    path: &Path,
    width: u16,
    height: u16,
) -> Result<crate::core::thumbnail::ThumbnailData, SwpError> {
    let path = path.to_path_buf();

    run_blocking(move || {
        crate::core::thumbnail::generate_or_get_thumbnail(&path, width as u32, height as u32)
    })
    .await
}

pub async fn list_lists_async() -> Result<Vec<crate::core::lists::ListSummary>, SwpError> {
    run_blocking(crate::core::lists::list_lists).await
}

pub async fn get_list_items_async(
    name: &str,
) -> Result<Vec<crate::core::lists::ListItem>, SwpError> {
    let name = name.to_string();

    run_blocking(move || crate::core::lists::get_list_items(&name)).await
}

pub async fn create_list_async(name: &str) -> Result<(), SwpError> {
    let name = name.to_string();

    run_blocking(move || crate::core::lists::create_list(&name)).await
}

pub async fn delete_list_async(name: &str) -> Result<(), SwpError> {
    let name = name.to_string();

    run_blocking(move || crate::core::lists::delete_list(&name)).await
}

pub async fn add_to_list_async(list_name: &str, paths: Vec<PathBuf>) -> Result<(), SwpError> {
    let list_name = list_name.to_string();

    run_blocking(move || {
        crate::core::lists::add_wallpapers_by_paths(&list_name, &paths).map(|_| ())
    })
    .await
}

pub async fn remove_from_list_async(
    list_name: &str,
    paths: Option<Vec<PathBuf>>,
    ids: Option<Vec<uuid::Uuid>>,
) -> Result<(), SwpError> {
    let list_name = list_name.to_string();

    run_blocking(move || {
        let paths_slice = paths.as_deref().unwrap_or(&[]);

        let ids_slice = ids.as_deref().unwrap_or(&[]);

        crate::core::lists::remove_wallpapers(&list_name, paths_slice, ids_slice).map(|_| ())
    })
    .await
}

pub async fn parse_interval_async(interval: &str) -> Result<std::time::Duration, SwpError> {
    let interval = interval.to_string();

    run_blocking(move || crate::core::lists_play::parse_interval(&interval)).await
}

pub async fn sleep_async(duration: Duration) -> Result<(), SwpError> {
    run_blocking(move || {
        std::thread::sleep(duration);

        Ok(())
    })
    .await
}
