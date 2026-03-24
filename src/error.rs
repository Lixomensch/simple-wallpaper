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

    #[error(transparent)]
    Query(#[from] QueryError),

    #[error(transparent)]
    Backend(#[from] BackendError),

    #[error(transparent)]
    Interval(#[from] IntervalError),

    #[error(transparent)]
    Selection(#[from] SelectionError),
}

#[derive(Debug, Error)]
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

    #[error("Wayland environment detected, but no compatible backend found. Install swww and start swww-daemon.")]
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
    SwwwFailed {
        code: Option<i32>,
    },
}

#[derive(Debug, Error)]
pub enum IntervalError {
    #[error("Invalid interval format: '{input}'. Use a number followed by s, m, or h (e.g., 30s, 10m, 2h).")]
    InvalidFormat { input: String },

    #[error("Invalid number in '{input}'. Use a positive integer (e.g., 30s, 10m, 2h).")]
    InvalidNumber { input: String },

    #[error("The interval must be greater than zero.")]
    ZeroInterval,
}

#[derive(Debug, Error)]
pub enum SelectionError {
    #[error("Canceled.")]
    Canceled,

    #[error("Invalid selection.")]
    InvalidSelection,

    #[error("{message}")]
    PromptFailure { message: String },
}
