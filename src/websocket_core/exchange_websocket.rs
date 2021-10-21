use std::{collections::HashMap, sync::{Arc, atomic::{AtomicBool, Ordering}}};
use futures::StreamExt;

use prometheus::{core::{AtomicF64, GenericCounter}};
use stopwatch::Stopwatch;
use tokio::{sync::mpsc::{UnboundedReceiver, UnboundedSender},};
use tokio_tungstenite::{connect_async};

use crate::{Metrics, websocket_core::WsMessageWriter};

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

    pub async fn start(mut self, metrics: Arc<Metrics>) {

        let url_to_connect = self.ctx.get_link_to_connect();
        println!("{}", url_to_connect);
        self.is_running = AtomicBool::new(true);

        while self.is_running.load(Ordering::Relaxed) {
            metrics.is_connected.inc();
            let sw = Stopwatch::start_new();
            let (ws_stream, _) = connect_async(&url_to_connect).await.unwrap();

            let (sink, mut stream) = ws_stream.split();
            
            let message_sender = Arc::new(WsMessageWriter::new(sink));
            self.ctx.on_connect(message_sender).await;
            
            let mut instrument_metrics: HashMap<String, GenericCounter<AtomicF64>> = HashMap::new();

            loop {
                let message = stream.next().await.unwrap();

                if message.is_err() {
                    metrics.is_connected.dec();
                }

                let bidask = self.ctx.handle_message_and_get_bid_ask(message.unwrap());

                match bidask {
                    Some(evnt) => {
                        {
                            let metric = instrument_metrics.get(&evnt.id);

                            if metric.is_none() {
                                let metric_to_insert_into_list = metrics.quote_income.with_label_values(&[&evnt.id]);
                                instrument_metrics.insert(evnt.id.clone(), metric_to_insert_into_list);
                            }
                        }

                        let id = evnt.id.clone();
                        self.sender.as_ref().unwrap().send(evnt).unwrap();
                        instrument_metrics.get(&id).unwrap().inc();
                        metrics.quote_process_time_sm_ws.with_label_values(&[&id]).inc_by(sw.elapsed_ms() as f64);
                        metrics.quote_process_time_ws_sm_count.with_label_values(&[&id]).inc();
                    }
                    None => {}
                }
                
            }
        }
    }
}
