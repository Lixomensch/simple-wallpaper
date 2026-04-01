use std::thread;
use colored::Colorize;
use uuid::Uuid;

use crate::cli::ListsCommands;
use crate::core::wallpaper;
use crate::core::operations;
use crate::core::lists;
use crate::core::slideshow;
use crate::cli::cmd::{interactive_pick, resolve_set_input};
use crate::error::SwpError;

pub fn handle_set(name: Vec<String>) -> Result<(), SwpError> {
    let query = name.join(" ");

    let path = if query.trim().is_empty() {
        interactive_pick()?
    } else {
        resolve_set_input(query.trim())?
    };

    let applied = operations::apply_wallpaper(&path)?;
    
    println!(
        "{} {}",
        "Wallpaper applied:".green().bold(),
        applied
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
    );

    Ok(())
}

pub fn handle_random() -> Result<(), SwpError> {
    let img = operations::random_wallpaper()?;
   
    println!(
        "{} {}",
        "Random wallpaper:".green().bold(),
        img
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
    );

    Ok(())
}

pub fn handle_play(interval: String) -> Result<(), SwpError> {
    let duration = slideshow::parse_interval(&interval)?;
    
    println!(
        "{} — interval {}  {}",
        "Play started".cyan().bold(),
        interval.yellow().bold(),
        "(Ctrl-C to stop)".dimmed()
    );

    loop {
        match operations::random_wallpaper() {
            Ok(applied) => println!(
                "  {}",
                applied
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
            ),
            Err(e) => eprintln!("  {} {}", "Error:".yellow().bold(), e),
        }
        thread::sleep(duration);
    }
}

pub fn handle_list(plain: bool) -> Result<(), SwpError> {
    let (dir, images) = operations::list_wallpapers()?;

    if plain {
        for img in &images {
            if let Some(name) = img.file_name().and_then(|n| n.to_str()) {
                println!("{name}");
            }
        }
    } else if images.is_empty() {
        println!(
            "{} No wallpaper found. Add images in",
            "!".yellow().bold()
        );
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
        Some(ListsCommands::Play { name, interval }) => {
            handle_lists_play(name, interval)
        }
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
    let items = lists::get_list_items(&name)?;

    if plain {
        for item in items {
            match item.relative_path {
                Some(relative) => println!("{}\t{}", item.id, relative.display()),
                None => println!("{}\t<missing-index-entry>", item.id),
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
                Some(relative) => println!("  {}  {}", item.id.to_string().dimmed(), relative.display()),
                None => println!("  {}  {}", item.id.to_string().dimmed(), "<missing-index-entry>".yellow()),
            }
        }
    }
    Ok(())
}

pub fn handle_lists_add(name: String, wallpapers: Vec<String>) -> Result<(), SwpError> {
    let mut paths = Vec::new();
    for input in wallpapers {
        paths.push(resolve_set_input(&input)?);
    }

    let added = lists::add_wallpapers_by_paths(&name, &paths)?;
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

    for input in wallpapers {
        if let Ok(id) = Uuid::parse_str(&input) {
            ids.push(id);
            continue;
        }
        paths.push(resolve_set_input(&input)?);
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

pub fn handle_lists_play(name: String, interval: String) -> Result<(), SwpError> {
    let duration = slideshow::parse_interval(&interval)?;
    let mut player = lists::ListPlayer::new(&name)?;

    println!(
        "{} '{}' — interval {}  {}",
        "List play started".cyan().bold(),
        name.yellow().bold(),
        interval.yellow().bold(),
        "(Ctrl-C to stop)".dimmed()
    );

    loop {
        match player.next_wallpaper() {
            Ok(path) => match operations::apply_wallpaper(&path) {
                Ok(applied) => println!(
                    "  {}",
                    applied
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                ),
                Err(e) => eprintln!("  {} {}", "Error:".yellow().bold(), e),
            },
            Err(e) => eprintln!("  {} {}", "Error:".yellow().bold(), e),
        }
        thread::sleep(duration);
    }
}