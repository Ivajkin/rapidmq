use lazy_static::lazy_static;
use prometheus::{IntCounter, IntGauge, Registry};

lazy_static! {
    pub static ref REGISTRY: Registry = Registry::new();
    pub static ref MESSAGES_PUBLISHED: IntCounter = IntCounter::new("messages_published", "Number of messages published").expect("metric can be created");
    pub static ref MESSAGES_CONSUMED: IntCounter = IntCounter::new("messages_consumed", "Number of messages consumed").expect("metric can be created");
    pub static ref QUEUE_COUNT: IntGauge = IntGauge::new("queue_count", "Number of queues").expect("metric can be created");
    pub static ref TOTAL_MESSAGES: IntGauge = IntGauge::new("total_messages", "Total number of messages in all queues").expect("metric can be created");
}

pub fn register_metrics() {
    REGISTRY.register(Box::new(MESSAGES_PUBLISHED.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(MESSAGES_CONSUMED.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(QUEUE_COUNT.clone())).expect("collector can be registered");
    REGISTRY.register(Box::new(TOTAL_MESSAGES.clone())).expect("collector can be registered");
}