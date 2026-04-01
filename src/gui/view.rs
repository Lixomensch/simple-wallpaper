use iced::widget::{
    button,
    column,
    container,
    image,
    row,
    scrollable,
    text,
    Space,
};
use iced::{
    Alignment,
    Element,
    Fill,
    Length,
};

use super::message::{Message, WallpaperMessage, SlideshowMessage};
use super::state::{GuiApp, ThumbnailState, WallpaperItem};

pub fn view(state: &GuiApp) -> Element<'_, Message> {
    let wallpaper_state = &state.wallpapers;
    let is_busy = state.is_busy();

    let random_button = if is_busy {
        button("Random")
    } else {
        button("Random").on_press_with(|| Message::Wallpapers(WallpaperMessage::ApplyRandom))
    };

    let refresh_button = if is_busy {
        button("Refresh List")
    } else {
        button("Refresh List").on_press_with(|| Message::Wallpapers(WallpaperMessage::LoadWallpapers))
    };

    let slideshow_button = 
        button("Start Slideshow").on_press_with(|| Message::Slideshow(SlideshowMessage::StartSlideshowPressed));

    let sidebar = column![
        random_button,
        slideshow_button,
        refresh_button,
        text("Actions").size(14),
    ]
    .spacing(12)
    .padding(16)
    .width(Length::Fixed(220.0));

    let content: Element<'_, Message> = if wallpaper_state.is_loading_list {
        container(text("Loading wallpapers...").size(22))
            .center(Fill)
            .into()
    } else if wallpaper_state.items.is_empty() {
        container(text("No wallpapers found.").size(20))
            .center(Fill)
            .into()
    } else {
        const COLUMNS: usize = 3;

        let mut grid = column!()
            .spacing(12)
            .padding(12)
            .width(Length::Fill)
            .height(Length::Shrink);

        for chunk in wallpaper_state.items.chunks(COLUMNS) {
            let row_widget = chunk
                .iter()
                .fold(row!().spacing(12), |row_acc, item| {
                    row_acc.push(thumbnail_card(item, wallpaper_state.is_applying))
                })
                .height(Length::Shrink);

            grid = grid.push(row_widget);
        }

        scrollable(container(grid).height(Length::Shrink))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    let status_text = if let Some(error) = &wallpaper_state.error_message {
        format!("Error: {error}")
    } else {
        wallpaper_state.status_message.clone()
    };

    let main = row![sidebar, content]
        .spacing(0)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_y(Alignment::Start);

    let status_bar = container(text(status_text).size(14))
        .width(Length::Fill)
        .padding(10);

    // Add slideshow panel if active
        let slideshow_panel = slideshow_panel(state);

        let layout = if let Some(panel) = slideshow_panel {
            column![main, panel, status_bar]
                .spacing(6)
                .height(Length::Fill)
                .width(Length::Fill)
        } else {
            column![main, status_bar]
                .spacing(6)
                .height(Length::Fill)
                .width(Length::Fill)
        };

        // Add slideshow dialog if shown (renders as overlay)
        if let Some(dialog) = slideshow_dialog(state) {
            dialog
        } else {
            container(layout)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
}

fn thumbnail_card(item: &WallpaperItem, disable_actions: bool) -> Element<'_, Message> {
    let thumb: Element<'_, Message> = match (&item.thumbnail, &item.thumb_state) {
        (Some(handle), ThumbnailState::Ready) => image(handle.clone())
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(140.0))
            .into(),
        (_, ThumbnailState::Loading) => container(text("Loading thumbnail...").size(14))
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(140.0))
            .center(Fill)
            .into(),
        (_, ThumbnailState::Failed(reason)) => container(text(format!("Preview unavailable\n{reason}")).size(12))
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(140.0))
            .center(Fill)
            .into(),
        _ => container(text("Preview pending...").size(14))
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(140.0))
            .center(Fill)
            .into(),
    };

    let card_content = column![
        thumb,
        Space::with_height(6),
        text(&item.filename).size(14),
    ]
    .width(Length::Fixed(220.0))
    .height(Length::Shrink);

    if disable_actions {
        container(card_content)
            .width(Length::Fixed(220.0))
            .height(Length::Shrink)
            .into()
    } else {
        let path = item.path.clone();
        button(card_content)
            .width(Length::Fixed(220.0))
            .height(Length::Shrink)
            .on_press_with(move || Message::Wallpapers(WallpaperMessage::ApplySpecific(path.clone())))
            .into()
    }
}

/// Render slideshow start dialog (modal overlay).
pub fn slideshow_dialog(state: &GuiApp) -> Option<Element<'_, Message>> {
    if !state.show_slideshow_dialog {
        return None;
    }

    // Get list of available lists for dropdown
    let lists_text = if state.lists.lists.is_empty() {
        "No lists available".to_string()
    } else {
        format!("{} list(s) available", state.lists.lists.len())
    };

    let list_name = state
        .slideshow_dialog_list
        .as_deref()
        .unwrap_or("Select a list...");

    let dialog_content = column![
        text("Start Slideshow").size(18),
        text("Select a list and interval:").size(12),
        text(format!("List: {}", list_name)).size(14),
        text(format!("Interval: {}", state.slideshow_dialog_interval)).size(14),
        text(lists_text).size(12),
        row![
            button("Start")
                .on_press_with(|| Message::Slideshow(SlideshowMessage::ConfirmStartSlideshow)),
            button("Cancel")
                .on_press_with(|| Message::Slideshow(SlideshowMessage::CancelSlideshowDialog)),
        ]
        .spacing(10),
    ]
    .spacing(10)
    .padding(16);

    Some(
        container(dialog_content)
            .center(Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    )
}

/// Render slideshow control panel (shown when slideshow is active).
pub fn slideshow_panel(state: &GuiApp) -> Option<Element<'_, Message>> {
    if !state.slideshow.is_active {
        return None;
    }

    let pause_resume_label = if state.slideshow.is_paused {
        "Resume"
    } else {
        "Pause"
    };

    let pause_resume_msg = if state.slideshow.is_paused {
        SlideshowMessage::ResumePressed
    } else {
        SlideshowMessage::PausePressed
    };

    let list_name = state
        .slideshow
        .current_list
        .as_deref()
        .unwrap_or("Unknown");

    let panel = column![
        text("🎬 Slideshow Active").size(16),
        text(format!("List: {}", list_name)).size(12),
        text(format!(
            "Wallpaper #{} | Interval: {}s",
            state.slideshow.cycle_count, state.slideshow.current_interval
        ))
        .size(12),
        row![
            button(pause_resume_label)
                .on_press_with(move || Message::Slideshow(pause_resume_msg.clone())),
            button("Stop")
                .on_press_with(|| Message::Slideshow(SlideshowMessage::StopPressed)),
        ]
        .spacing(10),
    ]
    .spacing(8)
    .padding(12)
    .width(Length::Fill);

    Some(
        container(panel)
            .width(Length::Fill)
            .padding(8)
            .into(),
    )
}