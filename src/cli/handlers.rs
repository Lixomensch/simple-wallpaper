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
        "Nenhuma imagem encontrada. Adicione imagens em `swp path` primeiro.",
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
        "✔ Wallpaper aplicado:".green().bold(),
        applied.display()
    );

    Ok(())
}

pub fn handle_random() -> Result<(), String> {

    let dir = wallpaper::wallpaper_dir()?;
    let img = wallpaper::random_image(&dir).ok_or(
        "Nenhuma imagem encontrada. Adicione imagens em `swp path` primeiro.",
    )?;
    backends::apply(&img)?;
   
    println!(
        "{} {}",
        "✔ Wallpaper aleatório:".green().bold(),
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
            "Formato de intervalo inválido: '{}'. \
             Use um número seguido de s, m ou h (ex: 30s, 10m, 2h).",
            interval
        ));
    }

    let num_part = &interval[..interval.len() - 1];
    let n = num_part.parse::<u64>().map_err(|_| {
        format!(
            "Número inválido em '{}'. Use um inteiro positivo (ex: 30s, 10m, 2h).",
            interval
        )
    })?;

    if n == 0 {
        return Err("O intervalo deve ser maior que zero.".into());
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
        "{} — intervalo {}  {}",
        "Slideshow iniciado".cyan().bold(),
        interval.yellow().bold(),
        "(Ctrl-C para parar)".dimmed()
    );

    loop {
        match random() {
            Ok(applied) => println!(
                "  {} {}",
                "↺".cyan(),
                applied
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
            ),
            Err(e) => eprintln!("  {} {}", "⚠ Erro:".yellow().bold(), e),
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
            "{} Nenhum wallpaper encontrado. Adicione imagens em:",
            "!".yellow().bold()
        );
        println!("  {}", dir.display().to_string().cyan());
    } else {
        println!(
            "{} {} imagem(ns) em {}:",
            "✔".green().bold(),
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