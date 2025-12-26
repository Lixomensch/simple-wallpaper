use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "swp", author, version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Set {
        name: String,
    },

    Random,

    Slideshow {
        #[arg(short, long, default_value = "15m")]
        interval: String,
    },

    Path,
}
