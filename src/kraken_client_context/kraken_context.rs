use crate::websocket_core::{BaseContext, BidAsk};
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::stream::SplitSink;
use serde_json::Value;
use tokio::net::TcpStream;
use std::collections::HashMap;
use substring::Substring;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use async_trait::async_trait;
use super::{
    contracts::{
        OrderBookEvent, OrderBookSnapshotEvent, RootOrderBookEvent, RootOrderBookSnapshotEvent,
    },
    KrakenOrderBook,
};

pub struct KrakenExchangeContext {
    pub instruments: Vec<String>,
    pub base_url: String,
    pub orderbooks: HashMap<String, KrakenOrderBook>,
}

impl KrakenExchangeContext {
    pub fn new(instruments: Vec<String>) -> KrakenExchangeContext {
        return KrakenExchangeContext {
            base_url: "wss://ws.kraken.com".to_string(),
            instruments: instruments,
            orderbooks: HashMap::new(),
        };
    }

    fn make_new_orderbook(&mut self, message: &RootOrderBookSnapshotEvent) {
        let new_order_book = KrakenOrderBook::new(message);
        let first = message.bid_asks.first();

        if let Some(val) = first {
            self.orderbooks.insert(val.pair.clone(), new_order_book);
        }
    }
}

#[async_trait]
impl BaseContext for KrakenExchangeContext {

    async fn subscribe_if_needed(&self, sink: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) -> Result<(), ()>
    {
        return Ok(());
    }

    fn get_link_to_connect(&self) -> String {
        let symbols: Vec<String> = self
            .instruments
            .clone()
            .into_iter()
            .map(String::from)
            .collect();

        let mut endpoints: Vec<String> = Vec::new();

        for symbol in symbols.iter() {
            endpoints.push(format!("{}@depth@100ms", symbol.to_lowercase()));
        }

        return format!("{}={}", self.base_url, endpoints.join("/"));
    }

    fn handle_message_and_get_bid_ask(&mut self, message: Message) -> Option<BidAsk> {
        let json_mess = match message.to_text() {
            Ok(str) => str,
            Err(err) => panic!(
                "Cant serialize message to text.  Message: {}. Error: {}",
                message.to_string(),
                err
            ),
        };

        let obj: Value = match serde_json::from_str(json_mess) {
            Ok(object) => object,
            Err(err) => panic!(
                "Cant serialize message to object.  Message: {}. Error: {}",
                message.to_string(),
                err
            ),
        };

        /* let data = obj.get("data");

        if data.is_none() {
            println!("Not found data field in obj.  Message: {}.", message.to_string());
            return None;
        } */

        let socket_event: Option<RootOrderBookSnapshotEvent> = {
            let parsed_json = serde_json::from_str(&obj.to_string());
            match parsed_json {
                Ok(t) => t,
                Err(err) => None, //panic!("Cant serialize json to RootOrderBookSnapshotEvent. Json: {}. Error: {}", data.unwrap().to_string(), err),
            }
        };

        if let Some(snapshot) = socket_event {
            let first = snapshot.bid_asks.first();
            
            if let Some(val) = first {
                if self.orderbooks.get(&val.pair).is_none() {
                    self.make_new_orderbook(&snapshot);
                }

                let book = self.orderbooks.get_mut(&val.pair).unwrap();
                return book.get_best_price();
            }
        }

        let socket_event: Option<RootOrderBookEvent> = {
            let parsed_json = serde_json::from_str(&obj.to_string());
            match parsed_json {
                Ok(t) => t,
                Err(err) => None, //panic!("Cant serialize json to RootOrderBookSnapshotEvent. Json: {}. Error: {}", data.unwrap().to_string(), err),
            }
        };

        if let Some(event) = socket_event {
            let first = event.bid_asks.first();
            if let Some(val) = first {
                let book = self.orderbooks.get_mut(&val.pair).unwrap();

                if book.is_valid(&event) {
                    book.process_bids_and_asks(&event);
                }

                return book.get_best_price();
            }
        }

        //HeartBeat probably
        return None;
    }

    fn parse_date(timestamp: String) -> String {
        let nanoseconds = timestamp.substring(10, 14).parse::<u32>().unwrap() * 1000000;
        let timestamp = timestamp.substring(0, 10).parse::<i64>().unwrap();
        let datetime =
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, nanoseconds), Utc);
        return datetime.format("%Y%m%d%H%M%S%3f").to_string();
    }
}
