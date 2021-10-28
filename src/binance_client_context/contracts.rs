use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BinanceWsBidsAsks {
    pub price: String,
    pub qty: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BinanceDepthOrderBookEvent {
    #[serde(rename = "e")]
    pub event_type: String,

    #[serde(rename = "E")]
    pub event_time: i64,

    #[serde(rename = "s")]
    pub symbol: String,

    #[serde(rename = "U")]
    pub first_update_id: u128,

    #[serde(rename = "u")]
    pub final_update_id: u128,

    #[serde(rename = "pu")]
    #[serde(default)]
    pub previous_final_update_id: Option<u64>,

    #[serde(rename = "b")]
    pub bids: Vec<BinanceWsBidsAsks>,

    #[serde(rename = "a")]
    pub asks: Vec<BinanceWsBidsAsks>
}

//[320,{"a":[["62203.20000","0.09848723","1634644467.091138"]],"c":"3471159483"},"book-10","XBT/USD"]