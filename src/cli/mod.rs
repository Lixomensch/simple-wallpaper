pub use crate as cli;
pub use simple_wallpaper_core as core;
pub use simple_wallpaper_core::error;

pub mod cmd;
pub mod handlers;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "swp",
    author,
    version,
    about,
    long_about = None,
    disable_help_subcommand = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Add {
        #[arg(required = true, num_args = 1.., value_name = "FILE")]
        files: Vec<String>,
    },
    Rmv {
        #[arg(value_name = "NAME")]
        name: Option<String>,
        #[arg(short, long)]
        force: bool,
    },
    Set {
        #[arg(num_args = 0.., value_name = "NAME")]
        name: Vec<String>,
    },
    Random,
    Wallpapers {
        #[arg(long)]
        plain: bool,
    },
    Lists {
        #[command(subcommand)]
        command: Option<ListsCommands>,
    },
    Path,
}

#[derive(Subcommand)]
pub enum ListsCommands {
    Create {
        name: String,
    },
    Delete {
        name: String,
    },
    Show {
        name: String,
        #[arg(short = 'p', long)]
        plain: bool,
    },
    Add {
        name: String,
        #[arg(num_args = 0.., value_name = "WALLPAPER")]
        wallpapers: Vec<String>,
    },
    Remove {
        name: String,
        #[arg(num_args = 0.., value_name = "WALLPAPER_OR_UUID")]
        wallpapers: Vec<String>,
    },
    Play {
        name: Option<String>,
        #[arg(short = 'i', long, default_value = "15m")]
        interval: String,
        #[arg(long, hide = true, default_value_t = false)]
        foreground: bool,
        #[arg(long, hide = true, default_value_t = false)]
        resume: bool,
    },
    Stop,
}
