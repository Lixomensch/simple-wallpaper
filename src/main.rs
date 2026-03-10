use colored::Colorize;

mod backends;
mod wallpaper;
mod cli;

use cli::{Cli, Commands};
use clap::Parser;

fn main() {
    
    if let Err(e) = run() {
        eprintln!("{} {}", "Erro:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {

    wallpaper::init()?;
    
    let app = Cli::parse();

    match app.command {
        Commands::Set { name } => cli::handlers::handle_set(name),
        Commands::Random => cli::handlers::handle_random(),
        Commands::Slideshow { interval } => cli::handlers::handle_slideshow(interval),
        Commands::List { plain } => cli::handlers::handle_list(plain),
        Commands::Path => cli::handlers::handle_path(),
    }
}