use std::collections::HashMap;

use super::{contracts::WsBidsAsks, DepthOrderBookEvent};
use crate::{BaseContext, BidAsk, BinanceExchangeContext};

#[derive(Clone, Debug)]
pub struct BinanceOrderBook {
    pub instrument: String,
    pub date: i64,
    pub last_id: u128,
    pub bids: HashMap<String, f64>,
    pub asks: HashMap<String, f64>,
}

impl BinanceOrderBook {
    pub fn new(message: &DepthOrderBookEvent) -> BinanceOrderBook {
        BinanceOrderBook {
            instrument: message.symbol.to_uppercase(),
            date: message.event_time,
            last_id: message.final_update_id,
            bids: bid_ask_to_hash_map(&message.bids),
            asks: bid_ask_to_hash_map(&message.asks),
        }
    }

    pub fn is_valid(&self, socket_book: &DepthOrderBookEvent) -> bool {
        if socket_book.first_update_id == &self.last_id + 1
            || (socket_book.first_update_id <= self.last_id
                && self.last_id <= socket_book.final_update_id)
                && self.asks.len() < 3000
                && self.bids.len() < 3000
        {
            return true;
        }

        return false;
    }

    pub fn process_bids_and_asks(&mut self, socket_message: &DepthOrderBookEvent) {
        for tick in &socket_message.asks {
            let price = tick.price.clone();
            let volume = tick.qty.parse::<f64>().unwrap();

            if volume < 0.0001 && volume > -0.0001 {
                self.asks.remove(&price);
            } else {
                self.asks.insert(price, volume);
            }
        }

        for tick in &socket_message.bids {
            let price = tick.price.clone();
            let volume = tick.qty.parse::<f64>().unwrap();

            if volume < 0.0001 && volume > -0.0001 {
                self.bids.remove(&price);
            } else {
                self.bids.insert(price, volume);
            }
        }

        self.last_id = socket_message.final_update_id;
        self.date = socket_message.event_time;
    }

    pub fn get_best_price(&self) -> Option<BidAsk> {
        if self.bids.len() == 0 || self.asks.len() == 0 {
            return None;
        }

        let bid = self
            .bids
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .iter()
            .map(|bid| bid.parse::<f64>().unwrap())
            .collect::<Vec<f64>>();
        let ask = self
            .asks
            .keys()
            .cloned()
            .collect::<Vec<String>>()
            .iter()
            .map(|ask| ask.parse::<f64>().unwrap())
            .collect::<Vec<f64>>();

        return Some(BidAsk {
            date: self.date,
            id: self.instrument.clone(),
            ask: ask.clone().into_iter().fold(f64::NAN, f64::min),
            bid: bid.clone().into_iter().fold(f64::NAN, f64::max),
        });
    }
}

pub fn bid_ask_to_hash_map(bidasks: &Vec<WsBidsAsks>) -> HashMap<String, f64> {
    let mut hashmap = HashMap::new();
    for bidask in bidasks {
        hashmap.insert(bidask.price.clone(), bidask.qty.parse::<f64>().unwrap());
    }

    return hashmap;
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn test_orderbook_best_price() {
        let date = Utc::now().timestamp_millis();

        let ws_socket_event1 = DepthOrderBookEvent {
            event_type: "test".into(),
            event_time: date,
            symbol: "BTCUSD".into(),
            first_update_id: 1,
            final_update_id: 1,
            previous_final_update_id: None,
            bids: vec![
                WsBidsAsks {
                    price: "5.55".into(),
                    qty: "22.55".into(),
                },
                WsBidsAsks {
                    price: "4.52".into(),
                    qty: "21.55".into(),
                },
            ],
            asks: vec![
                WsBidsAsks {
                    price: "2.55".into(),
                    qty: "19.55".into(),
                },
                WsBidsAsks {
                    price: "3.52".into(),
                    qty: "51.55".into(),
                },
            ],
        };

        let ws_socket_event2 = DepthOrderBookEvent {
            event_type: "test".into(),
            event_time: date + 100,
            symbol: "BTCUSD".into(),
            first_update_id: 2,
            final_update_id: 2,
            previous_final_update_id: None,
            bids: vec![
                WsBidsAsks {
                    price: "6.55".into(),
                    qty: "26.55".into(),
                },
                WsBidsAsks {
                    price: "48.52".into(),
                    qty: "1.55".into(),
                },
            ],
            asks: vec![
                WsBidsAsks {
                    price: "76.55".into(),
                    qty: "21.55".into(),
                },
                WsBidsAsks {
                    price: "512.52".into(),
                    qty: "45.55".into(),
                },
            ],
        };

        let mut orderbook = BinanceOrderBook::new(&ws_socket_event1);
        let best_price_with_init = orderbook.get_best_price().unwrap();

        assert_eq!("BTCUSD", best_price_with_init.id);
        assert_eq!(5.55, best_price_with_init.bid);
        assert_eq!(2.55, best_price_with_init.ask);
        assert_eq!(date, best_price_with_init.date);

        orderbook.process_bids_and_asks(&ws_socket_event2);
        let best_price = orderbook.get_best_price().unwrap();

        assert_eq!("BTCUSD", best_price.id);
        assert_eq!(date + 100, best_price.date);
        assert_eq!(48.52, best_price.bid);
        assert_eq!(2.55, best_price.ask);
    }
}
