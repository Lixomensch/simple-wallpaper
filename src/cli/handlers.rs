use colored::Colorize;
use uuid::Uuid;

use crate::cli::ListsCommands;
use crate::cli::cmd::{interactive_pick, resolve_set_input};
use crate::core::lists;
use crate::core::lists_play;
use crate::core::operations;
use crate::core::wallpaper;
use crate::error::{ListError, SwpError};

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
        Some(ListsCommands::Play { name, interval }) => handle_lists_play(name, interval),
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

pub fn handle_lists_play(name: Option<String>, interval: String) -> Result<(), SwpError> {
    let source = if let Some(list_name) = name {
        let player = lists::ListPlayer::new(&list_name)?;
        println!(
            "{} '{}' — interval {}  {}",
            "List play started".cyan().bold(),
            list_name.yellow().bold(),
            interval.yellow().bold(),
            "(Ctrl-C to stop)".dimmed()
        );
        lists_play::PlaybackSource::List(player)
    } else {
        println!(
            "{} — interval {}  {}",
            "Play started".cyan().bold(),
            interval.yellow().bold(),
            "(Ctrl-C to stop)".dimmed()
        );
        lists_play::PlaybackSource::Filesystem
    };

    lists_play::run(&interval, source)
}
