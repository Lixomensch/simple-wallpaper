//! Lists handler: manages wallpaper collection CRUD operations.
//!
//! Handles creating, deleting, editing, and playing wallpaper lists.

use iced::Task;

use crate::gui::message::ListsMessage;
use crate::gui::state::ListsState;

/// Update handler for lists messages.
pub fn update(state: &mut ListsState, message: ListsMessage) -> Task<ListsMessage> {
    match message {
        ListsMessage::LoadLists => {
            state.status_message = "Loading lists...".to_string();
            state.error_message = None;

            Task::perform(
                async move {
                    crate::core::async_bridge::list_lists_async().await
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
                async move {
                    crate::core::async_bridge::create_list_async(&name_clone).await
                },
                |result| ListsMessage::CreatedList(result),
            )
        }
        ListsMessage::CreatedList(result) => {
            match result {
                Ok(()) => {
                    state.status_message = "List created successfully.".to_string();
                    state.error_message = None;
                    // Reload lists
                    Task::done(ListsMessage::LoadLists)
                }
                Err(err) => {
                    state.status_message = "Failed to create list.".to_string();
                    state.error_message = Some(err.to_string());
                    Task::none()
                }
            }
        }
        ListsMessage::SelectList(_list_name) => {
            // For now, just update status
            state.status_message = "List selected.".to_string();
            state.error_message = None;
            state.selected_list_items.clear();
            Task::none()
        }
        ListsMessage::DeleteListPressed(list_name) => {
            state.status_message = format!("Are you sure? Deleting '{}'...", list_name);
            let name_clone = list_name.clone();
            Task::perform(
                async move {
                    crate::core::async_bridge::delete_list_async(&name_clone).await
                },
                move |result| ListsMessage::DeletedList(list_name.clone(), result),
            )
        }
        ListsMessage::DeletedList(name, result) => {
            match result {
                Ok(()) => {
                    state.selected_list = None;
                    state.status_message = format!("List '{}' deleted.", name);
                    state.error_message = None;
                    // Reload lists
                    Task::done(ListsMessage::LoadLists)
                }
                Err(err) => {
                    state.status_message = "Failed to delete list.".to_string();
                    state.error_message = Some(err.to_string());
                    Task::none()
                }
            }
        }
        ListsMessage::AddWallpapersPressed(_list_name) => {
            state.is_editing = true;
            state.pending_add_paths.clear();
            state.status_message = "Select wallpapers to add to this list.".to_string();
            Task::none()
        }
        ListsMessage::ConfirmedAddWallpapers(list_name, paths) => {
            state.is_editing = false;
            state.status_message = format!("Adding {} wallpapers to '{}'...", paths.len(), list_name);
            state.error_message = None;

            let name_clone = list_name.clone();
            Task::perform(
                async move {
                    crate::core::async_bridge::add_to_list_async(&name_clone, paths).await
                },
                move |result| ListsMessage::AddedToList(list_name.clone(), result),
            )
        }
        ListsMessage::AddedToList(list_name, result) => {
            match result {
                Ok(()) => {
                    state.status_message = format!("Wallpapers added to '{}'.", list_name);
                    state.error_message = None;
                    // Reload the list items
                    Task::done(ListsMessage::SelectList(list_name))
                }
                Err(err) => {
                    state.status_message = "Failed to add wallpapers.".to_string();
                    state.error_message = Some(err.to_string());
                    Task::none()
                }
            }
        }
        ListsMessage::RemoveWallpapersPressed(_list_name, _ids) => {
            state.is_editing = true;
            state.pending_remove_ids.clear();
            state.status_message = "Select wallpapers to remove.".to_string();
            Task::none()
        }
        ListsMessage::RemovedFromList(list_name, result) => {
            state.is_editing = false;
            match result {
                Ok(()) => {
                    state.status_message = format!("Wallpapers removed from '{}'.", list_name);
                    state.error_message = None;
                    // Reload the list items
                    Task::done(ListsMessage::SelectList(list_name))
                }
                Err(err) => {
                    state.status_message = "Failed to remove wallpapers.".to_string();
                    state.error_message = Some(err.to_string());
                    Task::none()
                }
            }
        }
        ListsMessage::PlayListPressed(list_name, _interval_secs) => {
            state.status_message = format!("Slideshow for '{}' not yet implemented.", list_name);
            state.error_message = None;
            Task::none()
        }
        _ => Task::none(),
    }
}
