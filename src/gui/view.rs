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

use super::message::Message;
use super::state::{GuiApp, ThumbnailState, WallpaperItem};

pub fn view(state: &GuiApp) -> Element<'_, Message> {
    let random_button = if state.is_applying || state.is_loading_list {
        button("Random")
    } else {
        button("Random").on_press_with(|| Message::ApplyRandom)
    };

    let refresh_button = if state.is_applying || state.is_loading_list {
        button("Refresh List")
    } else {
        button("Refresh List").on_press_with(|| Message::LoadWallpapers)
    };

    let slideshow_button = button("Start Slideshow").on_press_with(|| Message::StartSlideshowPressed);

    let sidebar = column![
        random_button,
        slideshow_button,
        refresh_button,
        text("Actions").size(14),
    ]
    .spacing(12)
    .padding(16)
    .width(Length::Fixed(220.0));

    let content: Element<'_, Message> = if state.is_loading_list {
        container(text("Loading wallpapers...").size(22))
            .center(Fill)
            .into()
    } else if state.wallpaper_items.is_empty() {
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

        for chunk in state.wallpaper_items.chunks(COLUMNS) {
            let row_widget = chunk
                .iter()
                .fold(row!().spacing(12), |row_acc, item| {
                    row_acc.push(thumbnail_card(item, state.is_applying))
                })
                .height(Length::Shrink);

            grid = grid.push(row_widget);
        }

        scrollable(container(grid).height(Length::Shrink))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    let status_text = if let Some(error) = &state.error_message {
        format!("Error: {error}")
    } else {
        state.status_message.clone()
    };

    let main = row![sidebar, content]
        .spacing(0)
        .height(Length::Fill)
        .width(Length::Fill)
        .align_y(Alignment::Start);

    let status_bar = container(text(status_text).size(14))
        .width(Length::Fill)
        .padding(10);

    container(
        column![main, status_bar]
            .spacing(6)
            .height(Length::Fill)
            .width(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
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
            .on_press_with(move || Message::ApplySpecific(path.clone()))
            .into()
    }
}