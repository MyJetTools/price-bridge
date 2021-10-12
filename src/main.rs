use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use binance_quote_bridge::{BaseContext, BidAsk, BinanceExchangeContext, ExchangeWebscoket, SessionList, Settings, start};
use chrono::{DateTime, NaiveDateTime, Utc};
use tokio::{fs, sync::{mpsc::UnboundedReceiver}};

const MESSAGE_SPLITTER: [u8; 2] = [13, 10];

#[tokio::main]
async fn main() {
    let settings = parse_settings().await;
    let server_sessions_list = Arc::new(SessionList::new());
    let mut socket = ExchangeWebscoket::new(BinanceExchangeContext::new(
        settings
            .instruments_mapping
            .keys()
            .cloned()
            .collect::<Vec<String>>(),
    ));

    let handler = socket.get_subscribe();
    
    tokio::spawn(start(SocketAddr::from(([0, 0, 0, 0], 8080)), server_sessions_list.clone()));
    tokio::spawn(handle_event(handler, server_sessions_list.clone(), settings.instruments_mapping));
    tokio::spawn(socket.start());

    // tokio::spawn(start_tcp_server(server));

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}


// async fn start_tcp_server(server: Arc<EventTcpServer>) {
//     server.start().await;
// }

async fn handle_event(mut rx: UnboundedReceiver<BidAsk>, sessions: Arc<SessionList>, id_mapping: HashMap<String, String>){
    loop {
        let line = rx.recv().await;
        if let Some(event) = line {
            let new_id = id_mapping.get(&event.id);

            if new_id.is_none() {
                println!("Not found id: {}. ", &event.id);
                continue;
            }

            let date = BinanceExchangeContext::parse_date(event.date.to_string());

            let str = format!(
                "{} {} {} {}",
                new_id.unwrap(),
                event.bid,
                event.ask,
                date
            );

            let mut data_to_send = str.as_bytes().to_vec();
            data_to_send.extend_from_slice(&MESSAGE_SPLITTER);
            sessions.send_to_all(data_to_send).await;
        } else {
            println!("Some how we did not get the log line");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

async fn parse_settings() -> Settings {
    let content = fs::read_to_string("./settings.json").await.unwrap();
    let parsed_json: Settings = serde_json::from_str(&content).unwrap();
    return parsed_json;
}
