use std::thread;
use colored::Colorize;

use crate::core::operations;
use crate::core::slideshow;
use crate::cli::cmd::{interactive_pick, resolve_set_input};

pub fn handle_set(name: Vec<String>) -> Result<(), String> {
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

pub fn handle_random() -> Result<(), String> {
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

pub fn handle_slideshow(interval: String) -> Result<(), String> {
    let duration = slideshow::parse_interval(&interval)?;
    
    println!(
        "{} — interval {}  {}",
        "Slideshow started".cyan().bold(),
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

pub fn handle_list(plain: bool) -> Result<(), String> {
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

pub fn handle_path() -> Result<(), String> {
    let dir = operations::wallpaper_directory()?;
    println!("{}", dir.display());
    Ok(())
}