use super::message::{ListsMessage, Message, PageMessage, SlideshowMessage, WallpaperMessage};
use super::state::{GuiApp, GuiPage, ThumbnailState, WallpaperItem};
use iced::widget::{Space, button, column, container, image, row, scrollable, text, text_input};
use iced::{Alignment, Element, Fill, Length};

pub fn view(state: &GuiApp) -> Element<'_, Message> {
    let is_busy = state.is_busy();

    let hero = hero_banner(state);

    let workspace = match state.current_page {
        GuiPage::Main => row![main_actions_panel(state, is_busy), gallery_panel(state)],
        GuiPage::Collections => {
            row![
                collections_actions_panel(state, is_busy),
                lists_panel(state)
            ]
        }
    }
    .spacing(16)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_y(Alignment::Start);

    let footer_status = status_bar(state);

    let layout = column![
        hero,
        workspace,
        slideshow_panel(state).unwrap_or_else(|| container(" ").height(Length::Shrink).into()),
        footer_status
    ]
    .spacing(16)
    .padding(16)
    .width(Length::Fill)
    .height(Length::Fill);

    if let Some(dialog) = slideshow_dialog(state) {
        dialog
    } else {
        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn hero_banner(state: &GuiApp) -> Element<'_, Message> {
    container(
        row![text("Simple Wallpaper").size(32), page_nav(state)]
            .spacing(12)
            .align_y(Alignment::Center),
    )
    .style(iced::widget::container::rounded_box)
    .padding(20)
    .width(Length::Fill)
    .into()
}

fn page_nav(state: &GuiApp) -> Element<'_, Message> {
    let main_button = if state.current_page == GuiPage::Main {
        button("Main")
    } else {
        button("Main")
            .style(iced::widget::button::primary)
            .on_press(Message::Page(PageMessage::OpenMain))
    };

    let collections_button = if state.current_page == GuiPage::Collections {
        button("Collections")
    } else {
        button("Collections")
            .style(iced::widget::button::primary)
            .on_press(Message::Page(PageMessage::OpenCollections))
    };

    row![main_button, collections_button].spacing(8).into()
}

fn main_actions_panel(state: &GuiApp, is_busy: bool) -> Element<'_, Message> {
    let random_button = primary_action_button(
        "Apply random",
        !is_busy,
        Message::Wallpapers(WallpaperMessage::ApplyRandom),
    );

    let refresh_button = primary_action_button(
        "Refresh wallpapers",
        !is_busy,
        Message::Wallpapers(WallpaperMessage::LoadWallpapers),
    );

    let selected_summary = state
        .wallpapers
        .selected
        .as_ref()
        .and_then(|path| path.file_name().and_then(|name| name.to_str()))
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| "No wallpaper selected".to_string());

    let controls = column![
        text("Actions").size(16),
        text("Main page actions").size(12),
        random_button,
        refresh_button,
        container(
            column![
                text("Current selection").size(14),
                text(selected_summary).size(12),
                text(if is_busy { "Busy" } else { "Ready" }).size(12),
            ]
            .spacing(6),
        )
        .style(iced::widget::container::rounded_box)
        .padding(12),
    ]
    .spacing(12)
    .width(Length::Fixed(260.0));

    panel_shell(controls, Some(260.0))
}

fn collections_actions_panel(state: &GuiApp, is_busy: bool) -> Element<'_, Message> {
    let slideshow_button = primary_action_button(
        "Start slideshow",
        !is_busy,
        Message::Slideshow(SlideshowMessage::StartSlideshowPressed),
    );

    let load_lists_button = primary_action_button(
        "Load lists",
        !is_busy,
        Message::Lists(ListsMessage::LoadLists),
    );

    let create_list_button = if is_busy {
        button("Create list")
    } else {
        let list_name = state.lists.new_list_name.clone();

        button("Create list")
            .style(iced::widget::button::primary)
            .on_press_with(move || {
                Message::Lists(ListsMessage::ConfirmCreateList(list_name.clone()))
            })
    };

    let selected_list = state
        .lists
        .selected_list
        .clone()
        .unwrap_or_else(|| "None".to_string());

    let controls = column![
        text("Actions").size(16),
        text("Collections page actions").size(12),
        slideshow_button,
        load_lists_button,
        container(
            column![
                text("Create collection").size(14),
                text_input("new list name", &state.lists.new_list_name)
                    .on_input(|value| Message::Lists(ListsMessage::UpdateNewListName(value))),
                create_list_button,
            ]
            .spacing(8),
        )
        .style(iced::widget::container::rounded_box)
        .padding(12),
        container(
            column![
                text("Selected collection").size(14),
                text(selected_list).size(12),
                text(format!("{} item(s)", state.lists.selected_list_items.len())).size(12),
            ]
            .spacing(6),
        )
        .style(iced::widget::container::rounded_box)
        .padding(12),
    ]
    .spacing(12)
    .width(Length::Fixed(260.0));

    panel_shell(controls, Some(260.0))
}

fn gallery_panel(state: &GuiApp) -> Element<'_, Message> {
    let wallpaper_state = &state.wallpapers;

    let header = column![
        text("Gallery").size(18),
        text(format!("{} wallpapers", wallpaper_state.items.len())).size(12),
    ]
    .spacing(4);

    let content: Element<'_, Message> = if wallpaper_state.is_loading_list {
        empty_state(
            "Loading wallpapers",
            "Scanning the library and building thumbnails...",
        )
    } else if wallpaper_state.items.is_empty() {
        empty_state(
            "No wallpapers found",
            "Add images to the wallpaper directory and refresh.",
        )
    } else {
        const COLUMNS: usize = 3;

        let mut grid = column!().spacing(12).width(Length::Fill);

        for chunk in wallpaper_state.items.chunks(COLUMNS) {
            let row_widget = chunk.iter().fold(row!().spacing(12), |row_acc, item| {
                row_acc.push(thumbnail_card(item, wallpaper_state.is_applying))
            });

            grid = grid.push(row_widget);
        }

        scrollable(grid)
            .height(Length::Fill)
            .width(Length::Fill)
            .into()
    };

    panel_shell(column![header, content].spacing(12), None)
}

fn lists_panel(state: &GuiApp) -> Element<'_, Message> {
    let lists_busy =
        state.lists.is_busy() || state.wallpapers.is_busy() || state.slideshow.is_active;

    let header = column![
        text("Collections").size(18),
        text(format!("{} list(s)", state.lists.lists.len())).size(12),
    ]
    .spacing(4);

    let list_items = state
        .lists
        .lists
        .iter()
        .fold(column!().spacing(8), |col, (name, count)| {
            let label = format!("{}", name);

            let name_for_select = name.clone();

            let name_for_delete = name.clone();

            let select_button = if lists_busy {
                button(text(label).size(13))
            } else {
                button(text(label).size(13)).on_press_with(move || {
                    Message::Lists(ListsMessage::SelectList(name_for_select.clone()))
                })
            };

            let delete_button = if lists_busy {
                button(text("Delete").size(12))
            } else {
                button(text("Delete").size(12)).on_press_with(move || {
                    Message::Lists(ListsMessage::DeleteListPressed(name_for_delete.clone()))
                })
            };

            let row_item = container(
                column![
                    row![select_button, delete_button].spacing(8),
                    text(format!("{} wallpaper(s)", count)).size(11),
                ]
                .spacing(6),
            )
            .style(iced::widget::container::rounded_box)
            .padding(10);

            col.push(row_item)
        });

    let mut body = column![header].spacing(12);

    body = body.push(scrollable(list_items).height(Length::Fixed(210.0)));

    if let Some(selected) = &state.lists.selected_list {
        body = body.push(selected_list_panel(state, selected.clone(), lists_busy));
    } else {
        body = body.push(empty_state(
            "No list selected",
            "Choose a collection to see its items and actions.",
        ));
    }

    if let Some(pending_delete) = &state.lists.pending_delete_list {
        let target = pending_delete.clone();

        body = body.push(
            container(
                column![
                    text("Delete confirmation").size(14),
                    text(format!("Delete '{}' permanently?", pending_delete)).size(12),
                    row![
                        button("Confirm")
                            .style(iced::widget::button::primary)
                            .on_press_with(move || Message::Lists(
                                ListsMessage::ConfirmedDeleteList(target.clone())
                            )),
                        button("Cancel")
                            .on_press_with(|| Message::Lists(ListsMessage::CancelDeleteList)),
                    ]
                    .spacing(8),
                ]
                .spacing(8),
            )
            .style(iced::widget::container::rounded_box)
            .padding(12),
        );
    }

    if let Some((_, _, rel_path)) = &state.lists.pending_remove_item {
        body = body.push(
            container(
                column![
                    text("Remove confirmation").size(14),
                    text(format!("Remove '{}' from list?", rel_path)).size(12),
                    row![
                        button("Confirm")
                            .style(iced::widget::button::primary)
                            .on_press_with(|| Message::Lists(ListsMessage::ConfirmRemoveWallpaper)),
                        button("Cancel")
                            .on_press_with(|| Message::Lists(ListsMessage::CancelRemoveWallpaper)),
                    ]
                    .spacing(8),
                ]
                .spacing(8),
            )
            .style(iced::widget::container::rounded_box)
            .padding(12),
        );
    }

    panel_shell(body, Some(340.0))
}

fn selected_list_panel(
    state: &GuiApp,
    selected: String,
    lists_busy: bool,
) -> Element<'static, Message> {
    let play_collection_button = if !lists_busy {
        let list_name = selected.clone();
        let interval_secs = state.slideshow.current_interval;

        button("Play collection")
            .style(iced::widget::button::primary)
            .on_press_with(move || {
                Message::Lists(ListsMessage::PlayListPressed(
                    list_name.clone(),
                    interval_secs,
                ))
            })
    } else {
        button("Play collection")
    };

    let add_selected_button = if !lists_busy {
        if let Some(path) = state.wallpapers.selected.clone() {
            let list_name = selected.clone();

            button("Add selected wallpaper")
                .style(iced::widget::button::primary)
                .on_press_with(move || {
                    Message::Lists(ListsMessage::ConfirmedAddWallpapers(
                        list_name.clone(),
                        vec![path.clone()],
                    ))
                })
        } else {
            button("Select a wallpaper first")
        }
    } else {
        button("Working...")
    };

    let items =
        state
            .lists
            .selected_list_items
            .iter()
            .fold(column!().spacing(8), |col, (id, rel_path)| {
                let list_name = selected.clone();

                let item_id = id.clone();

                let item_path_for_button = rel_path.clone();

                let item_path_label = rel_path.clone();

                let remove_button = if lists_busy {
                    button(text("Remove").size(12))
                } else {
                    button(text("Remove").size(12)).on_press_with(move || {
                        Message::Lists(ListsMessage::PromptRemoveWallpaper(
                            list_name.clone(),
                            item_id.clone(),
                            item_path_for_button.clone(),
                        ))
                    })
                };

                let row_item = container(
                    row![
                        text(item_path_label).size(12).width(Length::Fill),
                        remove_button
                    ]
                    .spacing(8)
                    .width(Length::Fill),
                )
                .style(iced::widget::container::rounded_box)
                .padding(10);

                col.push(row_item)
            });

    container(
        column![
            text(format!("Selected list: {}", selected)).size(14),
            play_collection_button,
            add_selected_button,
            text(format!("{} item(s)", state.lists.selected_list_items.len())).size(12),
            scrollable(items).height(Length::Fixed(280.0)),
        ]
        .spacing(10),
    )
    .style(iced::widget::container::rounded_box)
    .padding(12)
    .into()
}

fn status_bar(state: &GuiApp) -> Element<'_, Message> {
    let wallpaper_status = if let Some(error) = &state.wallpapers.error_message {
        format!("Gallery: {error}")
    } else {
        state.wallpapers.status_message.clone()
    };

    let lists_status = if let Some(error) = &state.lists.error_message {
        format!("Lists: {error}")
    } else {
        state.lists.status_message.clone()
    };

    let slideshow_status = state
        .slideshow
        .message
        .clone()
        .unwrap_or_else(|| "Slideshow idle".to_string());

    container(
        row![
            text(wallpaper_status).size(13),
            text("|").size(13),
            text(lists_status).size(13),
            text("|").size(13),
            text(slideshow_status).size(13),
        ]
        .spacing(10),
    )
    .style(iced::widget::container::rounded_box)
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn empty_state(title: impl Into<String>, subtitle: impl Into<String>) -> Element<'static, Message> {
    let title = title.into();

    let subtitle = subtitle.into();

    container(
        column![text(title).size(18), text(subtitle).size(12)]
            .spacing(6)
            .width(Length::Fill),
    )
    .style(iced::widget::container::rounded_box)
    .padding(20)
    .width(Length::Fill)
    .into()
}

fn panel_shell<'a>(
    content: impl Into<Element<'a, Message>>,
    fixed_width: Option<f32>,
) -> Element<'a, Message> {
    let panel = container(content)
        .style(iced::widget::container::rounded_box)
        .padding(16)
        .height(Length::Fill);

    if let Some(width) = fixed_width {
        panel.width(Length::Fixed(width)).into()
    } else {
        panel.width(Length::Fill).into()
    }
}

fn primary_action_button(label: &str, enabled: bool, message: Message) -> Element<'_, Message> {
    if enabled {
        button(label)
            .style(iced::widget::button::primary)
            .on_press(message)
            .into()
    } else {
        button(label).into()
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
        (_, ThumbnailState::Failed(reason)) => {
            container(text(format!("Preview unavailable\n{reason}")).size(12))
                .width(Length::Fixed(220.0))
                .height(Length::Fixed(140.0))
                .center(Fill)
                .into()
        }
        _ => container(text("Preview pending...").size(14))
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(140.0))
            .center(Fill)
            .into(),
    };

    let card_content = column![thumb, Space::with_height(6), text(&item.filename).size(14),]
        .width(Length::Fixed(220.0))
        .height(Length::Shrink);

    if disable_actions {
        container(card_content)
            .style(iced::widget::container::rounded_box)
            .width(Length::Fixed(220.0))
            .padding(10)
            .height(Length::Shrink)
            .into()
    } else {
        let path = item.path.clone();

        button(card_content)
            .style(iced::widget::button::primary)
            .width(Length::Fixed(220.0))
            .height(Length::Shrink)
            .on_press_with(move || {
                Message::Wallpapers(WallpaperMessage::ApplySpecific(path.clone()))
            })
            .into()
    }
}

pub fn slideshow_dialog(state: &GuiApp) -> Option<Element<'_, Message>> {
    if !state.show_slideshow_dialog {
        return None;
    }

    let lists_text = if state.lists.lists.is_empty() {
        "No lists available".to_string()
    } else {
        format!("{} list(s) available", state.lists.lists.len())
    };

    let list_name = state
        .slideshow_dialog_list
        .as_deref()
        .unwrap_or("Select a list...");

    let list_buttons = state
        .lists
        .lists
        .iter()
        .fold(column!().spacing(6), |col, (name, _)| {
            let name_for_select = name.clone();

            col.push(button(text(name.clone())).on_press_with(move || {
                Message::Slideshow(SlideshowMessage::SelectListForSlideshow(
                    name_for_select.clone(),
                ))
            }))
        });

    let dialog_content = column![
        text("Start Slideshow").size(18),
        text("Select a list and interval:").size(12),
        text(format!("List: {}", list_name)).size(14),
        scrollable(list_buttons).height(Length::Fixed(120.0)),
        text("Interval").size(12),
        text_input("5s", &state.slideshow_dialog_interval)
            .on_input(|value| Message::Slideshow(SlideshowMessage::ChangeIntervalInput(value))),
        if let Some(error) = &state.slideshow_dialog_error {
            text(error.as_str()).size(12)
        } else {
            text("").size(12)
        },
        text(lists_text).size(12),
        row![
            if state.slideshow_dialog_list.is_some() && state.slideshow_dialog_error.is_none() {
                button("Start")
                    .style(iced::widget::button::primary)
                    .on_press_with(|| Message::Slideshow(SlideshowMessage::ConfirmStartSlideshow))
            } else {
                button("Start")
            },
            button("Cancel")
                .on_press_with(|| Message::Slideshow(SlideshowMessage::CancelSlideshowDialog)),
        ]
        .spacing(10),
    ]
    .spacing(10)
    .padding(16);

    Some(
        container(dialog_content)
            .style(iced::widget::container::rounded_box)
            .center(Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
    )
}

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

    let list_name = state.slideshow.current_list.as_deref().unwrap_or("Unknown");

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
                .style(iced::widget::button::primary)
                .on_press_with(move || Message::Slideshow(pause_resume_msg.clone())),
            button("Stop").on_press_with(|| Message::Slideshow(SlideshowMessage::StopPressed)),
        ]
        .spacing(10),
    ]
    .spacing(8)
    .padding(12)
    .width(Length::Fill);

    Some(
        container(panel)
            .style(iced::widget::container::rounded_box)
            .width(Length::Fill)
            .padding(8)
            .into(),
    )
}
