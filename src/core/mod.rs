pub use crate as core;

pub mod backends;
pub mod error;
pub mod lists;
pub mod lists_play;
pub mod operations;
pub mod playback_manager;
pub mod playback_state;
pub mod query;
pub mod wallpaper;

pub use error::{BackendError, IntervalError, ListError, QueryError, SelectionError, SwpError};
