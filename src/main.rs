use std::{collections::HashMap, sync::Arc, time::Duration};

use binance_quote_bridge::{BidAsk, BinanceExchangeContext, EventTcpServer, ExchangeWebscoket, Settings};
use tokio::{fs, sync::{RwLock, mpsc::UnboundedReceiver}};

#[tokio::main]
async fn main() {

    let settings = parse_settings().await;

    let instruments = settings.instruments_mapping.keys().cloned().collect::<Vec<String>>();


    let binance_context = BinanceExchangeContext::new(instruments);
    let mut socket = ExchangeWebscoket::new(binance_context);

    let tcp_server = EventTcpServer::new();

    let server = Arc::new(tcp_server);

    let handler = socket.get_subscribe();
    tokio::spawn(handle_event(handler, server.clone(), settings.instruments_mapping));
    tokio::spawn(async {
        socket.start().await;
    });

    tokio::spawn(start_tcp_server(server));

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn start_tcp_server(server: Arc<EventTcpServer>){
    server.start().await;
}

async fn handle_event(mut rx: UnboundedReceiver<BidAsk>, server: Arc<EventTcpServer>, id_mapping: HashMap<String, String>){
    loop {
        let line = rx.recv().await;
        if let Some(event) = line {

            let new_id = id_mapping.get(&event.id);
            
            if new_id.is_none() {
                println!("Not found id: {}. ", &event.id);
                continue;
            }

            let str = format!("{} {} {} {}", new_id.unwrap(), event.bid, event.ask, event.date);
            server.send_event_to_all_sockets(str.as_bytes().to_vec()).await
            
        } else {
            println!("Some how we did not get the log line");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

async fn parse_settings() -> Settings{
    let content = fs::read_to_string("./settings.json").await.unwrap();
    let parsed_json : Settings = serde_json::from_str(&content).unwrap();
    return parsed_json;
}