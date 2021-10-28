use super::{
    contracts::{OrderBookEvent, OrderBookSnapshotEvent, SubscribeToKraken, SubscriptionKraken},
    KrakenOrderBook,
};
use crate::Settings;
use crate::kraken_client_context::contracts::WsBidsAsksSnapshot;
use crate::websocket_core::{BaseContext, BidAsk};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::stream::SplitSink;
use serde_json::{json, Value};
use std::collections::HashMap;
use substring::Substring;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::kraken_client_context::contracts::{
    WsBidAskContainer, WsBidAskSnapshotContainer, WsBidsAsks,
};
use crate::websocket_core::WsMessageWriter;
use std::sync::Arc;

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

    pub fn new_by_settings(settings: &Settings) -> KrakenExchangeContext {
        return KrakenExchangeContext {
            base_url: "wss://ws.kraken.com".to_string(),
            instruments: settings
                .instruments_mapping
                .keys()
                .cloned()
                .collect::<Vec<String>>(),
            orderbooks: HashMap::new(),
        };
    }

    fn make_new_orderbook(&mut self, message: &OrderBookSnapshotEvent) {
        let new_order_book = KrakenOrderBook::new(message);
        self.orderbooks.insert(message.pair.clone(), new_order_book);
    }
}

#[async_trait]
impl BaseContext for KrakenExchangeContext {
    fn get_link_to_connect(&self) -> String {
        return self.base_url.clone();
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

        let mut snapshot: OrderBookSnapshotEvent;
        let mut pair: String;
        let mut orderbook_snapshot_asks: Vec<WsBidsAsksSnapshot>;
        let mut orderbook_snapshot_bids: Vec<WsBidsAsksSnapshot>;
        let mut orderbook_asks: Vec<WsBidsAsks> = vec![];
        let mut orderbook_bids: Vec<WsBidsAsks> = vec![];
        let empty = vec![json!([""])];
        match obj {
            Value::Array(arr) => {
                if arr.len() == 4 {
                    let map = &arr[1];
                    pair = arr[3].to_string();
                    let as1 = map.get("as");
                    let a = map.get("a");
                    let b = map.get("b");

                    if let Some(as_val) = as1 {
                        let arr1 = match as_val {
                            Value::Array(arr1) => arr1,
                            _ => &empty,
                        };

                        orderbook_snapshot_asks = arr1
                            .iter()
                            .map(|v| {
                                let arr2 = match v {
                                    Value::Array(arr1) => arr1,
                                    _ => &empty,
                                };

                                WsBidsAsksSnapshot {
                                    price: arr2[0].as_str().unwrap().to_string(),
                                    qty: arr2[1].as_str().unwrap().to_string(),
                                    time: arr2[2].as_str().unwrap().to_string(),
                                }
                            })
                            .collect();
                        let bs1 = map.get("bs");
                        if let Some(bs_val) = bs1 {
                            let arr1 = match bs_val {
                                Value::Array(arr1) => arr1,
                                _ => &empty,
                            };

                            orderbook_snapshot_bids = arr1
                                .iter()
                                .map(|v| {
                                    let arr2 = match v {
                                        Value::Array(arr1) => arr1,
                                        _ => &empty,
                                    };

                                    WsBidsAsksSnapshot {
                                        price: arr2[0].as_str().unwrap().to_string(),
                                        qty: arr2[1].as_str().unwrap().to_string(),
                                        time: arr2[2].as_str().unwrap().to_string(),
                                    }
                                })
                                .collect();
                        } else {
                            orderbook_snapshot_bids = vec![]
                        }

                        if self.orderbooks.get(&pair).is_none() {
                            let event = OrderBookSnapshotEvent {
                                bid_ask: WsBidAskSnapshotContainer {
                                    bs_vec: orderbook_snapshot_bids,
                                    as_vec: orderbook_snapshot_asks,
                                },
                                pair: pair.clone(),
                            };
                            self.make_new_orderbook(&event);
                        }

                        let book = self.orderbooks.get_mut(&pair).unwrap();
                        return book.get_best_price();
                    } else if let Some(asks) = a {
                        let arr1 = match asks {
                            Value::Array(arr1) => arr1,
                            _ => &empty,
                        };

                        orderbook_asks = arr1
                            .iter()
                            .map(|v| {
                                let arr2 = match v {
                                    Value::Array(arr1) => arr1,
                                    _ => &empty,
                                };

                                if arr2.len() == 4 {
                                    WsBidsAsks {
                                        price: arr2[0].as_str().unwrap().to_string(),
                                        qty: arr2[1].as_str().unwrap().to_string(),
                                        time: arr2[2].as_str().unwrap().to_string(),
                                        republished: arr2[3].as_str().unwrap().to_string(),
                                    }
                                } else {
                                    WsBidsAsks {
                                        price: arr2[0].as_str().unwrap().to_string(),
                                        qty: arr2[1].as_str().unwrap().to_string(),
                                        time: arr2[2].as_str().unwrap().to_string(),
                                        republished: "".to_string(),
                                    }
                                }
                            })
                            .collect();

                            orderbook_bids = vec![];
                    } else {
                        if let Some(bids) = b {
                            let arr1 = match bids {
                                Value::Array(arr1) => arr1,
                                _ => &empty,
                            };
                            orderbook_asks = vec![];
                            orderbook_bids = arr1
                                .iter()
                                .map(|v| {
                                    let arr2 = match v {
                                        Value::Array(arr1) => arr1,
                                        _ => &empty,
                                    };

                                    if arr2.len() == 4 {
                                        WsBidsAsks {
                                            price: arr2[0].as_str().unwrap().to_string(),
                                            qty: arr2[1].as_str().unwrap().to_string(),
                                            time: arr2[2].as_str().unwrap().to_string(),
                                            republished: arr2[3].as_str().unwrap().to_string(),
                                        }
                                    } else {
                                        WsBidsAsks {
                                            price: arr2[0].as_str().unwrap().to_string(),
                                            qty: arr2[1].as_str().unwrap().to_string(),
                                            time: arr2[2].as_str().unwrap().to_string(),
                                            republished: "".to_string(),
                                        }
                                    }
                                })
                                .collect();
                        }
                    }
                } else {
                    if let Some(asks) = arr[1].get("a") {
                        let arr2 = match asks {
                            Value::Array(asks) => asks,
                            _ => &empty,
                        };

                        orderbook_asks = arr2
                            .iter()
                            .map(|v| {
                                let arr2 = match v {
                                    Value::Array(arr1) => arr1,
                                    _ => &empty,
                                };

                                if arr2.len() == 4 {
                                    WsBidsAsks {
                                        price: arr2[0].as_str().unwrap().to_string(),
                                        qty: arr2[1].as_str().unwrap().to_string(),
                                        time: arr2[2].as_str().unwrap().to_string(),
                                        republished: arr2[3].as_str().unwrap().to_string(),
                                    }
                                } else {
                                    WsBidsAsks {
                                        price: arr2[0].as_str().unwrap().to_string(),
                                        qty: arr2[1].as_str().unwrap().to_string(),
                                        time: arr2[2].as_str().unwrap().to_string(),
                                        republished: "".to_string(),
                                    }
                                }
                            })
                            .collect();
                    } else {
                        orderbook_asks = vec![];
                    }

                    if let Some(bids) = arr[2].get("b") {
                        let arr2 = match bids {
                            Value::Array(bids) => bids,
                            _ => &empty,
                        };

                        orderbook_bids = arr2
                            .iter()
                            .map(|v| {
                                let arr2 = match v {
                                    Value::Array(arr1) => arr1,
                                    _ => &empty,
                                };

                                if arr2.len() == 4 {
                                    WsBidsAsks {
                                        price: arr2[0].as_str().unwrap().to_string(),
                                        qty: arr2[1].as_str().unwrap().to_string(),
                                        time: arr2[2].as_str().unwrap().to_string(),
                                        republished: arr2[3].as_str().unwrap().to_string(),
                                    }
                                } else {
                                    WsBidsAsks {
                                        price: arr2[0].as_str().unwrap().to_string(),
                                        qty: arr2[1].as_str().unwrap().to_string(),
                                        time: arr2[2].as_str().unwrap().to_string(),
                                        republished: "".to_string(),
                                    }
                                }
                            })
                            .collect();
                    } else {
                        orderbook_bids = vec![];
                    }

                    pair = arr[4].as_str().unwrap().to_string();
                }
            }
            _ => {
                return None;
            }
        };

        let event = OrderBookEvent {
            bid_ask: WsBidAskContainer {
                as_vec: orderbook_asks,
                bs_vec: orderbook_bids,
            },
            pair: pair.clone(),
        };
        let book = self.orderbooks.get_mut(&pair).unwrap();

        if book.is_valid(&event) {
            book.process_bids_and_asks(&event);
        }

        return book.get_best_price();
    }

    async fn on_connect(&self, message_writer: Arc<WsMessageWriter>) {
        let symbols: Vec<String> = self
            .instruments
            .clone()
            .into_iter()
            .map(String::from)
            .collect();
        let message_writer = Arc::clone(&message_writer);
        let subscribe = SubscribeToKraken {
            event: "subscribe".into(),
            pair: symbols,
            subscription: SubscriptionKraken {
                name: "book".into(),
            },
        };

        let serialized = serde_json::to_string(&subscribe).unwrap();
        let message = Message::Text(serialized);
        message_writer.send_data(message).await;
    }

    fn get_lp_name(&self)->String {
        return "binance".into();
    }
}
