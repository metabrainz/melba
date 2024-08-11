use prometheus::{labels, push_metrics, Counter, Opts, Registry};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::spawn_blocking;

pub struct Metrics {
    pub db_poll_counter: Counter,
    pub network_request_counter: Counter,
    pub registry: Arc<Mutex<Registry>>,
}

impl Metrics {
    pub async fn new() -> Self {
        let registry = Arc::new(Mutex::new(Registry::new()));
        let poll_opts = Opts::new(
            "db_polls_total",
            "Total number of edit data and edit notes polls",
        );
        let db_poll_counter = Counter::with_opts(poll_opts).unwrap();
        registry
            .lock()
            .await
            .register(Box::new(db_poll_counter.clone()))
            .unwrap();

        let request_opts = Opts::new(
            "network_requests_total",
            "Total number of network requests made",
        );
        let network_request_counter = Counter::with_opts(request_opts).unwrap();
        registry
            .lock()
            .await
            .register(Box::new(network_request_counter.clone()))
            .unwrap();
        Metrics {
            db_poll_counter,
            network_request_counter,
            registry,
        }
    }
    pub async fn push_metrics(&self) {
        let registry = self.registry.clone();
        spawn_blocking(move || {
            push_metrics(
                "mb-ia-archiver",
                labels! {"pushgateway".to_string() => "rust".to_string()},
                "pushgateway:9091",
                registry.blocking_lock().gather(),
                None,
            )
            .unwrap_or_default();
        })
        .await
        .unwrap();
    }
}
