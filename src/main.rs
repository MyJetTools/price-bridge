use std::{net::SocketAddr, sync::Arc, time::Duration};

use binance_quote_bridge::{BinanceExchangeContext, ExchangeWebscoket, FtxExchangeContext, MetricsStore, SessionList, Settings, handle_and_process_bidask, http_start, start};

#[tokio::main]
async fn main() {
    let metrics = Arc::new(MetricsStore::new());
    let settings = Settings::from_file("./settings.json".into()).await;
    let server_sessions_list = Arc::new(SessionList::new());

    if settings.target_exchange == "ftx" {
        let ctx = FtxExchangeContext::new_by_settings(&settings);
        let mut binance_socket =
            ExchangeWebscoket::new(ctx);
        let handler = binance_socket.get_subscribe();

        tokio::spawn(handle_and_process_bidask(
            handler,
            server_sessions_list.clone(),
            settings.instruments_mapping,
            metrics.clone(),
            "ftx".into()
        ));
        tokio::spawn(binance_socket.start(metrics.clone()));
    } else {
        let mut ftx_socket =
            ExchangeWebscoket::new(BinanceExchangeContext::new_by_settings(&settings));
        let handler = ftx_socket.get_subscribe();

        tokio::spawn(handle_and_process_bidask(
            handler,
            server_sessions_list.clone(),
            settings.instruments_mapping,
            metrics.clone(),
            "ftx".into()
        ));

        tokio::spawn(ftx_socket.start(metrics.clone()));

        tokio::spawn(start(
            SocketAddr::from(([0, 0, 0, 0], 8080)),
            server_sessions_list.clone(),
            metrics.clone(),
        ));
        tokio::spawn(http_start(
            SocketAddr::from(([0, 0, 0, 0], 8081)),
            metrics.clone(),
        ));
    }

    loop {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
