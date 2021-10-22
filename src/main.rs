use binance_quote_bridge::{
    http_start, start, BaseContext, BidAsk, BinanceExchangeContext, ExchangeWebscoket,
    FtxExchangeContext, KrakenExchangeContext, Metrics, SessionList, Settings,
};
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use stopwatch::Stopwatch;
use substring::Substring;
use tokio::{fs, sync::mpsc::UnboundedReceiver};

const MESSAGE_SPLITTER: [u8; 2] = [13, 10];

#[tokio::main]
async fn main() {
    let metrics = Arc::new(Metrics::new());
    let settings = parse_settings().await;
    let server_sessions_list = Arc::new(SessionList::new());
    let mut socket = ExchangeWebscoket::new(KrakenExchangeContext::new(
        settings
            .instruments_mapping
            .keys()
            .cloned()
            .collect::<Vec<String>>(),
    ));

    match settings.target_exchange.as_str() {
        "ftx" => {
            let mut binance_socket =
                ExchangeWebscoket::new(FtxExchangeContext::new_by_settings(&settings));

            let handler = binance_socket.get_subscribe();

            tokio::spawn(start(
                SocketAddr::from(([0, 0, 0, 0], 8080)),
                server_sessions_list.clone(),
                metrics.clone(),
            ));
            tokio::spawn(http_start(
                SocketAddr::from(([0, 0, 0, 0], 8081)),
                metrics.clone(),
            ));
            tokio::spawn(handle_event(
                handler,
                server_sessions_list.clone(),
                settings.instruments_mapping,
                metrics.clone(),
            ));
            tokio::spawn(binance_socket.start(metrics.clone()));
        }
        "binance" => {
            let mut ftx_socket =
                ExchangeWebscoket::new(BinanceExchangeContext::new_by_settings(&settings));

            let handler = ftx_socket.get_subscribe();

            tokio::spawn(start(
                SocketAddr::from(([0, 0, 0, 0], 8080)),
                server_sessions_list.clone(),
                metrics.clone(),
            ));
            tokio::spawn(http_start(
                SocketAddr::from(([0, 0, 0, 0], 8081)),
                metrics.clone(),
            ));
            tokio::spawn(handle_event(
                handler,
                server_sessions_list.clone(),
                settings.instruments_mapping,
                metrics.clone(),
            ));
            tokio::spawn(ftx_socket.start(metrics.clone()));
        }
        "kraken" => {
            let mut kraken_socket =
                ExchangeWebscoket::new(KrakenExchangeContext::new_by_settings(&settings));

            let handler = kraken_socket.get_subscribe();

            tokio::spawn(start(
                SocketAddr::from(([0, 0, 0, 0], 8080)),
                server_sessions_list.clone(),
                metrics.clone(),
            ));
            tokio::spawn(http_start(
                SocketAddr::from(([0, 0, 0, 0], 8081)),
                metrics.clone(),
            ));
            tokio::spawn(handle_event(
                handler,
                server_sessions_list.clone(),
                settings.instruments_mapping,
                metrics.clone(),
            ));
            tokio::spawn(kraken_socket.start(metrics.clone()));
        }
        _ => {
            panic!("Not supported exchange");
        }
    };

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

async fn handle_event(
    mut rx: UnboundedReceiver<BidAsk>,
    sessions: Arc<SessionList>,
    id_mapping: HashMap<String, String>,
    metrics: Arc<Metrics>,
) {
    loop {
        let line = rx.recv().await;
        if let Some(event) = line {
            let sw = Stopwatch::start_new();
            let new_id = id_mapping.get(&event.id);

            if new_id.is_none() {
                println!("Not found id: {}. ", &event.id);
                continue;
            }

            let date = parse_timestamp_to_date(event.date.to_string());

            let str = format!("{} {} {} {}", new_id.unwrap(), event.bid, event.ask, date);

            let mut data_to_send = str.as_bytes().to_vec();
            data_to_send.extend_from_slice(&MESSAGE_SPLITTER);
            sessions.send_to_all(data_to_send).await;
            metrics
                .quote_process_time_sm
                .with_label_values(&[&new_id.unwrap()])
                .inc_by(sw.elapsed_ms() as f64);
            metrics
                .quote_process_time_sm_count
                .with_label_values(&[&new_id.unwrap()])
                .inc();
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

fn parse_timestamp_to_date(timestamp: String) -> String {
    let nanoseconds = timestamp.substring(10, 14).parse::<u32>().unwrap() * 1000000;
    let timestamp = timestamp.substring(0, 10).parse::<i64>().unwrap();
    let datetime =
        DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, nanoseconds), Utc);
    return datetime.format("%Y%m%d%H%M%S%3f").to_string();
}
