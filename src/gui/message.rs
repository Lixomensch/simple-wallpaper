use std::path::PathBuf;

use iced::widget::image::Handle;

use crate::error::SwpError;

/// Top-level message enum, routing to feature-specific message types.
#[derive(Debug, Clone)]
pub enum Message {
    /// Wallpaper operations (load, apply, thumbnails)
    Wallpapers(WallpaperMessage),
    /// List management operations (create, delete, add, remove, play)
    Lists(ListsMessage),
    /// Slideshow operations (start, pause, stop)
    Slideshow(SlideshowMessage),
}

/// Messages related to wallpaper selection, loading, and application.
#[derive(Debug, Clone)]
pub enum WallpaperMessage {
    /// Request to load all available wallpapers
    LoadWallpapers,
    /// Result of loading wallpapers
    WallpapersLoaded(Result<Vec<PathBuf>, SwpError>),
    /// Queue next thumbnail for generation
    GenerateNextThumbnail,
    /// Request to generate thumbnail for a specific path
    GenerateThumbnail(PathBuf),
    /// Result: thumbnail successfully generated
    ThumbnailLoaded(PathBuf, Handle),
    /// Result: thumbnail generation failed
    ThumbnailFailed(PathBuf, String),
    /// User clicked "Apply Random"
    ApplyRandom,
    /// User clicked a specific wallpaper (apply it)
    ApplySpecific(PathBuf),
    /// Result of applying a wallpaper  
    WallpaperApplied(Result<PathBuf, SwpError>),
}

/// Messages related to wallpaper list management.
///
/// Lists are collections of wallpapers that can be saved, shared, and played back.
#[derive(Debug, Clone)]
pub enum ListsMessage {
    /// Request to load all lists
    LoadLists,
    /// Result of loading lists
    ListsLoaded(Result<Vec<(String, usize)>, SwpError>),
    /// User clicked "Create List" - show form
    CreateListPressed,
    /// User input list name and confirm
    ConfirmCreateList(String),
    /// Result of creating list
    CreatedList(Result<(), SwpError>),
    /// User selected a list to view/edit
    SelectList(String),
    /// User clicked "Add Wallpapers" to list
    AddWallpapersPressed(String),
    /// User confirmed which wallpapers to add (list_name, paths)
    ConfirmedAddWallpapers(String, Vec<std::path::PathBuf>),
    /// Result of adding to list
    AddedToList(String, Result<(), SwpError>),
    /// User selected wallpapers to remove from list (list_name, item_ids)
    RemoveWallpapersPressed(String, Vec<String>),
    /// Result of removing from list
    RemovedFromList(String, Result<(), SwpError>),
    /// User clicked "Delete List" - show confirmation
    DeleteListPressed(String),
    /// User confirmed delete
    ConfirmedDeleteList(String),
    /// Result of deleting list
    DeletedList(String, Result<(), SwpError>),
    /// User clicked "Play" on a list (list_name, interval_seconds)
    PlayListPressed(String, u64),
}

/// Messages related to slideshow playback.
///
/// Slideshow allows automatic changing of wallpapers at fixed intervals.
#[derive(Debug, Clone)]
pub enum SlideshowMessage {
    /// User clicked "Start Slideshow" button - show dialog
    StartSlideshowPressed,
    /// User selected a list in the dialog
    SelectListForSlideshow(String),
    /// User changed interval in dialog
    ChangeIntervalInput(String),
    /// User confirmed start slideshow in dialog
    ConfirmStartSlideshow,
    /// Cancel slideshow dialog
    CancelSlideshowDialog,
    /// User started slideshow with specific list and interval (list_name, interval_str)
    StartWithList(String, String),
    /// Result: wallpaper successfully applied in slideshow cycle
    SlideApplied(Result<std::path::PathBuf, SwpError>),
    /// User clicked "Pause" button
    PausePressed,
    /// User clicked "Resume" button
    ResumePressed,
    /// User clicked "Stop" button
    StopPressed,
    /// User changed interval setting (interval_str)
    IntervalChanged(String),
}