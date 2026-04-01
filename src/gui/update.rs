use super::handlers;
use super::message::{ListsMessage, Message, PageMessage, SlideshowMessage};
use super::state::{GuiApp, GuiPage};
use iced::Task;

pub fn update(app: &mut GuiApp, message: Message) -> Task<Message> {
    match message {
        Message::Wallpapers(wmsg) => {
            handlers::wallpaper::update(&mut app.wallpapers, wmsg).map(Message::Wallpapers)
        }
        Message::Lists(ListsMessage::PlayListPressed(list_name, interval_secs)) => {
            app.show_slideshow_dialog = false;
            app.slideshow_dialog_list = None;
            app.slideshow_dialog_error = None;

            handlers::slideshow::update(
                &mut app.slideshow,
                SlideshowMessage::StartWithList(list_name, format!("{}s", interval_secs)),
            )
            .map(Message::Slideshow)
        }
        Message::Lists(lmsg) => handlers::lists::update(&mut app.lists, lmsg).map(Message::Lists),
        Message::Slideshow(smsg) => {
            match &smsg {
                SlideshowMessage::StartSlideshowPressed => {
                    app.show_slideshow_dialog = true;

                    if let Some((list_name, _)) = app.lists.lists.first() {
                        app.slideshow_dialog_list = Some(list_name.clone());
                    }

                    app.slideshow_dialog_error = handlers::slideshow::validate_interval_input(
                        &app.slideshow_dialog_interval,
                    );

                    return Task::none();
                }
                SlideshowMessage::SelectListForSlideshow(list_name) => {
                    app.slideshow_dialog_list = Some(list_name.clone());

                    return Task::none();
                }
                SlideshowMessage::ChangeIntervalInput(interval) => {
                    app.slideshow_dialog_interval = interval.clone();

                    app.slideshow_dialog_error =
                        handlers::slideshow::validate_interval_input(interval);

                    return Task::none();
                }
                SlideshowMessage::CancelSlideshowDialog => {
                    app.show_slideshow_dialog = false;

                    app.slideshow_dialog_error = None;

                    return Task::none();
                }
                SlideshowMessage::ConfirmStartSlideshow => {
                    if app.slideshow_dialog_error.is_some() {
                        return Task::none();
                    }

                    if let Some(list_name) = &app.slideshow_dialog_list {
                        let list = list_name.clone();

                        let interval = app.slideshow_dialog_interval.clone();

                        app.show_slideshow_dialog = false;

                        app.slideshow_dialog_error = None;

                        return handlers::slideshow::update(
                            &mut app.slideshow,
                            SlideshowMessage::StartWithList(list, interval),
                        )
                        .map(Message::Slideshow);
                    }

                    return Task::none();
                }
                _ => {}
            }

            handlers::slideshow::update(&mut app.slideshow, smsg).map(Message::Slideshow)
        }
        Message::Page(pmsg) => {
            match pmsg {
                PageMessage::OpenMain => app.current_page = GuiPage::Main,
                PageMessage::OpenCollections => app.current_page = GuiPage::Collections,
                PageMessage::NavigateTo(page) => app.current_page = page,
            }

            Task::none()
        }
    }
}

#[cfg(test)]

mod tests {

    use crate::gui::handlers::slideshow::validate_interval_input;

    #[test]

    fn accepts_plain_seconds() {
        assert_eq!(validate_interval_input("30"), None);
    }

    #[test]

    fn accepts_suffixed_intervals() {
        assert_eq!(validate_interval_input("5s"), None);

        assert_eq!(validate_interval_input("2m"), None);
    }

    #[test]

    fn rejects_empty_and_zero() {
        assert!(validate_interval_input("").is_some());

        assert!(validate_interval_input("0s").is_some());

        assert!(validate_interval_input("0").is_some());
    }

    #[test]

    fn rejects_invalid_formats() {
        assert!(validate_interval_input("abc").is_some());

        assert!(validate_interval_input("5x").is_some());
    }
}
