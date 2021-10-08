use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::websocket_core::BidAsk;


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WsBidsAsks {
    pub price: String,
    pub qty: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DepthOrderBookEvent {
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
    pub bids: Vec<WsBidsAsks>,

    #[serde(rename = "a")]
    pub asks: Vec<WsBidsAsks>
}


pub fn order_book_event_to_bid_ask(book_event: DepthOrderBookEvent) -> BidAsk{
    BidAsk{
        bid: book_event.bids.first().unwrap().price.parse::<f64>().unwrap(),
        ask: book_event.asks.first().unwrap().price.parse::<f64>().unwrap(),
        id: book_event.symbol,
        date: book_event.event_time,
    }
}