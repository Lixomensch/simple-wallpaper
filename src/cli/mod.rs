pub mod handlers;
pub mod cmd;

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
    Set {
        #[arg(num_args = 0.., value_name = "NAME")]
        name: Vec<String>,
    },
    Random,
    Play {
        #[arg(default_value = "15m")]
        interval: String,
    },
    List {
        #[arg(long)]
        plain: bool,
    },
    Lists {
        #[command(subcommand)]
        command: Option<ListsCommands>,
    },
    Path,
    Gui,
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
        #[arg(long)]
        plain: bool,
    },
    Add {
        name: String,
        #[arg(required = true, num_args = 1.., value_name = "WALLPAPER")]
        wallpapers: Vec<String>,
    },
    Remove {
        name: String,
        #[arg(required = true, num_args = 1.., value_name = "WALLPAPER_OR_UUID")]
        wallpapers: Vec<String>,
    },
    Play {
        name: String,
        #[arg(long, default_value = "15m")]
        interval: String,
    },
}