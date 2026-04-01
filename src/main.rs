use clap::Parser;
use colored::Colorize;
use simple_wallpaper::cli::{Cli, Commands};
use simple_wallpaper::error::SwpError;
use simple_wallpaper::{cli, core, gui};

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), SwpError> {
    core::wallpaper::init()?;

    let app = Cli::parse();

    match app.command {
        Commands::Set { name } => cli::handlers::handle_set(name),
        Commands::Random => cli::handlers::handle_random(),
        Commands::Wallpapers { plain } => cli::handlers::handle_wallpapers(plain),
        Commands::Lists { command } => cli::handlers::handle_lists(command),
        Commands::Path => cli::handlers::handle_path(),
        Commands::Gui => gui::launch(),
    }
}
