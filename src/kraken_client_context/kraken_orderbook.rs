use std::collections::{btree_map, BTreeMap, HashMap};

use super::contracts::{
    OrderBookEvent, OrderBookSnapshotEvent, WsBidAskContainer, WsBidAskSnapshotContainer,
    WsBidsAsks, WsBidsAsksSnapshot,
};
use crate::BidAsk;

#[derive(Clone, Debug)]
pub struct KrakenOrderBook {
    pub instrument: String,
    pub date: i64,
    pub last_id: u128,
    pub bids: BTreeMap<String, (f64, f64)>,
    pub asks: BTreeMap<String, (f64, f64)>,
}

impl KrakenOrderBook {
    pub fn new(message: &OrderBookSnapshotEvent) -> KrakenOrderBook {
        return KrakenOrderBook {
            instrument: message.pair.to_uppercase(),
            //date: val.,
            date: 0,
            last_id: 0,
            bids: bid_ask_to_btree_map(&message.bid_ask.bs_vec),
            asks: bid_ask_to_btree_map(&message.bid_ask.as_vec),
        };
    }

    pub fn is_valid(&self, socket_book: &OrderBookEvent) -> bool {
        return true;

        /* if //socket_book.first_update_id == &self.last_id + 1
            //|| (socket_book.first_update_id <= self.last_id
              //  && self.last_id <= socket_book.final_update_id)
                self.asks.len() < 1000
                && self.bids.len() < 1000
        {
            return true;
        }

        return false; */
    }

    pub fn process_bids_and_asks(&mut self, socket_message: &OrderBookEvent) {
        let container = &socket_message.bid_ask;
        for tick in &container.as_vec {
            let price = tick.price.clone();
            let volume = tick.qty.parse::<f64>().unwrap();
            let time = tick.time.parse::<f64>().unwrap();

            if volume < 0.00000001 && volume > -0.00000001 {
                self.asks.remove(&price);
            } else {
                self.asks.insert(price, (volume, time));
            }
        }

        for tick in &container.bs_vec {
            let price = tick.price.clone();
            let volume = tick.qty.parse::<f64>().unwrap();
            let time = tick.time.parse::<f64>().unwrap();

            if volume < 0.00000001 && volume > -0.00000001 {
                self.bids.remove(&price);
            } else {
                self.bids.insert(price, (volume, time));
            }
        }

        //self.last_id = socket_message.final_update_id;
        //self.date = socket_message.event_time;
    }

    pub fn get_best_price(&self) -> Option<BidAsk> {
        if self.bids.len() == 0 || self.asks.len() == 0 {
            return None;
        }

        let mut ask_price: f64 = 0.0;
        for pair in self.asks.iter() {
            ask_price = pair.0.parse::<f64>().unwrap();
            break;
        }

        let mut bid_price: f64 = 0.0;
        for pair in self.bids.iter().rev() {
            bid_price = pair.0.parse::<f64>().unwrap();
            break;
        }

        return Some(BidAsk {
            date: self.date,
            id: self.instrument.clone(),
            ask: ask_price,
            bid: bid_price,
        });
    }
}

pub fn bid_ask_to_btree_map(bidasks: &Vec<WsBidsAsksSnapshot>) -> BTreeMap<String, (f64, f64)> {
    let mut btree_map = BTreeMap::new();
    for bidask in bidasks {
        btree_map.insert(
            bidask.price.clone(),
            (
                bidask.qty.parse::<f64>().unwrap(),
                bidask.time.parse::<f64>().unwrap(),
            ),
        );
    }

    return btree_map;
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use crate::kraken_client_context::contracts::OrderBookSnapshotEvent;

    use super::*;

    #[test]
    fn test_orderbook_best_price() {
        let date = Utc::now().timestamp_millis();

        let ws_socket_event1 = OrderBookSnapshotEvent {
            bid_ask: WsBidAskSnapshotContainer {
                as_vec: vec![WsBidsAsksSnapshot {
                    price: "1.0".into(),
                    qty: "1.0".into(),
                    time: "1.0".into(),
                }],
                bs_vec: vec![WsBidsAsksSnapshot {
                    price: "0.9".into(),
                    qty: "1.0".into(),
                    time: "1.0".into(),
                }],
            },
            pair: "XBT/USD".into(),
        };

        let ws_socket_event2 = OrderBookEvent {
            bid_ask: WsBidAskContainer {
                as_vec: vec![WsBidsAsks {
                    price: "0.95".into(),
                    qty: "1.0".into(),
                    time: "1.0".into(),
                    republished: "".into(),
                }],
                bs_vec: vec![WsBidsAsks {
                    price: "0.94".into(),
                    qty: "1.0".into(),
                    time: "1.0".into(),
                    republished: "".into(),
                }],
            },
            pair: "XBT/USD".into(),
        };

        let mut orderbook = KrakenOrderBook::new(&ws_socket_event1);
        let best_price_with_init = orderbook.get_best_price().unwrap();

        assert_eq!("XBT/USD", best_price_with_init.id);
        assert_eq!(0.9, best_price_with_init.bid);
        assert_eq!(1.0, best_price_with_init.ask);
        //assert_eq!(date, best_price_with_init.date);

        orderbook.process_bids_and_asks(&ws_socket_event2);
        let best_price = orderbook.get_best_price().unwrap();

        assert_eq!("XBT/USD", best_price.id);
        //assert_eq!(date + 100, best_price.date);
        assert_eq!(0.94, best_price.bid);
        assert_eq!(0.95, best_price.ask);
    }
}
