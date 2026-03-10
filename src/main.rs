use std::thread;

use clap::Parser;
use colored::Colorize;

mod backends;
mod cli;
mod utils;
mod wallpaper;

use cli::{Cli, Commands};

use crate::{utils::parse_interval, wallpaper::{interactive_pick, resolve_set_input}};

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Erro:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Set { name } => {
            let query = name.join(" ");
            let path = if query.trim().is_empty() {
                interactive_pick()?
            } else {
                resolve_set_input(query.trim())?
            };

            let applied = wallpaper::set(&path)?;
            println!(
                "{} {}",
                "✔ Wallpaper aplicado:".green().bold(),
                applied.display()
            );
        }

        Commands::Random => {
            let applied = wallpaper::random()?;
            println!(
                "{} {}",
                "✔ Wallpaper aleatório:".green().bold(),
                applied
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("?")
            );
        }

        Commands::Slideshow { interval } => {
            let duration = parse_interval(&interval)?;
            println!(
                "{} — intervalo {}  {}",
                "Slideshow iniciado".cyan().bold(),
                interval.yellow().bold(),
                "(Ctrl-C para parar)".dimmed()
            );

            loop {
                match wallpaper::random() {
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

        Commands::List { plain } => {
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
        }

        Commands::Path => {
            let dir = wallpaper::wallpaper_dir()?;
            println!("{}", dir.display());
        }
    }

    Ok(())
}
