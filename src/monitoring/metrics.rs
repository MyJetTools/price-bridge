use prometheus::{CounterVec, Encoder, Gauge, GaugeVec, Opts, Registry, TextEncoder};

pub struct Metrics {
    registry: Registry,
    pub quote_income: CounterVec,
    pub quote_process_time_sm: CounterVec,
    pub quote_process_time_sm_count: CounterVec,
    pub quote_process_time_sm_ws: CounterVec,
    pub is_connected: Gauge,
    pub tcp_server_clients_count: GaugeVec,
    pub quote_process_time_ws_sm_count: CounterVec,
}

impl Metrics {

    pub fn new() -> Metrics{

        let registry = Registry::new();

        let average_income = CounterVec::new(Opts::new("average_income_to_socket", "amount of income messages"), &["instrument"]).unwrap();
        let is_socket_connected = Gauge::with_opts(Opts::new("is_connected_into_socket", "is_connected_onto_socket")).unwrap();
        let amount_of_socket_clients = GaugeVec::new(Opts::new("amount_of_connected_clients", "amount of connected clients into tcp server"), &["id"]).unwrap();
        let quote_process_time_sm = CounterVec::new(Opts::new("average_quote_process_time", "amount of connected clients into tcp server"), &["instrument"]).unwrap();
        let quote_process_time_sm_count = CounterVec::new(Opts::new("average_quote_process_time_count", "amount of connected clients into tcp server"), &["instrument"]).unwrap();
        let quote_process_time_ws_sm = CounterVec::new(Opts::new("average_quote_ws_process_time", "amount of connected clients into tcp server"), &["instrument"]).unwrap();
        let quote_process_time_ws_sm_count = CounterVec::new(Opts::new("average_quote_ws_process_time_count", "amount of connected clients into tcp server"), &["instrument"]).unwrap();

        registry.register(Box::new(average_income.clone())).unwrap();
        registry.register(Box::new(is_socket_connected.clone())).unwrap();
        registry.register(Box::new(amount_of_socket_clients.clone())).unwrap();
        registry.register(Box::new(quote_process_time_sm.clone())).unwrap();
        registry.register(Box::new(quote_process_time_sm_count.clone())).unwrap();
        registry.register(Box::new(quote_process_time_ws_sm.clone())).unwrap();
        registry.register(Box::new(quote_process_time_ws_sm_count.clone())).unwrap();

        Metrics{
            registry: registry,
            quote_income: average_income,
            is_connected: is_socket_connected,
            tcp_server_clients_count: amount_of_socket_clients,
            quote_process_time_sm: quote_process_time_sm,     
            quote_process_time_sm_count: quote_process_time_sm_count,     
            quote_process_time_sm_ws: quote_process_time_ws_sm,     
            quote_process_time_ws_sm_count: quote_process_time_ws_sm_count,     
        }
    }

    pub fn get_data(&self) -> String {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        return String::from_utf8(buffer).unwrap(); 
    }
}
