use clap::Parser;

use cardex::tcgdex::TcgDexClient;
use cardex::cli::config::{Config, Command};
use cardex::cli::commands;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let Config { command, region } = Config::parse();
    let client: TcgDexClient = TcgDexClient::new(&region); 

    match command {
        Command::Card { id } => {
            commands::card(&client, &id).await?
        }
        Command::Search { name, rarity } => {
            commands::search(&client, &name, rarity.as_deref()).await?
        }
        Command::Set { id } => {
            commands::set(&client, &id).await?
        }
        Command::Sets { name, } => {
            commands::sets(&client, name.as_deref()).await?
        }
    }
    Ok(())
}