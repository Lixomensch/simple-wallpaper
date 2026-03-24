use std::path::PathBuf;

use crate::core::query::{self, SetInputResolution};
use crate::core::wallpaper::{list_images, wallpaper_dir};

pub fn pick_from_entries(entries: Vec<(String, PathBuf)>, prompt: &str) -> Result<PathBuf, String> {
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
            | inquire::InquireError::OperationInterrupted => "Canceled.".to_string(),
            other => other.to_string(),
        })?;

    entries
        .into_iter()
        .find(|(name, _)| name == &selected)
        .map(|(_, path)| path)
        .ok_or_else(|| "Invalid selection.".to_string())
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

pub fn interactive_pick() -> Result<PathBuf, String> {
    let dir = wallpaper_dir()?;
    let images = list_images(&dir);

    if images.is_empty() {
        return Err(format!(
            "No wallpaper found in {}.\n\
             Add images and try again (`swp list` to check).",
            dir.display()
        ));
    }

    pick_from_entries(to_sorted_entries(images), "Choose a wallpaper:")
}

pub fn pick_from_matches(matches: Vec<PathBuf>, query: &str) -> Result<PathBuf, String> {
    let entries = to_sorted_entries(matches);
    let count = entries.len();
    let prompt = format!(
        "{} result(s) for \"{}\" — choose:",
        count, query
    );
    pick_from_entries(entries, &prompt)
}

pub fn resolve_set_input(input: &str) -> Result<PathBuf, String> {
    match query::resolve_set_input(input)? {
        SetInputResolution::Resolved(path) => Ok(path),
        SetInputResolution::MultipleMatches { query, matches } => {
            pick_from_matches(matches, &query)
        }
    }
}
