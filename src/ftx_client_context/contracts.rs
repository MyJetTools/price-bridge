use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsBidsAsks {
    pub price: String,
    pub qty: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FtxTickerMessageData{
    #[serde(rename = "bid")]
    pub bid: f64,
    #[serde(rename = "ask")]
    pub ask: f64,
    #[serde(rename = "bidSize")]
    pub bid_size: f64,
    #[serde(rename = "askSize")]
    pub ask_size: f64,
    #[serde(rename = "time")]
    pub time: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FtxTickerMessage{
    #[serde(rename = "channel")]
    pub channel: String,
    #[serde(rename = "market")]
    pub market: String,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(rename = "data")]
    pub data: FtxTickerMessageData,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FtxSubscribeMessage{
    #[serde(rename = "op")]
    pub mess_type: String,
    #[serde(rename = "channel")]
    pub channel: String,
    #[serde(rename = "market")]
    pub instrument: String,
}

impl FtxSubscribeMessage {
    pub fn make_ticket_subscribe_by_instrument(instrument: &String) -> FtxSubscribeMessage{
        FtxSubscribeMessage{
            mess_type: "subscribe".into(),
            channel: "ticker".into(),
            instrument: instrument.into()
        }
    }
}