use std::path::PathBuf;

use iced::widget::image::Handle;

use crate::error::SwpError;

#[derive(Debug, Clone)]
pub enum Message {
    LoadWallpapers,
    WallpapersLoaded(Result<Vec<PathBuf>, SwpError>),
    GenerateNextThumbnail,
    GenerateThumbnail(PathBuf),
    ThumbnailLoaded(PathBuf, Handle),
    ThumbnailFailed(PathBuf, String),
    ApplyRandom,
    ApplySpecific(PathBuf),
    WallpaperApplied(Result<PathBuf, SwpError>),
    StartSlideshowPressed,
}