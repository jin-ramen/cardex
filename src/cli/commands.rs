use crate::tcgdex::{TcgDexClient, Query};
use crate::render;

pub async fn card(client: &TcgDexClient, id: &str) -> anyhow::Result<()> {
    let card = client.get_card(id).await?;
    render::render_card(client, &card).await?;
    Ok(())
}

pub async fn search(
    client: &TcgDexClient,
    name: &str,
    rarity: Option<&str>,
) -> anyhow::Result<()> {
    let mut queries: Vec<Query> = vec![Query::new("name", name)];
    if let Some(r) = rarity {
        queries.push(Query::eq("rarity", r));
    }

    let cards: Vec<crate::tcgdex::CardBrief> = client.list_cards(&queries).await?;
    render::render_list(&cards)?;
    Ok(())
}

pub async fn set(client: &TcgDexClient, id: &str) -> anyhow::Result<()> {
    let set = client.get_set(id).await?;
    render::render_set(client, &set).await?;
    Ok(())
}

pub async fn sets(
    client: &TcgDexClient,
    name: Option<&str>
) -> anyhow::Result<()> {
    let mut queries: Vec<Query> = vec![];
    if let Some(n) = name {
        queries.push(Query::new("name", n));
    }

    let sets = client.list_sets(&queries).await?;
    render::render_sets(&sets)?;
    Ok(())
}