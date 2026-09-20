use pullrate::tcgdex::TcgDexClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = TcgDexClient::new(); 
    let set = client.get_set("ff").await?;

    println!("{:#?}", set);

    Ok(())
}