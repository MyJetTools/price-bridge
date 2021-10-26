use std::{collections::HashMap, sync::Arc, time::Duration};

use chrono::{DateTime, NaiveDateTime, Utc};
use substring::Substring;
use tokio::{sync::mpsc::UnboundedReceiver};

use crate::{BidAsk, MetricsStore, SessionList};

const MESSAGE_SPLITTER: [u8; 2] = [13, 10];

fn parse_timestamp_to_date(timestamp: String) -> String {
    let nanoseconds = timestamp.substring(10, 14).parse::<u32>().unwrap() * 1000000;
    let timestamp = timestamp.substring(0, 10).parse::<i64>().unwrap();
    let datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp, nanoseconds), Utc);
    return datetime.format("%Y%m%d%H%M%S%3f").to_string();
}

pub async fn handle_and_process_bidask(
    mut rx: UnboundedReceiver<BidAsk>,
    sessions: Arc<SessionList>,
    id_mapping: HashMap<String, String>,
    metrics: Arc<MetricsStore>,
    lp: String
) {
    loop {
        let line = rx.recv().await;
        if let Some(event) = line {
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
            metrics.handle_send_data_to_tcp_clients(new_id.unwrap(), &lp);   
            
        } else {
            println!("Some how we did not get the log line");
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
