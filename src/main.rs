use std::{path::PathBuf, thread, time::Duration};

use clap::Parser;

mod kde;
mod utils;
mod cli;
mod wallpaper;

use cli::{Cli, Commands};

fn parse_interval(interval: &str) -> Result<Duration, String> {
    if interval.ends_with('s') {
        let n = interval.trim_end_matches('s').parse::<u64>()
            .map_err(|_| "Intervalo inválido".to_string())?;
        Ok(Duration::from_secs(n))
    } else if interval.ends_with('m') {
        let n = interval.trim_end_matches('m').parse::<u64>()
            .map_err(|_| "Intervalo inválido".to_string())?;
        Ok(Duration::from_secs(n * 60))
    } else if interval.ends_with('h') {
        let n = interval.trim_end_matches('h').parse::<u64>()
            .map_err(|_| "Intervalo inválido".to_string())?;
        Ok(Duration::from_secs(n * 3600))
    } else {
        Err("Formato de intervalo inválido. Use 's', 'm' ou 'h'.".to_string())
    }
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Set { name } => {
            let path = PathBuf::from(name);
            wallpaper::set(&path)?;
            println!("✔ Wallpaper aplicado: {}", path.display());
        }
        Commands::Random => {
            wallpaper::random()?;
        }
        Commands::Slideshow { interval } => {
            let duration = parse_interval(&interval)?;
            println!("Slideshow iniciado: intervalo {}", interval);

            loop {
                if let Err(e) = wallpaper::random() {
                    eprintln!("Erro ao trocar wallpaper: {}", e);
                }
                thread::sleep(duration);
            }
        }
        Commands::Path => {
            let dir = wallpaper::wallpaper_dir()?;
            println!("{}", dir.display());
        }
    }

    Ok(())
}
