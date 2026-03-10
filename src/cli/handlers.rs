use std::thread;
use std::path::{Path, PathBuf};
use std::time::Duration;
use colored::Colorize;

use crate::backends;
use crate::cli::cmd::{interactive_pick, resolve_set_input};
use crate::wallpaper;

pub fn set(path: &Path) -> Result<PathBuf, String> {

    let img = path.to_path_buf();

    backends::apply(&img)?;
    Ok(img)
}

pub fn random() -> Result<PathBuf, String> {
    let dir = wallpaper::wallpaper_dir()?;
    let img = wallpaper::random_image(&dir).ok_or(
        "No image found. Add images to `swp path` first.",
    )?;
    backends::apply(&img)?;
    Ok(img)
}

pub fn handle_set(name: Vec<String>) -> Result<(), String> {
    let query = name.join(" ");

    let path = if query.trim().is_empty() {
        interactive_pick()?
    } else {
        resolve_set_input(query.trim())?
    };

    let applied = set(&path)?;
    
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

    let dir = wallpaper::wallpaper_dir()?;
    let img = wallpaper::random_image(&dir).ok_or(
        "No image found. Add images to `swp path` first.",
    )?;
    backends::apply(&img)?;
   
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

pub fn parse_interval(interval: &str) -> Result<Duration, String> {
    let last = interval.chars().last().unwrap_or('\0');

    if !"smh".contains(last) {
        return Err(format!(
            "Invalid interval format: '{}'. \
             Use a number followed by s, m, or h (e.g., 30s, 10m, 2h).",
            interval
        ));
    }

    let num_part = &interval[..interval.len() - 1];
    let n = num_part.parse::<u64>().map_err(|_| {
        format!(
            "Invalid number in '{}'. Use a positive integer (e.g., 30s, 10m, 2h).",
            interval
        )
    })?;

    if n == 0 {
        return Err("The interval must be greater than zero.".into());
    }

    match last {
        's' => Ok(Duration::from_secs(n)),
        'm' => Ok(Duration::from_secs(n * 60)),
        'h' => Ok(Duration::from_secs(n * 3600)),
        _ => unreachable!(),
    }
}

pub fn handle_slideshow(interval: String) -> Result<(), String> {
    let duration = parse_interval(&interval)?;
    
    println!(
        "{} — interval {}  {}",
        "Slideshow started".cyan().bold(),
        interval.yellow().bold(),
        "(Ctrl-C to stop)".dimmed()
    );

    loop {
        match random() {
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
    let dir = wallpaper::wallpaper_dir()?;
    let mut images = wallpaper::list_images(&dir);
    images.sort();

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
    let dir = wallpaper::wallpaper_dir()?;
    println!("{}", dir.display());
    Ok(())
}