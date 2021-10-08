use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct BidAsk{
    pub bid: f64,
    pub ask: f64,
    pub id: String,
    pub date: i64
}