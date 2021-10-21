use std::sync::Arc;

use tokio_tungstenite::{tungstenite::Message};

use super::{BidAsk, WsMessageWriter};
use async_trait::async_trait;

#[async_trait]
pub trait BaseContext {
    fn get_link_to_connect(&self) -> String;
    fn handle_message_and_get_bid_ask(&mut self, message: Message) -> Option<BidAsk>;
    async fn on_connect(&self, message_writer: Arc<WsMessageWriter>);
}