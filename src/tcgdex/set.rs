use serde::Deserialize;


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
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardCount {
    pub total: u32,
    pub official: u32,
}