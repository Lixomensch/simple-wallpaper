use crate::gui::message::ListsMessage;
use crate::gui::state::ListsState;
use iced::Task;
use uuid::Uuid;

pub fn update(state: &mut ListsState, message: ListsMessage) -> Task<ListsMessage> {
    match message {
        ListsMessage::LoadLists => {
            state.status_message = "Loading lists...".to_string();

            state.error_message = None;

            Task::perform(
                async move {
                    crate::core::async_bridge::list_lists_async()
                        .await
                        .map(|summaries| summaries.into_iter().map(|s| (s.name, s.count)).collect())
                },
                ListsMessage::ListsLoaded,
            )
        }
        ListsMessage::ListsLoaded(result) => {
            match result {
                Ok(lists) => {
                    state.lists = lists;

                    state.status_message = format!("Loaded {} list(s).", state.lists.len());

                    state.error_message = None;
                }
                Err(err) => {
                    state.status_message = "Failed to load lists.".to_string();

                    state.error_message = Some(err.to_string());
                }
            }

            Task::none()
        }
        ListsMessage::UpdateNewListName(name) => {
            state.new_list_name = name;

            Task::none()
        }
        ListsMessage::CreateListPressed => {
            state.is_editing = true;

            state.new_list_name.clear();

            state.status_message = "Enter list name and press Create.".to_string();

            Task::none()
        }
        ListsMessage::ConfirmCreateList(name) => {
            state.is_editing = false;

            let trimmed = name.trim().to_string();

            if trimmed.is_empty() {
                state.error_message = Some("List name cannot be empty.".to_string());

                return Task::none();
            }

            state.status_message = format!("Creating list '{}'...", trimmed);

            state.error_message = None;

            let name_clone = trimmed.clone();

            Task::perform(
                async move { crate::core::async_bridge::create_list_async(&name_clone).await },
                |result| ListsMessage::CreatedList(result),
            )
        }
        ListsMessage::CreatedList(result) => match result {
            Ok(()) => {
                state.status_message = "List created successfully.".to_string();

                state.error_message = None;

                Task::done(ListsMessage::LoadLists)
            }
            Err(err) => {
                state.status_message = "Failed to create list.".to_string();

                state.error_message = Some(err.to_string());

                Task::none()
            }
        },
        ListsMessage::SelectList(list_name) => {
            state.selected_list = Some(list_name.clone());

            state.status_message = format!("Loading list '{}'...", list_name);

            state.error_message = None;

            state.selected_list_items.clear();

            let list_name_for_task = list_name.clone();

            Task::perform(
                async move {
                    crate::core::async_bridge::get_list_items_async(&list_name_for_task)
                        .await
                        .map(|items| {
                            items
                                .into_iter()
                                .map(|item| {
                                    let id = item.id.to_string();

                                    let rel = item
                                        .relative_path
                                        .map(|p| p.display().to_string())
                                        .unwrap_or_else(|| "<missing path>".to_string());

                                    (id, rel)
                                })
                                .collect::<Vec<(String, String)>>()
                        })
                },
                move |result| ListsMessage::ListItemsLoaded(list_name.clone(), result),
            )
        }
        ListsMessage::ListItemsLoaded(list_name, result) => {
            match result {
                Ok(items) => {
                    state.selected_list_items = items;

                    state.status_message = format!(
                        "Loaded '{}' ({} item(s)).",
                        list_name,
                        state.selected_list_items.len()
                    );

                    state.error_message = None;
                }
                Err(err) => {
                    state.selected_list_items.clear();

                    state.status_message = format!("Failed to load list '{}'.", list_name);

                    state.error_message = Some(err.to_string());
                }
            }

            Task::none()
        }
        ListsMessage::DeleteListPressed(list_name) => {
            state.pending_delete_list = Some(list_name.clone());

            state.status_message = format!("Confirm delete for '{}'", list_name);

            state.error_message = None;

            Task::none()
        }
        ListsMessage::CancelDeleteList => {
            state.pending_delete_list = None;

            state.status_message = "Delete canceled.".to_string();

            Task::none()
        }
        ListsMessage::ConfirmedDeleteList(list_name) => {
            state.pending_delete_list = None;

            state.status_message = format!("Deleting '{}'...", list_name);

            let name_clone = list_name.clone();

            Task::perform(
                async move { crate::core::async_bridge::delete_list_async(&name_clone).await },
                move |result| ListsMessage::DeletedList(list_name.clone(), result),
            )
        }
        ListsMessage::DeletedList(name, result) => match result {
            Ok(()) => {
                state.selected_list = None;

                state.status_message = format!("List '{}' deleted.", name);

                state.error_message = None;

                state.pending_delete_list = None;

                Task::done(ListsMessage::LoadLists)
            }
            Err(err) => {
                state.status_message = "Failed to delete list.".to_string();

                state.error_message = Some(err.to_string());

                state.pending_delete_list = None;

                Task::none()
            }
        },
        ListsMessage::AddWallpapersPressed(_list_name) => {
            state.is_editing = true;

            state.pending_add_paths.clear();

            state.status_message = "Select wallpapers to add to this list.".to_string();

            Task::none()
        }
        ListsMessage::ConfirmedAddWallpapers(list_name, paths) => {
            state.is_editing = false;

            state.status_message =
                format!("Adding {} wallpapers to '{}'...", paths.len(), list_name);

            state.error_message = None;

            let name_clone = list_name.clone();

            Task::perform(
                async move { crate::core::async_bridge::add_to_list_async(&name_clone, paths).await },
                move |result| ListsMessage::AddedToList(list_name.clone(), result),
            )
        }
        ListsMessage::AddedToList(list_name, result) => match result {
            Ok(()) => {
                state.status_message = format!("Wallpapers added to '{}'.", list_name);

                state.error_message = None;

                Task::done(ListsMessage::SelectList(list_name))
            }
            Err(err) => {
                state.status_message = "Failed to add wallpapers.".to_string();

                state.error_message = Some(err.to_string());

                Task::none()
            }
        },
        ListsMessage::PromptRemoveWallpaper(list_name, item_id, rel_path) => {
            state.pending_remove_item = Some((list_name.clone(), item_id, rel_path));

            state.status_message = format!("Confirm remove item from '{}'", list_name);

            state.error_message = None;

            Task::none()
        }
        ListsMessage::CancelRemoveWallpaper => {
            state.pending_remove_item = None;

            state.status_message = "Removal canceled.".to_string();

            Task::none()
        }
        ListsMessage::ConfirmRemoveWallpaper => {
            let Some((list_name, item_id, _)) = state.pending_remove_item.clone() else {
                return Task::none();
            };

            state.pending_remove_item = None;

            state.is_editing = true;

            state.status_message = format!("Removing 1 item from '{}'...", list_name);

            Task::done(ListsMessage::ConfirmedRemoveWallpapers(
                list_name,
                vec![item_id],
            ))
        }
        ListsMessage::RemoveWallpapersPressed(list_name, ids) => {
            state.is_editing = true;

            state.pending_remove_ids = ids.clone();

            state.status_message =
                format!("Removing {} item(s) from '{}'...", ids.len(), list_name);

            Task::done(ListsMessage::ConfirmedRemoveWallpapers(list_name, ids))
        }
        ListsMessage::ConfirmedRemoveWallpapers(list_name, ids) => {
            let parsed_ids = ids
                .into_iter()
                .filter_map(|id| Uuid::parse_str(&id).ok())
                .collect::<Vec<Uuid>>();

            if parsed_ids.is_empty() {
                state.is_editing = false;

                state.error_message = Some("No valid item IDs selected for removal.".to_string());

                state.status_message = "Failed to remove wallpapers.".to_string();

                return Task::none();
            }

            let list_name_for_task = list_name.clone();

            Task::perform(
                async move {
                    crate::core::async_bridge::remove_from_list_async(
                        &list_name_for_task,
                        None,
                        Some(parsed_ids),
                    )
                    .await
                },
                move |result| ListsMessage::RemovedFromList(list_name.clone(), result),
            )
        }
        ListsMessage::PlayListPressed(list_name, _interval_secs) => {
            state.status_message = format!("Preparing slideshow for '{}'.", list_name);

            state.error_message = None;

            Task::none()
        }
        ListsMessage::RemovedFromList(list_name, result) => {
            state.is_editing = false;

            state.pending_remove_item = None;

            match result {
                Ok(()) => {
                    state.status_message = format!("Wallpapers removed from '{}'.", list_name);

                    state.error_message = None;

                    Task::done(ListsMessage::SelectList(list_name))
                }
                Err(err) => {
                    state.status_message = "Failed to remove wallpapers.".to_string();

                    state.error_message = Some(err.to_string());

                    Task::none()
                }
            }
        }
    }
}
