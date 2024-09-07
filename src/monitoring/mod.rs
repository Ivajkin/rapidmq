use prometheus::{Registry, Counter, Gauge, Histogram};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref MESSAGE_COUNT: Counter = Counter::new("rapidmq_messages_total", "Total number of messages").expect("metric can be created");
    pub static ref QUEUE_SIZE: Gauge = Gauge::new("rapidmq_queue_size", "Current queue size").expect("metric can be created");
    pub static ref MESSAGE_PROCESSING_TIME: Histogram = Histogram::new("rapidmq_message_processing_seconds", "Message processing time in seconds").expect("metric can be created");
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(MESSAGE_COUNT.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(QUEUE_SIZE.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(MESSAGE_PROCESSING_TIME.clone())).expect("collector can be registered");
}