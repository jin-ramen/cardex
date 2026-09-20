use std::collections::HashMap;
use std::fmt;

use serde::Deserialize;

use super::SetBrief;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CardBrief {
    pub id: String,
    pub local_id: String,
    pub name: String,
    pub image: Option<String>,
}

impl fmt::Display for CardBrief {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id: {}, name: {}", self.id, self.name)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Card {
    pub id: String,
    pub local_id: String,
    pub name: String,
    pub image: Option<String>,
    pub category: String,
    pub illustrator: Option<String>,
    pub rarity: Option<String>,
    pub set: SetBrief,
    #[serde(default)]
    pub variants: Variant,
    #[serde(default)]
    pub boosters: Vec<Booster>,
    pub pricing: Option<Pricing>,
    pub updated: Option<String>,
    pub regulation_mark: Option<String>,
    pub legal: Option<Legal>,
    #[serde(default)]
    pub dex_id: Vec<u32>,
    pub hp: Option<u16>,
    pub types: Option<Vec<String>>,
    pub evolve_from: Option<String>,
    pub description: Option<String>,
    pub level: Option<serde_json::Value>,
    pub stage: Option<String>,
    pub suffix: Option<String>,
    pub item: Option<HeldItem>,
    #[serde(default)]
    pub abilities: Vec<Ability>,
    #[serde(default)]
    pub attacks: Vec<Attack>,
    #[serde(default)]
    pub weaknesses: Vec<Weakness>,
    #[serde(default)]
    pub resistances: Vec<Resistance>,
    #[serde(default)]
    pub retreat: u8,
    pub effect: Option<String>,
    pub trainer_type: Option<String>,
    pub energy_type: Option<String>,
}

impl Card {
    pub fn image_url(&self) -> Option<String> {
        self.image.as_ref().map(|base| format!("{base}/low.png"))
    }

    pub fn is_pokemon(&self) -> bool {
        self.category.eq_ignore_ascii_case("pokemon")
    }

    pub fn is_trainer(&self) -> bool {
        self.category.eq_ignore_ascii_case("trainer")
    }

    pub fn is_energy(&self) -> bool {
        self.category.eq_ignore_ascii_case("energy")
    }

    pub fn number(&self) -> String {
        format!("{}/{}", self.local_id, self.set.card_count.official)
    }

    pub fn dex_label(&self) -> Option<String> {
        if self.dex_id.is_empty() {
            return None;
        }
        Some(
            self.dex_id
                .iter()
                .map(|d| format!("#{d:03}"))
                .collect::<Vec<_>>()
                .join("/"),
        )
    }

    pub fn level_str(&self) -> Option<String> {
        self.level
            .as_ref()
            .map(|v| v.to_string().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Variant {
    pub first_edition: bool,
    pub holo: bool,
    pub normal: bool,
    pub reverse: bool,
    pub w_promo: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Booster {
    pub id: String,
    pub name: String,
    pub logo: Option<String>,
    pub artwork_front: Option<String>,
    pub artwork_back: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Legal {
    pub standard: bool,
    pub expanded: bool,
}

#[derive(Deserialize, Debug)]
pub struct Ability {
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub effect: String,
}

#[derive(Deserialize, Debug)]
pub struct HeldItem {
    pub name: String,
    pub effect: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Attack {
    #[serde(default)]
    pub cost: Vec<String>,
    pub name: String,
    pub effect: Option<String>,
    pub damage: Option<serde_json::Value>,
}

impl Attack {
    pub fn damage_str(&self) -> Option<String> {
        self.damage
            .as_ref()
            .map(|d| d.to_string().trim_matches('"').to_string())
            .filter(|s| !s.is_empty())
    }
}

#[derive(Deserialize, Debug)]
pub struct Weakness {
    #[serde(rename = "type")]
    pub weakness_type: String,
    pub value: String,
}

pub type Resistance = Weakness;

#[derive(Deserialize, Debug)]
pub struct Pricing {
    pub cardmarket: Option<Cardmarket>,
    pub tcgplayer: Option<TcgPlayer>,
}

#[derive(Deserialize, Debug)]
pub struct Cardmarket {
    pub updated: Option<String>,
    pub unit: String,
    pub avg: Option<f64>,
    pub low: Option<f64>,
    pub trend: Option<f64>,
    pub avg1: Option<f64>,
    pub avg7: Option<f64>,
    pub avg30: Option<f64>,
    #[serde(rename = "avg-holo")]
    pub avg_holo: Option<f64>,
    #[serde(rename = "low-holo")]
    pub low_holo: Option<f64>,
    #[serde(rename = "trend-holo")]
    pub trend_holo: Option<f64>,
    #[serde(rename = "avg1-holo")]
    pub avg1_holo: Option<f64>,
    #[serde(rename = "avg7-holo")]
    pub avg7_holo: Option<f64>,
    #[serde(rename = "avg30-holo")]
    pub avg30_holo: Option<f64>,
}

#[derive(Deserialize, Debug)]
pub struct TcgPlayer {
    pub updated: Option<String>,
    pub unit: String,
    #[serde(flatten)]
    pub variants: HashMap<String, TcgPlayerPrice>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TcgPlayerPrice {
    pub low_price: Option<f64>,
    pub mid_price: Option<f64>,
    pub high_price: Option<f64>,
    pub market_price: Option<f64>,
    pub direct_low_price: Option<f64>,
}