use iced::Task;

use super::message::{Message, SlideshowMessage};
use super::state::GuiApp;
use super::handlers;

/// Main update dispatcher: routes messages to appropriate handlers.
pub fn update(app: &mut GuiApp, message: Message) -> Task<Message> {
    match message {
        Message::Wallpapers(wmsg) => {
            handlers::wallpaper::update(&mut app.wallpapers, wmsg)
                .map(Message::Wallpapers)
        }
        Message::Lists(lmsg) => {
            handlers::lists::update(&mut app.lists, lmsg)
                .map(Message::Lists)
        }
        Message::Slideshow(smsg) => {
            // Handle dialog-related messages at app level
            match &smsg {
                SlideshowMessage::StartSlideshowPressed => {
                    app.show_slideshow_dialog = true;
                    // Default to first list if available
                    if let Some((list_name, _)) = app.lists.lists.first() {
                        app.slideshow_dialog_list = Some(list_name.clone());
                    }
                    return Task::none();
                }
                SlideshowMessage::SelectListForSlideshow(list_name) => {
                    app.slideshow_dialog_list = Some(list_name.clone());
                    return Task::none();
                }
                SlideshowMessage::ChangeIntervalInput(interval) => {
                    app.slideshow_dialog_interval = interval.clone();
                    return Task::none();
                }
                SlideshowMessage::CancelSlideshowDialog => {
                    app.show_slideshow_dialog = false;
                    return Task::none();
                }
                SlideshowMessage::ConfirmStartSlideshow => {
                    if let Some(list_name) = &app.slideshow_dialog_list {
                        let list = list_name.clone();
                        let interval = app.slideshow_dialog_interval.clone();
                        app.show_slideshow_dialog = false;
                        // Trigger the actual start
                        return handlers::slideshow::update(
                            &mut app.slideshow,
                            SlideshowMessage::StartWithList(list, interval),
                        )
                        .map(Message::Slideshow);
                    }
                    return Task::none();
                }
                _ => {} // Other messages handled by handler
            }
            
            handlers::slideshow::update(&mut app.slideshow, smsg)
                .map(Message::Slideshow)
        }
    }
}