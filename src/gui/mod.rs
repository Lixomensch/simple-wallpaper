pub mod handlers;
pub mod message;
pub mod runtime;
pub mod state;
pub mod update;
pub mod view;

use crate::error::SwpError;
use iced::{Task, application};
use message::Message;
use state::GuiApp;
use std::sync::OnceLock;

static TOKIO_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

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
    get_runtime_handle();

    application("swp", update::update, view::view)
        .run_with(|| {
            (
                GuiApp::new(),
                Task::batch(vec![
                    Task::done(Message::Wallpapers(
                        message::WallpaperMessage::LoadWallpapers,
                    )),
                    Task::done(Message::Lists(message::ListsMessage::LoadLists)),
                ]),
            )
        })
        .map_err(|e| SwpError::GuiLaunch {
            message: e.to_string(),
        })?;

    Ok(())
}
