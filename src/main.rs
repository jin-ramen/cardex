use std::io;
use std::{env, process};

use cardex::tcgdex::TcgDexClient;
use cardex::render;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args : Vec<String> = env::args().collect();
    let config: Config = Config::build(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing argument: {err}");
        process::exit(1);
    });

    let client: TcgDexClient = TcgDexClient::new(); 
    run(config, &client).await
}

pub struct Config {
    pub command: String,
    pub value: String,
}

impl Config {
    pub fn build(args: &[String]) -> Result<Config, &'static str> {
        if args.len() < 3 {
            return Err("not enough arguments");
        }

        let command: String = args[1].clone();
        let value: String = args[2].clone();

        Ok(Config { command, value })
    }
}

async fn run(config: Config, client: &TcgDexClient) -> anyhow::Result<()> {
    match (config.command.as_str(), config.value.as_str()) {
        ("card", id) => {
            let card = client.get_card(id).await?;

            let img = match card.image_url() {
                Some(url) => {
                    let bytes = client.fetch_image(&url).await?;
                    Some(render::prepare(&bytes)?)
                }
                None => None,
            };

            let stdout = std::io::stdout();
            render::render(&mut stdout.lock(), &card, img.as_ref())?;
        }
        ("search", name) => {
            let cards = client.list_cards(name).await?;
            render::render_list(&mut io::stdout().lock(), name, &cards)?;
        }
        (cmd, _) => anyhow::bail!("unknown command '{cmd}'\nusage: cardex card <id> | search <name>")
    }
    Ok(())
}