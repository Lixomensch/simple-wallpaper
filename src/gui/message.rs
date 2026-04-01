use crate::error::SwpError;
use crate::gui::state::GuiPage;
use iced::widget::image::Handle;
use std::path::PathBuf;

#[derive(Debug, Clone)]

pub enum Message {
    Wallpapers(WallpaperMessage),
    Lists(ListsMessage),
    Slideshow(SlideshowMessage),
    Page(PageMessage),
}

#[derive(Debug, Clone)]
pub enum PageMessage {
    OpenMain,
    OpenCollections,
    NavigateTo(GuiPage),
}

#[derive(Debug, Clone)]

pub enum WallpaperMessage {
    LoadWallpapers,
    WallpapersLoaded(Result<Vec<PathBuf>, SwpError>),
    GenerateNextThumbnail,
    GenerateThumbnail(PathBuf),
    ThumbnailLoaded(PathBuf, Handle),
    ThumbnailFailed(PathBuf, String),
    ApplyRandom,
    ApplySpecific(PathBuf),
    WallpaperApplied(Result<PathBuf, SwpError>),
}

#[derive(Debug, Clone)]

pub enum ListsMessage {
    LoadLists,
    ListsLoaded(Result<Vec<(String, usize)>, SwpError>),
    ListItemsLoaded(String, Result<Vec<(String, String)>, SwpError>),
    UpdateNewListName(String),
    CreateListPressed,
    ConfirmCreateList(String),
    CreatedList(Result<(), SwpError>),
    SelectList(String),
    AddWallpapersPressed(String),
    ConfirmedAddWallpapers(String, Vec<std::path::PathBuf>),
    AddedToList(String, Result<(), SwpError>),
    PromptRemoveWallpaper(String, String, String),
    CancelRemoveWallpaper,
    ConfirmRemoveWallpaper,
    RemoveWallpapersPressed(String, Vec<String>),
    ConfirmedRemoveWallpapers(String, Vec<String>),
    RemovedFromList(String, Result<(), SwpError>),
    DeleteListPressed(String),
    CancelDeleteList,
    ConfirmedDeleteList(String),
    DeletedList(String, Result<(), SwpError>),
    PlayListPressed(String, u64),
}

#[derive(Debug, Clone)]

pub enum SlideshowMessage {
    StartSlideshowPressed,
    SelectListForSlideshow(String),
    ChangeIntervalInput(String),
    ConfirmStartSlideshow,
    CancelSlideshowDialog,
    StartWithList(String, String),
    PlaylistLoaded(u64, Result<Vec<std::path::PathBuf>, SwpError>),
    ApplyCurrent(u64),
    AppliedCurrent(u64, Result<std::path::PathBuf, SwpError>),
    Tick(u64),
    PausePressed,
    ResumePressed,
    StopPressed,
    IntervalChanged(String),
}
