use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsBidsAsksSnapshot {
    pub price: String,
    pub qty: String,
    pub time: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsBidAskSnapshotContainer {
    pub as_vec: Vec<WsBidsAsksSnapshot>,

    pub bs_vec: Vec<WsBidsAsksSnapshot>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderBookSnapshotEvent {
    pub bid_ask: WsBidAskSnapshotContainer,

    pub pair: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsBidsAsks {
    pub price: String,
    pub qty: String,
    pub time: String,
    pub republished: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WsBidAskContainer {
    #[serde(rename = "a")]
    pub as_vec: Vec<WsBidsAsks>,

    #[serde(rename = "b")]
    pub bs_vec: Vec<WsBidsAsks>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OrderBookEvent {
    pub bid_ask: WsBidAskContainer,

    pub pair: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeToKraken {
    pub event: String,
    pub pair: Vec<String>,
    pub subscription: SubscriptionKraken,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubscriptionKraken {
    pub name: String,
}
//[320,{"a":[["62203.20000","0.09848723","1634644467.091138"]],"c":"3471159483"},"book-10","XBT/USD"]
//[320,{"as":[["62200.00000","5.58539905","1634645635.905132"],["62200.20000","2.41178281","1634645635.087010"],["62204.20000","0.40107937","1634645630.161740"],["62204.40000","0.42954747","1634645635.515127"],["62205.80000","1.10283176","1634645631.031071"],["62206.60000","0.00997124","1634645582.887873"],["62207.20000","4.01963677","1634645635.241855"],["62209.20000","1.10968097","1634645633.669583"],["62210.40000","0.03667578","1634645633.823980"],["62214.50000","0.27966537","1634645625.988158"]],"bs":[["62199.90000","0.01066044","1634645633.737771"],["62198.10000","0.00060000","1634645585.901944"],["62187.60000","0.03241399","1634645635.668679"],["62187.50000","0.03202250","1634645634.764619"],["62180.80000","0.08547701","1634645635.960172"],["62180.60000","0.15893165","1634645634.929793"],["62180.40000","0.08691998","1634645634.089003"],["62179.90000","0.06048606","1634645597.294984"],["62179.70000","0.06289686","1634645597.279263"],["62179.30000","4.01982296","1634645635.818219"]]},"book-10","XBT/USD"]
/*
{
  "event": "subscribe",
  "pair": [
    "XBT/USD"
  ],
  "subscription": {
    "name": "book"
  }
}
*/