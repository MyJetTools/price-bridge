use std::{net::SocketAddr, sync::Arc, time::Duration};

use binance_quote_bridge::{
    handle_and_process_bidask, http_start, start, BinanceExchangeContext, ExchangeWebscoket,
    KrakenExchangeContext,
    FtxExchangeContext, MetricsStore, SessionList, Settings,
};

#[tokio::main]
async fn main() {
    let metrics = Arc::new(MetricsStore::new());
    let settings = Settings::from_file("./settings.json".into()).await;
    let server_sessions_list = Arc::new(SessionList::new());

    tokio::spawn(start(
        SocketAddr::from(([0, 0, 0, 0], 8080)),
        server_sessions_list.clone(),
        metrics.clone(),
    ));
    tokio::spawn(http_start(
        SocketAddr::from(([0, 0, 0, 0], 8081)),
        metrics.clone(),
    ));

    match settings.target_exchange.as_str() {
        "ftx" => {
            let mut binance_socket =
                ExchangeWebscoket::new(FtxExchangeContext::new_by_settings(&settings));
            let handler = binance_socket.get_subscribe();

            tokio::spawn(handle_and_process_bidask(
                handler,
                server_sessions_list.clone(),
                settings.instruments_mapping,
                metrics.clone(),
                "ftx".into(),
            ));
            tokio::spawn(binance_socket.start(metrics.clone()));
        }
        "binance" => {
            let mut ftx_socket =
                ExchangeWebscoket::new(BinanceExchangeContext::new_by_settings(&settings));
            let handler = ftx_socket.get_subscribe();

            tokio::spawn(handle_and_process_bidask(
                handler,
                server_sessions_list.clone(),
                settings.instruments_mapping,
                metrics.clone(),
                "ftx".into(),
            ));

            tokio::spawn(ftx_socket.start(metrics.clone()));
        }
        "kraken" => {
            let mut kraken_socket =
                ExchangeWebscoket::new(KrakenExchangeContext::new_by_settings(&settings));
            let handler = kraken_socket.get_subscribe();

            tokio::spawn(handle_and_process_bidask(
                handler,
                server_sessions_list.clone(),
                settings.instruments_mapping,
                metrics.clone(),
                "kraken".into(),
            ));

            tokio::spawn(kraken_socket.start(metrics.clone()));
        }
        _ => {
            panic!("Exchabge is not supported");
        }
    };

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
