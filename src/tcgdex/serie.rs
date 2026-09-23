use serde::Deserialize;

use super::set::SetBrief;

#[derive(Deserialize, Debug)]
pub struct SerieBrief {
    pub id: String,
    pub name: String,
}

// #[derive(Deserialize, Debug)]
// pub struct Serie {
//     pub id: String,
//     pub name: String,
//     pub logo: Option<String>,
//     pub sets: Vec<SetBrief>
// }