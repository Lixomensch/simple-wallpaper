pub mod message;
pub mod runtime;
pub mod state;
pub mod update;
pub mod view;
pub mod handlers;  // Add handlers as public submodule

use iced::{application, Task};
use std::sync::OnceLock;

use crate::error::SwpError;

use message::Message;
use state::GuiApp;

/// Global tokio runtime for the GUI.
/// Initialized on first launch, provides context for async operations.
static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

/// Get the global tokio runtime handle.
pub fn get_runtime_handle() -> tokio::runtime::Handle {
    let rt = TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create tokio runtime")
    });
    rt.handle().clone()
}

pub fn launch() -> Result<(), SwpError> {
    // Initialize the tokio runtime early
    get_runtime_handle();

    application("swp", update::update, view::view)
        .run_with(|| {
            (
                GuiApp::new(),
                Task::done(Message::Wallpapers(message::WallpaperMessage::LoadWallpapers)),
            )
        })
        .map_err(|e| SwpError::GuiLaunch {
            message: e.to_string(),
        })?;
    Ok(())
}
