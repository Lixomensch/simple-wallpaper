use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "swp", author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Set {
        #[arg(num_args = 0.., value_name = "NOME")]
        name: Vec<String>,
    },
    Random,
    Slideshow {
        #[arg(short, long, default_value = "15m")]
        interval: String,
    },
    List {
        #[arg(long)]
        plain: bool,
    },
    Path,
}
