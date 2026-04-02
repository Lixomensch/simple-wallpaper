use colored::Colorize;
use std::path::PathBuf;
use uuid::Uuid;

use crate::cli::ListsCommands;
use crate::cli::cmd::{confirm_removal, interactive_pick, resolve_set_input};
use crate::core::lists;
use crate::core::lists_play;
use crate::core::operations;
use crate::core::playback_manager;
use crate::core::wallpaper;
use crate::error::{ListError, SelectionError, SwpError};

fn print_applied(prefix: &str, path: &std::path::Path) {
    println!(
        "{} {}",
        prefix.green().bold(),
        path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
    );
}

pub fn handle_set(name: Vec<String>) -> Result<(), SwpError> {
    let query = name.join(" ");

    let path = if query.trim().is_empty() {
        interactive_pick()?
    } else {
        resolve_set_input(query.trim())?
    };

    let applied = operations::apply_wallpaper(&path)?;
    print_applied("Wallpaper applied:", &applied);

    Ok(())
}

pub fn handle_add(files: Vec<String>) -> Result<(), SwpError> {
    let mut added = 0usize;
    let mut failed: Vec<(String, String)> = Vec::new();

    for input in files {
        let source = PathBuf::from(&input);
        match wallpaper::import_image(&source) {
            Ok(destination) => {
                added += 1;
                println!(
                    "{} {}",
                    "Imported:".green().bold(),
                    destination
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                );
            }
            Err(err) => failed.push((input, err.to_string())),
        }
    }

    println!(
        "{} {} {}",
        "Imported".green().bold(),
        added.to_string().yellow().bold(),
        "wallpaper(s).".green().bold()
    );

    if !failed.is_empty() {
        println!(
            "{} {} {}",
            "Failed:".yellow().bold(),
            failed.len().to_string().yellow().bold(),
            "input(s).".yellow().bold()
        );
        for (input, reason) in failed {
            println!("  {} {}", input.cyan(), reason.dimmed());
        }
    }

    Ok(())
}

pub fn handle_rmv(name: Option<String>, force: bool) -> Result<(), SwpError> {
    let path = match name {
        Some(name) if name.trim().is_empty() => interactive_pick(),
        Some(name) => resolve_set_input(name.trim()),
        None => interactive_pick(),
    };

    let path = match path {
        Ok(path) => path,
        Err(SwpError::Selection(SelectionError::Canceled)) => {
            println!("{}", "Canceled.".yellow().bold());
            return Ok(());
        }
        Err(err) => return Err(err),
    };

    if !force {
        let display_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let confirmed = match confirm_removal(display_name) {
            Ok(value) => value,
            Err(SwpError::Selection(SelectionError::Canceled)) => {
                println!("{}", "Canceled.".yellow().bold());
                return Ok(());
            }
            Err(err) => return Err(err),
        };

        if !confirmed {
            println!("{}", "Canceled.".yellow().bold());
            return Ok(());
        }
    }

    wallpaper::remove_image(&path)?;
    println!(
        "{} {}",
        "Removed:".green().bold(),
        path.file_name().and_then(|n| n.to_str()).unwrap_or("?")
    );

    Ok(())
}

pub fn handle_random() -> Result<(), SwpError> {
    let img = operations::random_wallpaper()?;
    print_applied("Random wallpaper:", &img);

    Ok(())
}

pub fn handle_wallpapers(plain: bool) -> Result<(), SwpError> {
    let (dir, images) = operations::list_wallpapers()?;

    if plain {
        for img in &images {
            if let Some(name) = img.file_name().and_then(|n| n.to_str()) {
                println!("{name}");
            }
        }
    } else if images.is_empty() {
        println!("{} No wallpaper found. Add images in", "!".yellow().bold());
        println!("  {}", dir.display().to_string().cyan());
    } else {
        println!(
            "{} image(s) in {}:",
            images.len().to_string().yellow().bold(),
            dir.display().to_string().dimmed()
        );
        for img in &images {
            if let Some(name) = img.file_name().and_then(|n| n.to_str()) {
                println!("  {}", name.cyan());
            }
        }
    }

    Ok(())
}

pub fn handle_path() -> Result<(), SwpError> {
    let dir = wallpaper::wallpaper_dir()?;
    println!("{}", dir.display());
    Ok(())
}

pub fn handle_lists(command: Option<ListsCommands>) -> Result<(), SwpError> {
    match command {
        None => handle_lists_list(),
        Some(ListsCommands::Create { name }) => handle_lists_create(name),
        Some(ListsCommands::Delete { name }) => handle_lists_delete(name),
        Some(ListsCommands::Show { name, plain }) => handle_lists_show(name, plain),
        Some(ListsCommands::Add { name, wallpapers }) => handle_lists_add(name, wallpapers),
        Some(ListsCommands::Remove { name, wallpapers }) => handle_lists_remove(name, wallpapers),
        Some(ListsCommands::Play {
            name,
            interval,
            foreground,
            resume,
        }) => handle_lists_play(name, interval, foreground, resume),
        Some(ListsCommands::Stop) => handle_lists_stop(),
    }
}

pub fn handle_lists_list() -> Result<(), SwpError> {
    let summaries = lists::list_lists()?;

    if summaries.is_empty() {
        println!("{} No lists found.", "!".yellow().bold());
        return Ok(());
    }

    println!("{}", "Wallpaper lists:".cyan().bold());
    for summary in summaries {
        println!("  {} ({})", summary.name.green(), summary.count);
    }
    Ok(())
}

pub fn handle_lists_create(name: String) -> Result<(), SwpError> {
    lists::create_list(&name)?;
    println!("{} {}", "List created:".green().bold(), name);
    Ok(())
}

pub fn handle_lists_delete(name: String) -> Result<(), SwpError> {
    lists::delete_list(&name)?;
    println!("{} {}", "List deleted:".green().bold(), name);
    Ok(())
}

pub fn handle_lists_show(name: String, plain: bool) -> Result<(), SwpError> {
    let list = lists::get_list(&name)?;
    let items = lists::get_list_items_from_list(&list)?;

    if plain {
        for item in items {
            match item.relative_path {
                Some(relative) => println!("{}", relative.display()),
                None => println!("<missing-index-entry>"),
            }
        }
    } else {
        println!(
            "{} {} ({})",
            "List:".cyan().bold(),
            list.name.green().bold(),
            format!("{} wallpaper(s)", list.wallpapers.len()).dimmed()
        );
        for item in items {
            match item.relative_path {
                Some(relative) => println!("  {}", relative.display()),
                None => println!("  {}", "<missing-index-entry>".yellow()),
            }
        }
    }
    Ok(())
}

pub fn handle_lists_add(name: String, wallpapers: Vec<String>) -> Result<(), SwpError> {
    let mut paths = Vec::new();

    if wallpapers.is_empty() {
        paths.push(interactive_pick()?);
    } else {
        for input in wallpapers {
            paths.push(resolve_set_input(&input)?);
        }
    }

    let added = match lists::add_wallpapers_by_paths(&name, &paths) {
        Ok(added) => added,
        Err(SwpError::List(ListError::AlreadyInList { .. })) => {
            println!(
                "{}",
                "The wallpaper is already on the list.".yellow().bold()
            );
            return Ok(());
        }
        Err(err) => return Err(err),
    };
    println!(
        "{} {} {}",
        "Added".green().bold(),
        added.to_string().yellow().bold(),
        "wallpaper(s)".green().bold()
    );
    Ok(())
}

pub fn handle_lists_remove(name: String, wallpapers: Vec<String>) -> Result<(), SwpError> {
    let mut ids = Vec::new();
    let mut paths = Vec::new();

    if wallpapers.is_empty() {
        ids.push(crate::cli::cmd::pick_from_list(&name)?);
    } else {
        for input in wallpapers {
            if let Ok(id) = Uuid::parse_str(&input) {
                ids.push(id);
                continue;
            }
            paths.push(resolve_set_input(&input)?);
        }
    }

    let removed = lists::remove_wallpapers(&name, &paths, &ids)?;
    println!(
        "{} {} {}",
        "Removed".green().bold(),
        removed.to_string().yellow().bold(),
        "wallpaper(s)".green().bold()
    );
    Ok(())
}

pub fn handle_lists_play(
    name: Option<String>,
    interval: String,
    foreground: bool,
    resume: bool,
) -> Result<(), SwpError> {
    if resume {
        return playback_manager::resume_playback();
    }

    if foreground {
        let source = if let Some(list_name) = name {
            let player = lists::ListPlayer::new(&list_name)?;
            lists_play::PlaybackSource::List(player)
        } else {
            lists_play::PlaybackSource::Filesystem
        };

        return lists_play::run_foreground(&interval, source);
    }

    let pid = playback_manager::start_playback(name.clone(), &interval)?;
    println!(
        "{} pid={} interval={} {}",
        "Playback started".cyan().bold(),
        pid.to_string().yellow().bold(),
        interval.yellow().bold(),
        "(runs in background)".dimmed()
    );

    Ok(())
}

pub fn handle_lists_stop() -> Result<(), SwpError> {
    playback_manager::stop_playback()?;
    println!("{}", "Playback stopped.".green().bold());
    Ok(())
}
