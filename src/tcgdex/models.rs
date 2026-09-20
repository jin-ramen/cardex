use serde::Deserialize;

pub struct CardBrief {
    id: String,
    local_id: String,
    name: String,
    image: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    category: String,
    id: String,
    illustrator: String,
    local_id: String,
    name: String,
    rarity: String, // placeholder
    set: Set,
    variants: Variants,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Variants {
    first_edition: bool,
    holo: bool,
    normal: bool,
    reverse: bool,
    w_promo: bool,
}

pub struct SetBrief {
    id: String,
    image: String,
    local_id: u32,
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Set {
    id: String,
    name: String,
    logo: Option<String>,
    symbol: Option<String>,
    card_count: CardCount,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CardCount {
    total: u32,
    official: u32,
}