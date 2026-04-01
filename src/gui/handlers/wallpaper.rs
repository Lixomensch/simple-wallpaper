use crate::gui::message::WallpaperMessage;
use crate::gui::state::{ThumbnailState, WallpaperItem, WallpaperState};
use iced::Task;
use iced::widget::image::Handle;
use std::path::PathBuf;

pub fn update(state: &mut WallpaperState, message: WallpaperMessage) -> Task<WallpaperMessage> {
    match message {
        WallpaperMessage::LoadWallpapers => {
            state.is_loading_list = true;

            state.error_message = None;

            state.items.clear();

            state.status_message = "Loading wallpapers...".to_string();

            Task::perform(
                async move { crate::core::async_bridge::list_wallpapers_async().await },
                WallpaperMessage::WallpapersLoaded,
            )
        }
        WallpaperMessage::WallpapersLoaded(result) => {
            state.is_loading_list = false;

            match result {
                Ok(images) => {
                    state.items = images
                        .into_iter()
                        .map(|path| WallpaperItem {
                            filename: display_name(&path),
                            path,
                            thumbnail: None,
                            thumb_state: ThumbnailState::Pending,
                        })
                        .collect();

                    state.status_message = format!("Loaded {} wallpaper(s).", state.items.len());

                    state.error_message = None;

                    Task::done(WallpaperMessage::GenerateNextThumbnail)
                }
                Err(err) => {
                    state.items.clear();

                    state.status_message = "Failed to load wallpapers.".to_string();

                    state.error_message = Some(err.to_string());

                    Task::none()
                }
            }
        }
        WallpaperMessage::GenerateNextThumbnail => {
            let pending = state
                .items
                .iter_mut()
                .find(|item| matches!(item.thumb_state, ThumbnailState::Pending));

            if let Some(item) = pending {
                item.thumb_state = ThumbnailState::Loading;

                return Task::done(WallpaperMessage::GenerateThumbnail(item.path.clone()));
            }

            Task::none()
        }
        WallpaperMessage::GenerateThumbnail(path) => {
            let path_for_loaded = path.clone();

            let path_for_failed = path.clone();

            Task::perform(
                async move {
                    crate::core::async_bridge::generate_thumbnail_async(path.as_path(), 256, 160)
                        .await
                },
                move |result| match result {
                    Ok(data) => WallpaperMessage::ThumbnailLoaded(
                        path_for_loaded.clone(),
                        Handle::from_rgba(data.width, data.height, data.pixels),
                    ),
                    Err(err) => {
                        WallpaperMessage::ThumbnailFailed(path_for_failed.clone(), err.to_string())
                    }
                },
            )
        }
        WallpaperMessage::ThumbnailLoaded(path, handle) => {
            if let Some(item) = state.items.iter_mut().find(|item| item.path == path) {
                item.thumbnail = Some(handle);

                item.thumb_state = ThumbnailState::Ready;
            }

            Task::done(WallpaperMessage::GenerateNextThumbnail)
        }
        WallpaperMessage::ThumbnailFailed(path, err) => {
            if let Some(item) = state.items.iter_mut().find(|item| item.path == path) {
                item.thumbnail = None;

                item.thumb_state = ThumbnailState::Failed(err);
            }

            Task::done(WallpaperMessage::GenerateNextThumbnail)
        }
        WallpaperMessage::ApplyRandom => {
            state.is_applying = true;

            state.error_message = None;

            state.status_message = "Applying random wallpaper...".to_string();

            Task::perform(
                async move { crate::core::async_bridge::random_wallpaper_async().await },
                WallpaperMessage::WallpaperApplied,
            )
        }
        WallpaperMessage::ApplySpecific(path) => {
            state.is_applying = true;

            state.selected = Some(path.clone());

            state.error_message = None;

            state.status_message = format!("Applying {}...", display_name(&path));

            Task::perform(
                async move { crate::core::async_bridge::apply_wallpaper_async(path.as_path()).await },
                WallpaperMessage::WallpaperApplied,
            )
        }
        WallpaperMessage::WallpaperApplied(result) => {
            state.is_applying = false;

            match result {
                Ok(path) => {
                    state.status_message = format!("Applied: {}", display_name(&path));

                    state.error_message = None;

                    Task::none()
                }
                Err(err) => {
                    state.status_message = "Failed to apply wallpaper.".to_string();

                    state.error_message = Some(err.to_string());

                    Task::none()
                }
            }
        }
    }
}

fn display_name(path: &PathBuf) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .map_or_else(|| path.display().to_string(), ToOwned::to_owned)
}
