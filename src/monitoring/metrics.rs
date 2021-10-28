use prometheus::{CounterVec, Encoder, Gauge, GaugeVec, Opts, Registry, TextEncoder};

use crate::BidAsk;

pub struct MetricsStore {
    registry: Registry,
    income_quotes_from_lp: CounterVec,
    // send_quotes_to_sb: CounterVec,
    // send_unfiltered_quotes_to_sb: CounterVec,
    price_of_income_bid: GaugeVec,

    price_of_income_ask: GaugeVec,

    is_connected_to_sorce: Gauge,
    tcp_clients_count: GaugeVec,
    send_to_tcp_clients: CounterVec,
    // tcp_server_process_time: CounterVec,
    // tcp_server_process_time_count: CounterVec,
}

impl MetricsStore {
    pub fn new() -> MetricsStore {
        let registry = Registry::new();

        let income_quotes_from_lp = CounterVec::new(
            Opts::new("income_quotes_from_lp", "income from lp"),
            &["lp", "instrument"],
        )
        .unwrap();
        let price_of_income_bid = GaugeVec::new(
            Opts::new("price_of_income_bid", "price_of_income_bid"),
            &["lp", "instrument"],
        )
        .unwrap();
        let price_of_income_ask = GaugeVec::new(
            Opts::new("price_of_income_ask", "price_of_income_ask"),
            &["lp", "instrument"],
        )
        .unwrap();

        let tcp_clients_count = GaugeVec::new(
            Opts::new(
                "tcp_clients_count",
                "amount of connected clients into tcp server",
            ),
            &["ip"],
        )
        .unwrap();
        let send_to_tcp_clients = CounterVec::new(
            Opts::new("send_to_tcp_clients", "send_to_tcp_clients"),
            &["lp", "instrument"],
        )
        .unwrap();
        let is_connected_to_sorce =
            Gauge::with_opts(Opts::new("is_connected_to_sorce", "is_connected_to_sorce")).unwrap();

        registry
            .register(Box::new(income_quotes_from_lp.clone()))
            .unwrap();
        registry
            .register(Box::new(price_of_income_bid.clone()))
            .unwrap();
        registry
            .register(Box::new(price_of_income_ask.clone()))
            .unwrap();
        registry
            .register(Box::new(tcp_clients_count.clone()))
            .unwrap();
        registry
            .register(Box::new(send_to_tcp_clients.clone()))
            .unwrap();
        registry
            .register(Box::new(is_connected_to_sorce.clone()))
            .unwrap();

        MetricsStore {
            registry: registry,
            income_quotes_from_lp: income_quotes_from_lp,
            price_of_income_bid: price_of_income_bid,
            price_of_income_ask: price_of_income_ask,
            tcp_clients_count: tcp_clients_count,
            send_to_tcp_clients: send_to_tcp_clients,
            is_connected_to_sorce: is_connected_to_sorce,
        }
    }

    pub fn handle_quote_income(&self, lp: &String, bidask: &BidAsk) {
        self.income_quotes_from_lp
            .with_label_values(&[lp, &bidask.id])
            .inc();

        self.price_of_income_ask
            .with_label_values(&[lp, &bidask.id])
            .set(bidask.ask);

        self.price_of_income_bid
            .with_label_values(&[lp, &bidask.id])
            .set(bidask.bid);
    }

    pub fn handle_connect_change_to_tcp(&self, is_connected: bool, connect_address: &String) {
        let set_value: f64 = match is_connected {
            true => 1.0,
            false => 0.0,
        };

        self.tcp_clients_count
            .with_label_values(&[connect_address])
            .set(set_value);
    }

    pub fn handle_change_connect_to_lp(&self, is_connected: bool, exchange: &String) {
        let set_value: f64 = match is_connected {
            true => 1.0,
            false => 0.0,
        };

        self.is_connected_to_sorce.set(set_value);
    }

    pub fn handle_send_data_to_tcp_clients(&self, instrument: &String, exchange: &String) {
        self.send_to_tcp_clients
            .with_label_values(&[exchange, instrument])
            .inc();
    }

    // pub fn handle_send_data_to_sb(&self, instrument: &String, exchange: &String) {
    //     self.send_quotes_to_sb
    //         .with_label_values(&[exchange, instrument])
    //         .inc();
    // }

    // pub fn handle_send_data_to_sb_unfiltered(&self, instrument: &String, exchange: &String) {
    //     self.send_unfiltered_quotes_to_sb
    //         .with_label_values(&[exchange, instrument])
    //         .inc();
    // }

    pub fn get_data(&self) -> String {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        return String::from_utf8(buffer).unwrap();
    }
}
