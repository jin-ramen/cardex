use clap::{ Parser, Subcommand };

#[derive(Parser)]
#[command(name = "cardex", about = "Seach Pokemon cards via cli")]
pub struct Config {
    #[command(subcommand)]
    pub command: Command,

    #[arg(short = 'R', long, global = true, default_value = "en")]
    /// Card region to query (en, fr, de, ja, ...)
    pub region: String,
}

#[derive(Subcommand)]
#[command(author, about, long_about = None)]
pub enum Command {
    /// Show full details for a single card
    Card {
        id: String,
    },
    /// Search cards by name, with optional filters
    Search {
        name: String,
        #[arg(short, long)]
        rarity: Option<String>,
    },
    /// Show a set and list every card in it
    Set {
        id: String
    },
    /// List sets by name, all if no name is given
    Sets {
        #[arg(short, long)]
        name: Option<String>,
    },
}