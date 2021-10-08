use std::sync::atomic::{AtomicBool, Ordering};
use futures::StreamExt;

use tokio::{sync::mpsc::{UnboundedReceiver, UnboundedSender},};
use tokio_tungstenite::{connect_async};

use super::{BaseContext, BidAsk};

pub struct ExchangeWebscoket<T> {
    sender: Option<UnboundedSender<BidAsk>>,
    ctx: T,
    is_running: AtomicBool,
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

    pub async fn start(mut self) {

        let url_to_connect = self.ctx.get_link_to_connect();
        println!("{}", url_to_connect);
        self.is_running = AtomicBool::new(true);

        while self.is_running.load(Ordering::Relaxed) {

            let (mut ws_stream, _) = connect_async(&url_to_connect).await.unwrap();


            loop {
                let message = ws_stream.next().await.unwrap().unwrap();
                let bidask = self.ctx.handle_message_and_get_bid_ask(message);

                match bidask {
                    Some(evnt) => {
                        self.sender.as_ref().unwrap().send(evnt);
                    }
                    None => {}
                }
            }
        }
    }
}
