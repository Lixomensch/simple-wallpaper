use std::path::PathBuf;

use iced::widget::image::Handle;

#[derive(Debug, Clone)]
pub enum ThumbnailState {
    Pending,
    Loading,
    Ready,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct WallpaperItem {
    pub path: PathBuf,
    pub filename: String,
    pub thumbnail: Option<Handle>,
    pub thumb_state: ThumbnailState,
}

#[derive(Default)]
pub struct GuiApp {
    pub wallpaper_items: Vec<WallpaperItem>,
    pub status_message: String,
    pub error_message: Option<String>,
    pub is_loading_list: bool,
    pub is_applying: bool,
    pub selected: Option<PathBuf>,
}

impl GuiApp {
    pub fn new() -> Self {
        Self {
            wallpaper_items: Vec::new(),
            status_message: "Initializing GUI...".to_string(),
            error_message: None,
            is_loading_list: true,
            is_applying: false,
            selected: None,
        }
    }
}