use std::collections::{HashMap};
use crate::websocket_core::{BaseContext, BidAsk};
use chrono::{DateTime, NaiveDateTime, Utc};
use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;
use substring::{Substring};


use super::{BinanceOrderBook, DepthOrderBookEvent};

pub struct BinanceExchangeContext {
    pub instruments: Vec<String>,
    pub base_url: String,
    pub orderbooks: HashMap<String, BinanceOrderBook>,
}

impl BinanceExchangeContext {
    pub fn new(instruments: Vec<String>) -> BinanceExchangeContext {
        return BinanceExchangeContext {
            base_url: "wss://stream.binance.com:9443/stream?streams".to_string(),
            instruments: instruments,
            orderbooks: HashMap::new(),
        };
    }

    fn make_new_orderbook(&mut self, message: &DepthOrderBookEvent){
        let new_order_book = BinanceOrderBook::new(message);
        self.orderbooks.insert(message.symbol.clone(), new_order_book);
    }
}


impl BaseContext for BinanceExchangeContext {
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
            Err(err) => panic!("Cant serialize message to text.  Message: {}. Error: {}", message.to_string(), err)
        };

        let obj : Value = match serde_json::from_str(json_mess) {
            Ok(object) => object,
            Err(err) => panic!("Cant serialize message to object.  Message: {}. Error: {}", message.to_string(), err)
        };

        let data = obj.get("data");

        if data.is_none() {
            println!("Not found data field in obj.  Message: {}.", message.to_string());
            return None;
        }

        let socket_event: DepthOrderBookEvent = {
            let parsed_json = serde_json::from_str(&data.unwrap().to_string());
            match parsed_json {
                Ok(t) => t,
                Err(err) => panic!("Cant serialize json to DepthOrderBookEvent. Json: {}. Error: {}", data.unwrap().to_string(), err),
            }
        };

        if self.orderbooks.get(&socket_event.symbol).is_none() {
            self.make_new_orderbook(&socket_event);
        }

        let book = self.orderbooks.get_mut(&socket_event.symbol).unwrap();

        if socket_event.final_update_id <= book.last_id {
            return None;
        }

        if book.is_valid(&socket_event) {
            book.process_bids_and_asks(&socket_event);
        }
        
        return book.get_best_price();
    }

    fn parse_date(timestamp: String) -> String {

        let nanoseconds = timestamp.substring(10, 14).parse::<u32>().unwrap() * 1000000;
        let timestamp = timestamp.substring(0, 10).parse::<i64>().unwrap();
        let datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, nanoseconds), Utc);
        return datetime.format("%Y%m%d%H%M%S%3f").to_string();
    }
}
