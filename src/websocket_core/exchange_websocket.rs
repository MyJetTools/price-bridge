use std::{sync::{Arc, atomic::{AtomicBool, Ordering}}};
use futures::{SinkExt, StreamExt, Sink};

use tokio::{sync::mpsc::{UnboundedReceiver, UnboundedSender},};
use tokio_tungstenite::{connect_async};

use crate::{MetricsStore, websocket_core::WsMessageWriter};

use super::{BaseContext, BidAsk};

pub struct ExchangeWebscoket<T> {
    sender: Option<UnboundedSender<BidAsk>>,
    ctx: T,
    is_running: AtomicBool
}

impl<T: BaseContext> ExchangeWebscoket<T> {
    pub fn new(ctx: T) -> ExchangeWebscoket<T> {
        ExchangeWebscoket {
            sender: None,
            ctx: ctx,
            is_running: AtomicBool::new(false),
        }
    }

    pub fn get_subscribe(&mut self) -> UnboundedReceiver<BidAsk> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<BidAsk>();
        self.sender = Some(tx);
        return rx;
    }

    pub async fn start(mut self, metrics: Arc<MetricsStore>) {
        let url_to_connect = self.ctx.get_link_to_connect();
        println!("Connected to ws: {}", url_to_connect);
        self.is_running = AtomicBool::new(true);

        while self.is_running.load(Ordering::Relaxed) {

            let (ws_stream, _) = connect_async(&url_to_connect).await.unwrap();
            metrics.handle_change_connect_to_lp(true, &self.ctx.get_lp_name());

            let (sink, mut stream) = ws_stream.split();
            let message_sender = Arc::new(WsMessageWriter::new(sink));

            self.ctx.on_connect(message_sender).await;
            


            loop {
                let message = stream.next().await.unwrap();

                if message.is_err() {
                    metrics.handle_change_connect_to_lp(false, &self.ctx.get_lp_name());
                }

                let bidask = self.ctx.handle_message_and_get_bid_ask(message.unwrap());

                match bidask {
                    Some(evnt) => {
                        metrics.handle_quote_income(&self.ctx.get_lp_name(), &evnt);
                        self.sender.as_ref().unwrap().send(evnt).unwrap();
                    }
                    None => {}
                }
                
            }
        }
    }
}
