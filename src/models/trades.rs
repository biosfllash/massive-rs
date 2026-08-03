use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Trade {
    pub conditions: Option<Vec<i64>>,
    pub correction: Option<i64>,
    pub exchange: Option<i64>,
    #[serde(rename = "i")]
    pub id: Option<String>,
    #[serde(rename = "y")]
    pub participant_timestamp: Option<i64>,
    #[serde(rename = "p")]
    pub price: Option<f64>,
    #[serde(rename = "q")]
    pub sequence_number: Option<i64>,
    #[serde(rename = "t")]
    pub sip_timestamp: Option<i64>,
    #[serde(rename = "s")]
    pub size: Option<f64>,
    #[serde(rename = "z")]
    pub tape: Option<i64>,
    #[serde(rename = "r")]
    pub trf_id: Option<i64>,
    #[serde(rename = "f")]
    pub trf_timestamp: Option<i64>,
    #[serde(rename = "ds")]
    pub decimal_size: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LastTrade {
    #[serde(rename = "T")]
    pub ticker: Option<String>,
    #[serde(rename = "f")]
    pub trf_timestamp: Option<i64>,
    #[serde(rename = "q")]
    pub sequence_number: Option<f64>,
    #[serde(rename = "t")]
    pub sip_timestamp: Option<i64>,
    #[serde(rename = "y")]
    pub participant_timestamp: Option<i64>,
    pub conditions: Option<Vec<i64>>,
    pub correction: Option<i64>,
    #[serde(rename = "i")]
    pub id: Option<String>,
    #[serde(rename = "p")]
    pub price: Option<f64>,
    #[serde(rename = "r")]
    pub trf_id: Option<i64>,
    #[serde(rename = "s")]
    pub size: Option<f64>,
    #[serde(rename = "x")]
    pub exchange: Option<i64>,
    #[serde(rename = "z")]
    pub tape: Option<i64>,
    #[serde(rename = "ds")]
    pub fractional_size: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct CryptoTrade {
    pub conditions: Option<Vec<i64>>,
    pub exchange: Option<i64>,
    pub price: Option<f64>,
    pub size: Option<f64>,
    pub timestamp: Option<i64>,
}
