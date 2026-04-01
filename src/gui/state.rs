use iced::widget::image::Handle;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuiPage {
    Main,
    Collections,
}

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

#[derive(Debug, Clone)]

pub struct WallpaperState {
    pub items: Vec<WallpaperItem>,
    pub status_message: String,
    pub error_message: Option<String>,
    pub is_loading_list: bool,
    pub is_applying: bool,
    pub selected: Option<PathBuf>,
}

impl Default for WallpaperState {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperState {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            status_message: "Ready".to_string(),
            error_message: None,
            is_loading_list: false,
            is_applying: false,
            selected: None,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.is_loading_list || self.is_applying
    }

    pub fn clear_messages(&mut self) {
        self.status_message.clear();

        self.error_message = None;
    }
}

#[derive(Debug, Clone)]

pub struct ListsState {
    pub lists: Vec<(String, usize)>,
    pub selected_list: Option<String>,
    pub is_editing: bool,
    pub error_message: Option<String>,
    pub status_message: String,
    pub selected_list_items: Vec<(String, String)>,
    pub new_list_name: String,
    pub pending_add_paths: Vec<std::path::PathBuf>,
    pub pending_remove_ids: Vec<String>,
    pub pending_delete_list: Option<String>,
    pub pending_remove_item: Option<(String, String, String)>,
}

impl ListsState {
    pub fn new() -> Self {
        Self {
            lists: Vec::new(),
            selected_list: None,
            is_editing: false,
            error_message: None,
            status_message: "Ready".to_string(),
            selected_list_items: Vec::new(),
            new_list_name: String::new(),
            pending_add_paths: Vec::new(),
            pending_remove_ids: Vec::new(),
            pending_delete_list: None,
            pending_remove_item: None,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.is_editing
    }
}

#[derive(Debug, Clone)]

pub struct SlideshowState {
    pub is_active: bool,
    pub current_list: Option<String>,
    pub message: Option<String>,
    pub is_paused: bool,
    pub current_interval: u64,
    pub currently_playing: Option<std::path::PathBuf>,
    pub cycle_count: u32,
    pub playlist_paths: Vec<std::path::PathBuf>,
    pub current_index: usize,
    pub run_id: u64,
}

impl Default for SlideshowState {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideshowState {
    pub fn new() -> Self {
        Self {
            is_active: false,
            current_list: None,
            message: None,
            is_paused: false,
            current_interval: 5,
            currently_playing: None,
            cycle_count: 0,
            playlist_paths: Vec::new(),
            current_index: 0,
            run_id: 0,
        }
    }

    pub fn reset(&mut self) {
        self.is_active = false;

        self.current_list = None;

        self.message = None;

        self.is_paused = false;

        self.currently_playing = None;

        self.cycle_count = 0;

        self.playlist_paths.clear();

        self.current_index = 0;
    }
}

#[derive(Debug, Clone)]

pub struct GuiApp {
    pub wallpapers: WallpaperState,
    pub lists: ListsState,
    pub slideshow: SlideshowState,
    pub show_slideshow_dialog: bool,
    pub slideshow_dialog_list: Option<String>,
    pub slideshow_dialog_interval: String,
    pub slideshow_dialog_error: Option<String>,
    pub current_page: GuiPage,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiApp {
    pub fn new() -> Self {
        Self {
            wallpapers: WallpaperState::new(),
            lists: ListsState::new(),
            slideshow: SlideshowState::new(),
            show_slideshow_dialog: false,
            slideshow_dialog_list: None,
            slideshow_dialog_interval: "5s".to_string(),
            slideshow_dialog_error: None,
            current_page: GuiPage::Main,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.wallpapers.is_busy() || self.lists.is_editing || self.slideshow.is_active
    }
}
