use std::path::PathBuf;

use iced::Task;
use iced::widget::image::Handle;

use super::message::Message;
use super::state::{GuiApp, ThumbnailState, WallpaperItem};

pub fn update(state: &mut GuiApp, message: Message) -> Task<Message> {
    match message {
        Message::LoadWallpapers => {
            state.is_loading_list = true;
            state.error_message = None;
            state.wallpaper_items.clear();
            state.status_message = "Loading wallpapers...".to_string();

            Task::perform(
                async move {
                    crate::core::operations::list_wallpapers().map(|(_, images)| images)
                },
                Message::WallpapersLoaded,
            )
        }
        Message::WallpapersLoaded(result) => {
            state.is_loading_list = false;

            match result {
                Ok(images) => {
                    state.wallpaper_items = images
                        .into_iter()
                        .map(|path| WallpaperItem {
                            filename: display_name(&path),
                            path,
                            thumbnail: None,
                            thumb_state: ThumbnailState::Pending,
                        })
                        .collect();

                    state.status_message =
                        format!("Loaded {} wallpaper(s).", state.wallpaper_items.len());
                    state.error_message = None;
                    Task::done(Message::GenerateNextThumbnail)
                }
                Err(err) => {
                    state.wallpaper_items.clear();
                    state.status_message = "Failed to load wallpapers.".to_string();
                    state.error_message = Some(err.to_string());
                    Task::none()
                }
            }
        }
        Message::GenerateNextThumbnail => {
            let pending = state
                .wallpaper_items
                .iter_mut()
                .find(|item| matches!(item.thumb_state, ThumbnailState::Pending));

            if let Some(item) = pending {
                item.thumb_state = ThumbnailState::Loading;
                return Task::done(Message::GenerateThumbnail(item.path.clone()));
            }

            Task::none()
        }
        Message::GenerateThumbnail(path) => {
            let path_for_loaded = path.clone();
            let path_for_failed = path.clone();

            Task::perform(
                async move { crate::core::thumbnail::generate_or_get_thumbnail(path.as_path(), 256, 160) },
                move |result| match result {
                    Ok(data) => Message::ThumbnailLoaded(
                        path_for_loaded.clone(),
                        Handle::from_rgba(data.width, data.height, data.pixels),
                    ),
                    Err(err) => Message::ThumbnailFailed(path_for_failed.clone(), err.to_string()),
                },
            )
        }
        Message::ThumbnailLoaded(path, handle) => {
            if let Some(item) = state.wallpaper_items.iter_mut().find(|item| item.path == path) {
                item.thumbnail = Some(handle);
                item.thumb_state = ThumbnailState::Ready;
            }

            Task::done(Message::GenerateNextThumbnail)
        }
        Message::ThumbnailFailed(path, err) => {
            if let Some(item) = state.wallpaper_items.iter_mut().find(|item| item.path == path) {
                item.thumbnail = None;
                item.thumb_state = ThumbnailState::Failed(err);
            }

            Task::done(Message::GenerateNextThumbnail)
        }
        Message::ApplyRandom => {
            state.is_applying = true;
            state.error_message = None;
            state.status_message = "Applying random wallpaper...".to_string();

            Task::perform(
                async move { crate::core::operations::random_wallpaper() },
                Message::WallpaperApplied,
            )
        }
        Message::ApplySpecific(path) => {
            state.is_applying = true;
            state.selected = Some(path.clone());
            state.error_message = None;
            state.status_message = format!("Applying {}...", display_name(&path));

            Task::perform(
                async move { crate::core::operations::apply_wallpaper(path.as_path()) },
                Message::WallpaperApplied,
            )
        }
        Message::WallpaperApplied(result) => {
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
        Message::StartSlideshowPressed => {
            state.status_message = "Slideshow in GUI is not implemented yet.".to_string();
            state.error_message = None;
            Task::none()
        }
    }
}

fn display_name(path: &PathBuf) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .map_or_else(|| path.display().to_string(), ToOwned::to_owned)
}