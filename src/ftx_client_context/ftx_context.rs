use std::collections::HashMap;

use serde_json::Value;
use tokio_tungstenite::tungstenite::Message;

use crate::{BaseContext, BidAsk, Settings};
use async_trait::async_trait;

use super::contracts::{FtxSubscribeMessage, FtxTickerMessage};

pub struct FtxExchangeContext {
    pub instruments: Vec<String>,
    pub base_url: String,
    pub last_bid_ask: HashMap<String, FtxTickerMessage>,
}

impl FtxExchangeContext {
    pub fn new(instruments: Vec<String>) -> FtxExchangeContext {
        return FtxExchangeContext {
            base_url: "wss://ftx.com/ws/".to_string(),
            instruments: instruments,
            last_bid_ask: HashMap::new(),
        };
    }

    pub fn new_by_settings(settings: &Settings) -> FtxExchangeContext {
        return FtxExchangeContext {
            base_url: "wss://ftx.com/ws/".to_string(),
            instruments: settings
                .instruments_mapping
                .keys()
                .cloned()
                .collect::<Vec<String>>(),
            last_bid_ask: HashMap::new(),
        };
    }
}

#[async_trait]
impl BaseContext for FtxExchangeContext {
    fn get_link_to_connect(&self) -> String {
        return self.base_url.clone();
    }

    fn handle_message_and_get_bid_ask(&mut self, message: Message) -> Option<crate::BidAsk> {
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

        let mess_type = obj.get("type");

        if mess_type.is_none() {
            println!("No type field. Skip message. Message: {}", json_mess);
            return None;
        }

        if mess_type.unwrap().to_string() != "\"update\"" {
            println!("Field is not update. Skip message. Message: {}", json_mess);
            return None;
        }

        // if data.is_none() {
        //     println!("Not found data field in obj.  Message: {}.", message.to_string());
        //     return None;
        // }

        let socket_event: FtxTickerMessage = {
            let parsed_json = serde_json::from_str(&obj.to_string());
            match parsed_json {
                Ok(t) => t,
                Err(err) => panic!(
                    "Cant serialize json to FtxTickerMessage. Json: {}. Error: {}",
                    obj.to_string(),
                    err
                ),
            }
        };

        let last_bid_ask = self.last_bid_ask.get(&socket_event.market);

        if last_bid_ask.is_none() {
            let bidask = ticker_to_bid_ask(&socket_event);
            self.last_bid_ask
                .insert(socket_event.market.clone(), socket_event);

            return Some(bidask);
        }

        let unwrap_last_bid_ask = last_bid_ask.unwrap();

        if unwrap_last_bid_ask.data.time < socket_event.data.time {
            return Some(ticker_to_bid_ask(unwrap_last_bid_ask));
        }

        return None;
    }

    async fn on_connect(
        &self,
        message_writer: std::sync::Arc<crate::websocket_core::WsMessageWriter>,
    ) {
        for instument in self.instruments.iter() {
            let object_to_send =
                FtxSubscribeMessage::make_ticket_subscribe_by_instrument(instument);
            let subscribe_message = serde_json::to_string(&object_to_send).unwrap();
            let message_to_subscribe = Message::text(subscribe_message);
            message_writer.send_data(message_to_subscribe).await;
            println!("Sent subscribe message: {:?}", object_to_send);
        }
    }

    fn get_lp_name(&self) -> String {
        return "ftx".into();
    }
}

fn ticker_to_bid_ask(ticker: &FtxTickerMessage) -> BidAsk {
    BidAsk {
        bid: ticker.data.bid,
        ask: ticker.data.ask,
        id: ticker.market.clone(),
        date: parse_date_to_timestamp(ticker.data.time),
    }
}

fn parse_date_to_timestamp(date: f64) -> i64 {
    let split_date = date.to_string();
    let split_date = split_date.split(".");
    let date_parts: Vec<&str> = split_date.collect();
    let date_as_string = format!(
        "{}{}",
        date_parts[0].to_string(),
        date_parts[1][..3].to_string()
    );

    return date_as_string.parse::<i64>().unwrap();
}
