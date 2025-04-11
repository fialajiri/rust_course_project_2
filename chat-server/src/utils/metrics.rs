use prometheus::{Counter, Gauge, Registry};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Metrics {
    pub messages_sent: Counter,
    pub active_connections: Gauge,
    registry: Registry,
}

impl Metrics {
    pub fn new() -> Arc<Mutex<Self>> {
        let registry = Registry::new();

        let messages_sent = Counter::new(
            "chat_messages_sent_total",
            "Total number of messages sent through the server",
        )
        .unwrap();

        let active_connections = Gauge::new(
            "chat_active_connections",
            "Number of active connections to the server",
        )
        .unwrap();

        registry.register(Box::new(messages_sent.clone())).unwrap();
        registry
            .register(Box::new(active_connections.clone()))
            .unwrap();

        Arc::new(Mutex::new(Self {
            messages_sent,
            active_connections,
            registry,
        }))
    }

    pub fn get_metrics(&self) -> String {
        self.registry
            .gather()
            .iter()
            .map(|mf| {
                prometheus::TextEncoder::new()
                    .encode_to_string(&[mf.clone()])
                    .unwrap()
            })
            .collect::<Vec<String>>()
            .join("\n")
    }
}
