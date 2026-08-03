use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Quote {
    #[serde(rename = "X")]
    pub ask_exchange: Option<i64>,
    #[serde(rename = "P")]
    pub ask_price: Option<f64>,
    #[serde(rename = "S")]
    pub ask_size: Option<f64>,
    #[serde(rename = "x")]
    pub bid_exchange: Option<i64>,
    #[serde(rename = "p")]
    pub bid_price: Option<f64>,
    #[serde(rename = "s")]
    pub bid_size: Option<f64>,
    pub conditions: Option<Vec<i64>>,
    pub indicators: Option<Vec<i64>>,
    #[serde(rename = "y")]
    pub participant_timestamp: Option<i64>,
    #[serde(rename = "q")]
    pub sequence_number: Option<i64>,
    #[serde(rename = "t")]
    pub sip_timestamp: Option<i64>,
    #[serde(rename = "z")]
    pub tape: Option<i64>,
    #[serde(rename = "f")]
    pub trf_timestamp: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LastQuote {
    #[serde(rename = "T")]
    pub ticker: Option<String>,
    #[serde(rename = "f")]
    pub trf_timestamp: Option<i64>,
    #[serde(rename = "q")]
    pub sequence_number: Option<i64>,
    #[serde(rename = "t")]
    pub sip_timestamp: Option<i64>,
    #[serde(rename = "y")]
    pub participant_timestamp: Option<i64>,
    #[serde(rename = "P")]
    pub ask_price: Option<f64>,
    #[serde(rename = "S")]
    pub ask_size: Option<i64>,
    #[serde(rename = "X")]
    pub ask_exchange: Option<i64>,
    pub conditions: Option<Vec<i64>>,
    pub indicators: Option<Vec<i64>>,
    #[serde(rename = "p")]
    pub bid_price: Option<f64>,
    #[serde(rename = "s")]
    pub bid_size: Option<i64>,
    #[serde(rename = "x")]
    pub bid_exchange: Option<i64>,
    #[serde(rename = "z")]
    pub tape: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ForexQuote {
    pub ask: Option<f64>,
    pub bid: Option<f64>,
    pub exchange: Option<i64>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct LastForexQuote {
    pub last: Option<ForexQuote>,
    pub symbol: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RealTimeCurrencyConversion {
    pub converted: Option<f64>,
    #[serde(rename = "from")]
    pub from_: Option<String>,
    #[serde(rename = "initialAmount")]
    pub initial_amount: Option<f64>,
    pub last: Option<ForexQuote>,
    pub to: Option<String>,
}
