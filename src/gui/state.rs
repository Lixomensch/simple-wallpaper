use std::path::PathBuf;

use iced::widget::image::Handle;

/// State of a thumbnail: pending, loading, ready, or failed.
#[derive(Debug, Clone)]
pub enum ThumbnailState {
    /// Thumbnail not yet generated
    Pending,
    /// Thumbnail generation in progress
    Loading,
    /// Thumbnail ready and cached
    Ready,
    /// Thumbnail generation failed
    Failed(String),
}

/// A single wallpaper item with its metadata and thumbnail.
#[derive(Debug, Clone)]
pub struct WallpaperItem {
    /// Path to wallpaper file
    pub path: PathBuf,
    /// Display name (filename)
    pub filename: String,
    /// Cached thumbnail image handle (if ready)
    pub thumbnail: Option<Handle>,
    /// Current state of thumbnail generation
    pub thumb_state: ThumbnailState,
}

/// State for the wallpaper gallery: loading, applying, thumbnails.
#[derive(Debug, Clone)]
pub struct WallpaperState {
    /// All discovered wallpapers with their items
    pub items: Vec<WallpaperItem>,
    /// User-facing status message
    pub status_message: String,
    /// Error message (if any)
    pub error_message: Option<String>,
    /// True if currently loading wallpapers from disk
    pub is_loading_list: bool,
    /// True if currently applying a wallpaper
    pub is_applying: bool,
    /// Currently selected wallpaper (for apply operations)
    pub selected: Option<PathBuf>,
}

impl Default for WallpaperState {
    fn default() -> Self {
        Self::new()
    }
}

impl WallpaperState {
    /// Create a new WallpaperState with default values.
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

    /// Check if any operation is in progress.
    pub fn is_busy(&self) -> bool {
        self.is_loading_list || self.is_applying
    }

    /// Clear transient messages (status and error).
    pub fn clear_messages(&mut self) {
        self.status_message.clear();
        self.error_message = None;
    }
}

/// State for wallpaper list management (Fase 3).
///
/// Manages creation, editing, deletion, and playback of wallpaper collections.
#[derive(Debug, Clone)]
pub struct ListsState {
    /// All created lists (name + item count)
    pub lists: Vec<(String, usize)>,
    /// Currently selected list (if any)
    pub selected_list: Option<String>,
    /// True if in create/edit mode
    pub is_editing: bool,
    /// Error message (if any)
    pub error_message: Option<String>,
    /// Status message
    pub status_message: String,
    /// List items for selected list (id, path_display)
    pub selected_list_items: Vec<(String, String)>,
    /// For create list: pending name input
    pub new_list_name: String,
    /// For add: pending wallpapers to add
    pub pending_add_paths: Vec<std::path::PathBuf>,
    /// For remove: pending item ids to remove
    pub pending_remove_ids: Vec<String>,
}

impl ListsState {
    /// Create a new ListsState with default values.
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
        }
    }

    /// Check if any operation is in progress.
    pub fn is_busy(&self) -> bool {
        self.is_editing
    }
}

/// State for slideshow playback (Fase 4).
///
/// Manages automatic wallpaper cycling at configurable intervals.
#[derive(Debug, Clone)]
pub struct SlideshowState {
    /// True if slideshow is currently active
    pub is_active: bool,
    /// Current list being played (if applicable)
    pub current_list: Option<String>,
    /// Associated message/status
    pub message: Option<String>,
    /// True if slideshow is paused
    pub is_paused: bool,
    /// Current interval between wallpaper changes (in seconds)
    pub current_interval: u64,
    /// Currently displayed wallpaper in slideshow
    pub currently_playing: Option<std::path::PathBuf>,
    /// Number of wallpapers cycled through
    pub cycle_count: u32,
}

impl Default for SlideshowState {
    fn default() -> Self {
        Self::new()
    }
}

impl SlideshowState {
    /// Create a new SlideshowState with default values.
    pub fn new() -> Self {
        Self {
            is_active: false,
            current_list: None,
            message: None,
            is_paused: false,
            current_interval: 5, // Default 5 seconds
            currently_playing: None,
            cycle_count: 0,
        }
    }

    /// Reset slideshow state to idle.
    pub fn reset(&mut self) {
        self.is_active = false;
        self.current_list = None;
        self.message = None;
        self.is_paused = false;
        self.currently_playing = None;
        self.cycle_count = 0;
    }
}

/// Global application state containing all feature-specific states.
#[derive(Debug, Clone)]
pub struct GuiApp {
    /// Wallpaper gallery state (loading, applying, thumbnails)
    pub wallpapers: WallpaperState,
    /// List management state (Fase 3)
    pub lists: ListsState,
    /// Slideshow playback state (Fase 4)
    pub slideshow: SlideshowState,
    /// True if slideshow start dialog should be shown (Fase 5)
    pub show_slideshow_dialog: bool,
    /// Slideshow dialog: list name for starting slideshow (Fase 5)
    pub slideshow_dialog_list: Option<String>,
    /// Slideshow dialog: interval string input (Fase 5)
    pub slideshow_dialog_interval: String,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self::new()
    }
}

impl GuiApp {
    /// Create a new GuiApp with default/initial state.
    pub fn new() -> Self {
        Self {
            wallpapers: WallpaperState::new(),
            lists: ListsState::new(),
            slideshow: SlideshowState::new(),
            show_slideshow_dialog: false,
            slideshow_dialog_list: None,
            slideshow_dialog_interval: "5s".to_string(),
        }
    }

    /// Check if any operation is in progress.
    pub fn is_busy(&self) -> bool {
        self.wallpapers.is_busy() || self.lists.is_editing || self.slideshow.is_active
    }
}