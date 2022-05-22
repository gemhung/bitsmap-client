use serde::de;
use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
pub struct OfferData {
    #[serde(deserialize_with = "de_float_from_str")]
    pub price: f32,
    #[serde(deserialize_with = "de_float_from_str")]
    pub qty: f32,
}

#[derive(Debug, Deserialize)]
pub struct Book {
    pub timestamp: String,
    pub microtimestamp: String,
    pub bids: Vec<OfferData>,
    pub asks: Vec<OfferData>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Data {
    Book(Book),
    None {},
}

#[derive(Debug, Deserialize)]
pub struct BitsMap {
    pub channel: String,
    pub event: String,
    pub data: Data,
}

pub fn de_float_from_str<'a, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: Deserializer<'a>,
{
    let str_val = String::deserialize(deserializer)?;
    str_val.parse::<f32>().map_err(de::Error::custom)
}
