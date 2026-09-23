use serde::Deserialize;

use crate::tcgdex::card::CardBrief;
use crate::tcgdex::serie::SerieBrief;


#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBrief {
    pub id: String,
    pub name: String,
    pub logo: Option<String>,
    pub symbol: Option<String>,
    pub card_count: CardCount,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Set {
    pub id: String,
    pub name: String,
    pub logo: Option<String>,
    pub symbol: Option<String>,
    pub card_count: CardCount,
    pub serie: SerieBrief,
    pub tcg_online: Option<String>,
    pub release_date: String,
    pub legal: Legal,
    pub boosters: Option<Vec<Booster>>,
    pub cards: Vec<CardBrief>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardCount {
    pub total: u32,
    pub official: u32,
    pub reverse: Option<u32>,
    pub holo: Option<u32>,
    pub first_ed: Option<u32>,
}

#[derive(Deserialize, Debug)]
pub struct Legal {
    pub expanded: bool,
    pub standard: bool,
}


#[derive(Deserialize, Debug)]
pub struct Booster {
    pub id: String,
    pub name: String,
    pub logo: Option<String>,
    pub artwork_front: Option<String>,
    pub artwork_back: Option<String>,
}