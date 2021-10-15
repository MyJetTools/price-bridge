use tokio_tungstenite::{tungstenite::Message};

use super::BidAsk;

pub trait BaseContext {
    fn get_link_to_connect(&self) -> String;
    fn handle_message_and_get_bid_ask(&mut self, message: Message) -> Option<BidAsk>;
    fn parse_date(date: String) -> String;
}