use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SwpError {
    #[error("HOME environment variable not set")]
    HomeEnvMissing,

    #[error("Error creating wallpapers directory: {source}")]
    CreateWallpapersDir {
        #[source]
        source: io::Error,
    },

    #[error("Wallpaper source not found: {path}")]
    WallpaperSourceNotFound { path: PathBuf },

    #[error("Wallpaper source is not a file: {path}")]
    WallpaperSourceNotFile { path: PathBuf },

    #[error("Unsupported wallpaper format: {path}")]
    UnsupportedWallpaperFormat { path: PathBuf },

    #[error("Failed to copy wallpaper from {from} to {to}: {source}")]
    CopyWallpaper {
        from: PathBuf,
        to: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error("Failed to remove wallpaper {path}: {source}")]
    RemoveWallpaper {
        path: PathBuf,
        #[source]
        source: io::Error,
    },

    #[error(transparent)]
    Query(#[from] QueryError),

    #[error(transparent)]
    Backend(#[from] BackendError),

    #[error(transparent)]
    Interval(#[from] IntervalError),

    #[error(transparent)]
    Selection(#[from] SelectionError),

    #[error(transparent)]
    List(#[from] ListError),

    #[error(transparent)]
    Playback(#[from] PlaybackError),

    #[error("Thumbnail error: {message}")]
    Thumbnail { message: String },
}

impl Clone for SwpError {
    fn clone(&self) -> Self {
        match self {
            Self::HomeEnvMissing => Self::HomeEnvMissing,
            Self::CreateWallpapersDir { source } => Self::CreateWallpapersDir {
                source: io::Error::new(source.kind(), source.to_string()),
            },
            Self::WallpaperSourceNotFound { path } => Self::WallpaperSourceNotFound {
                path: path.clone(),
            },
            Self::WallpaperSourceNotFile { path } => Self::WallpaperSourceNotFile {
                path: path.clone(),
            },
            Self::UnsupportedWallpaperFormat { path } => Self::UnsupportedWallpaperFormat {
                path: path.clone(),
            },
            Self::CopyWallpaper { from, to, source } => Self::CopyWallpaper {
                from: from.clone(),
                to: to.clone(),
                source: io::Error::new(source.kind(), source.to_string()),
            },
            Self::RemoveWallpaper { path, source } => Self::RemoveWallpaper {
                path: path.clone(),
                source: io::Error::new(source.kind(), source.to_string()),
            },
            Self::Query(err) => Self::Query(err.clone()),
            Self::Backend(err) => Self::Backend(err.clone()),
            Self::Interval(err) => Self::Interval(err.clone()),
            Self::Selection(err) => Self::Selection(err.clone()),
            Self::List(err) => Self::List(err.clone()),
            Self::Playback(err) => Self::Playback(err.clone()),
            Self::Thumbnail { message } => Self::Thumbnail {
                message: message.clone(),
            },
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum QueryError {
    #[error("No wallpaper found for '{query}'.\nUse `swp list` to see available ones.")]
    NoMatches { query: String },

    #[error("No wallpaper found. Add images to `swp path` first.")]
    NoImagesFound,

    #[error("No wallpaper found in {dir}.\nAdd images and try again (`swp list` to check).")]
    NoImagesInDir { dir: PathBuf },
}

#[derive(Debug, Error)]
pub enum BackendError {
    #[error(
        "Unrecognized desktop environment (XDG_CURRENT_DESKTOP='{desktop}'). Supported environments: KDE, GNOME/Cinnamon, Hyprland (swww), Sway (swaybg), X11 (feh)."
    )]
    UnknownDesktop { desktop: String },

    #[error(
        "Wayland environment detected, but no compatible backend found. Install swww and start swww-daemon."
    )]
    WaylandNoCompatibleBackend,

    #[error("Failed to execute {tool}: {source}\n{help}")]
    CommandSpawn {
        tool: &'static str,
        #[source]
        source: io::Error,
        help: &'static str,
    },

    #[error("{tool} failed (code {code:?})")]
    CommandFailed {
        tool: &'static str,
        code: Option<i32>,
    },

    #[error("swww failed (code {code:?}). Make sure swww-daemon is running.")]
    SwwwFailed { code: Option<i32> },
}

impl Clone for BackendError {
    fn clone(&self) -> Self {
        match self {
            Self::UnknownDesktop { desktop } => Self::UnknownDesktop {
                desktop: desktop.clone(),
            },
            Self::WaylandNoCompatibleBackend => Self::WaylandNoCompatibleBackend,
            Self::CommandSpawn { tool, source, help } => Self::CommandSpawn {
                tool,
                source: io::Error::new(source.kind(), source.to_string()),
                help,
            },
            Self::CommandFailed { tool, code } => Self::CommandFailed { tool, code: *code },
            Self::SwwwFailed { code } => Self::SwwwFailed { code: *code },
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum IntervalError {
    #[error(
        "Invalid interval format: '{input}'. Use a number followed by s, m, or h (e.g., 30s, 10m, 2h)."
    )]
    InvalidFormat { input: String },

    #[error("Invalid number in '{input}'. Use a positive integer (e.g., 30s, 10m, 2h).")]
    InvalidNumber { input: String },

    #[error("The interval must be greater than zero.")]
    ZeroInterval,
}

#[derive(Debug, Error, Clone)]
pub enum SelectionError {
    #[error("Canceled.")]
    Canceled,

    #[error("Invalid selection.")]
    InvalidSelection,

    #[error("{message}")]
    PromptFailure { message: String },
}

#[derive(Debug, Error, Clone)]
pub enum ListError {
    #[error("Invalid list name '{name}'. Use letters, numbers, '-' and '_'.")]
    InvalidName { name: String },

    #[error("List '{name}' already exists.")]
    AlreadyExists { name: String },

    #[error("Wallpaper '{wallpaper}' is already in list '{name}'.")]
    AlreadyInList { name: String, wallpaper: String },

    #[error("List '{name}' not found. Use `swp lists` to see available lists.")]
    NotFound { name: String },

    #[error("List '{name}' is empty.")]
    Empty { name: String },

    #[error("Wallpaper path must be inside {base}: {path}")]
    WallpaperOutsideBase { path: PathBuf, base: PathBuf },

    #[error("Wallpaper input not found: '{input}'")]
    InvalidWallpaperInput { input: String },

    #[error("Storage error: {message}")]
    Storage { message: String },

    #[error("Index error: {message}")]
    Index { message: String },
}

#[derive(Debug, Error)]
pub enum PlaybackError {
    #[error("No active playback found.")]
    NoActivePlayback,

    #[error("Invalid playback state: {message}")]
    InvalidState { message: String },

    #[error("Failed to spawn playback process: {message}")]
    SpawnError { message: String },

    #[error("Failed to stop playback process {pid}: {message}")]
    KillError { pid: u32, message: String },
}

impl Clone for PlaybackError {
    fn clone(&self) -> Self {
        match self {
            Self::NoActivePlayback => Self::NoActivePlayback,
            Self::InvalidState { message } => Self::InvalidState {
                message: message.clone(),
            },
            Self::SpawnError { message } => Self::SpawnError {
                message: message.clone(),
            },
            Self::KillError { pid, message } => Self::KillError {
                pid: *pid,
                message: message.clone(),
            },
        }
    }
}
