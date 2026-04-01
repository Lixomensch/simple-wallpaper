use std::path::PathBuf;

use crate::core::lists;
use crate::core::query::{self, SetInputResolution};
use crate::core::wallpaper::{list_images, wallpaper_dir};
use crate::error::{QueryError, SelectionError, SwpError};
use uuid::Uuid;

pub fn pick_from_entries(
    entries: Vec<(String, PathBuf)>,
    prompt: &str,
) -> Result<PathBuf, SwpError> {
    let count = entries.len();
    let names: Vec<String> = entries.iter().map(|(n, _)| n.clone()).collect();

    let selected = inquire::Select::new(prompt, names)
        .with_page_size(14.min(count))
        .with_help_message(
            "↑↓ to navigate  •  Type to filter  •  Enter to confirm  •  Esc to cancel",
        )
        .prompt()
        .map_err(|e| match e {
            inquire::InquireError::OperationCanceled
            | inquire::InquireError::OperationInterrupted => SelectionError::Canceled,
            other => SelectionError::PromptFailure {
                message: other.to_string(),
            },
        })?;

    entries
        .into_iter()
        .find(|(name, _)| name == &selected)
        .map(|(_, path)| path)
        .ok_or(SelectionError::InvalidSelection.into())
}

pub fn to_sorted_entries(images: Vec<PathBuf>) -> Vec<(String, PathBuf)> {
    let mut entries: Vec<(String, PathBuf)> = images
        .into_iter()
        .filter_map(|p| {
            let name = p.file_name()?.to_str()?.to_owned();
            Some((name, p))
        })
        .collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

pub fn interactive_pick() -> Result<PathBuf, SwpError> {
    let dir = wallpaper_dir()?;
    let images = list_images(&dir);

    if images.is_empty() {
        return Err(QueryError::NoImagesInDir { dir }.into());
    }

    pick_from_entries(to_sorted_entries(images), "Choose a wallpaper:")
}

pub fn pick_from_matches(matches: Vec<PathBuf>, query: &str) -> Result<PathBuf, SwpError> {
    let entries = to_sorted_entries(matches);
    let count = entries.len();
    let prompt = format!("{} result(s) for \"{}\" — choose:", count, query);
    pick_from_entries(entries, &prompt)
}

pub fn resolve_set_input(input: &str) -> Result<PathBuf, SwpError> {
    match query::resolve_set_input(input)? {
        SetInputResolution::Resolved(path) => Ok(path),
        SetInputResolution::MultipleMatches { query, matches } => {
            pick_from_matches(matches, &query)
        }
    }
}

pub fn pick_from_list(list_name: &str) -> Result<Uuid, SwpError> {
    let items = lists::get_list_items(list_name)?;

    if items.is_empty() {
        return Err(SelectionError::PromptFailure {
            message: "No wallpapers in list.".to_string(),
        }
        .into());
    }

    let names: Vec<String> = items
        .iter()
        .filter_map(|item| {
            item.relative_path
                .as_ref()
                .and_then(|p| p.file_name()?.to_str())
                .map(|s| s.to_owned())
        })
        .collect();

    if names.is_empty() {
        return Err(SelectionError::PromptFailure {
            message: "No valid wallpapers in list.".to_string(),
        }
        .into());
    }

    let count = names.len();
    let selected = inquire::Select::new("Choose a wallpaper to remove:", names)
        .with_page_size(14.min(count))
        .with_help_message(
            "↑↓ to navigate  •  Type to filter  •  Enter to confirm  •  Esc to cancel",
        )
        .prompt()
        .map_err(|e| match e {
            inquire::InquireError::OperationCanceled
            | inquire::InquireError::OperationInterrupted => SelectionError::Canceled,
            other => SelectionError::PromptFailure {
                message: other.to_string(),
            },
        })?;

    items
        .into_iter()
        .find_map(|item| {
            item.relative_path
                .as_ref()
                .and_then(|p| p.file_name()?.to_str())
                .and_then(|name| {
                    if name == selected {
                        Some(item.id)
                    } else {
                        None
                    }
                })
        })
        .ok_or(SelectionError::InvalidSelection.into())
}
