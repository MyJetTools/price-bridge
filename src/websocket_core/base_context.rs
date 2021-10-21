use std::sync::Arc;

use tokio_tungstenite::{tungstenite::Message};
use futures::stream::SplitSink;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::MaybeTlsStream;
use super::{BidAsk, WsMessageWriter};
use async_trait::async_trait;
use tokio::net::TcpStream;

#[async_trait]
pub trait BaseContext {
    fn get_link_to_connect(&self) -> String;
    fn handle_message_and_get_bid_ask(&mut self, message: Message) -> Option<BidAsk>;
    async fn on_connect(&self, message_writer: Arc<WsMessageWriter>);
}