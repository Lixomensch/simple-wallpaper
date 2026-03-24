pub mod message;
pub mod state;
pub mod update;
pub mod view;

use iced::{application, Task};

use crate::error::SwpError;

use message::Message;
use state::GuiApp;

pub fn launch() -> Result<(), SwpError> {
    application("swp", update::update, view::view)
        .run_with(|| {
            (
                GuiApp::new(),
                Task::done(Message::LoadWallpapers),
            )
        })
        .map_err(|e| SwpError::GuiLaunch {
            message: e.to_string(),
        })?;
    Ok(())
}
