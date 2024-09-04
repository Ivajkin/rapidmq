use cucumber::{given, when, then, World};
use std::collections::HashMap;
use tokio::sync::Mutex;
use async_trait::async_trait;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rand::Rng;
use thiserror::Error;

mod mocks {
    use super::*;

    #[derive(Clone, Debug)]
    pub struct Message {
        pub topic: String,
        pub payload: String,
        pub timestamp: u64,
        pub priority: u8,
        pub expiration: Option<u64>,
    }

    impl Message {
        pub fn new(topic: String, payload: String, priority: u8, expiration: Option<u64>) -> Self {
            Self {
                topic,
                payload,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                priority,
                expiration,
            }
        }

        pub fn is_expired(&self, current_time: u64) -> bool {
            self.expiration.map_or(false, |exp| current_time > exp)
        }
    }

    pub struct EdgeNode {
        pub id: String,
        pub pending_messages: Vec<Message>,
        pub sync_interval: u64,
        pub storage_capacity: usize,
    }

    impl EdgeNode {
        pub fn new(id: String) -> Self {
            Self {
                id,
                pending_messages: Vec::new(),
                sync_interval: 60,
                storage_capacity: 1000,
            }
        }

        pub fn store_message(&mut self, message: Message) -> Result<(), &'static str> {
            if self.pending_messages.len() >= self.storage_capacity {
                Err("Storage capacity reached")
            } else {
                self.pending_messages.push(message);
                Ok(())
            }
        }

        pub fn sync_messages(&mut self, bandwidth_limit: Option<usize>) -> Vec<Message> {
            let mut synced_messages = Vec::new();
            self.pending_messages.sort_by_key(|m| std::cmp::Reverse(m.priority));
            
            let sync_count = bandwidth_limit.unwrap_or(self.pending_messages.len());
            for _ in 0..sync_count {
                if let Some(msg) = self.pending_messages.pop() {
                    synced_messages.push(msg);
                } else {
                    break;
                }
            }
            synced_messages
        }

        pub fn clean_expired_messages(&mut self, current_time: u64) {
            self.pending_messages.retain(|msg| !msg.is_expired(current_time));
        }
    }

    pub struct Broker {
        pub received_messages: Vec<Message>,
        pub is_online: bool,
    }

    impl Broker {
        pub fn new() -> Self {
            Self { received_messages: Vec::new(), is_online: true }
        }

        pub fn receive_messages(&mut self, messages: Vec<Message>) -> Result<(), &'static str> {
            if !self.is_online {
                return Err("Broker is offline");
            }
            self.received_messages.extend(messages);
            Ok(())
        }
    }
}

use mocks::*;

#[derive(Error, Debug)]
enum TestError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Broker error: {0}")]
    BrokerError(String),
}

struct Network {
    is_available: bool,
    latency: Duration,
}

struct TestClock {
    current_time: SystemTime,
}

#[derive(Debug, World)]
pub struct StoreAndForwardWorld {
    edge_node: Mutex<EdgeNode>,
    broker: Mutex<Broker>,
    network: Mutex<Network>,
    clock: Mutex<TestClock>,
}

#[async_trait]
impl cucumber::WorldInit for StoreAndForwardWorld {
    async fn new() -> Self {
        Self {
            edge_node: Mutex::new(EdgeNode::new("default".to_string())),
            broker: Mutex::new(Broker::new()),
            network: Mutex::new(Network { is_available: true, latency: Duration::from_millis(50) }),
            clock: Mutex::new(TestClock { current_time: SystemTime::now() }),
        }
    }
}

impl StoreAndForwardWorld {
    async fn simulate_network_delay(&self) -> Result<(), TestError> {
        let network = self.network.lock().await;
        if !network.is_available {
            return Err(TestError::NetworkError("Network is unavailable".to_string()));
        }
        tokio::time::sleep(network.latency).await;
        Ok(())
    }

    async fn advance_time(&mut self, duration: Duration) {
        let mut clock = self.clock.lock().await;
        clock.current_time += duration;
    }

    async fn setup_messages_with_priorities(&mut self, count: usize) -> Result<(), TestError> {
        let mut edge_node = self.edge_node.lock().await;
        for i in 1..=count {
            edge_node.store_message(Message::new(
                "test".to_string(),
                format!("data{}", i),
                (i % 5 + 1) as u8,
                None
            )).map_err(|e| TestError::StorageError(e.to_string()))?;
        }
        Ok(())
    }
}

// Step definitions

#[given(expr = "an edge node with ID {string} is configured")]
async fn configure_edge_node(w: &mut StoreAndForwardWorld, id: String) {
    let mut edge_node = w.edge_node.lock().await;
    *edge_node = EdgeNode::new(id);
}

#[given("the broker is online")]
async fn set_broker_online(w: &mut StoreAndForwardWorld) {
    let mut broker = w.broker.lock().await;
    broker.is_online = true;
}

#[given("the network is unavailable")]
#[when("the network becomes unavailable")]
async fn set_network_unavailable(w: &mut StoreAndForwardWorld) {
    let mut network = w.network.lock().await;
    network.is_available = false;
}

#[when("a producer sends the following messages:")]
async fn producer_sends_messages(w: &mut StoreAndForwardWorld, messages: Vec<HashMap<String, String>>) -> Result<(), TestError> {
    let mut edge_node = w.edge_node.lock().await;
    for msg in messages {
        let message = Message::new(
            msg["topic"].clone(),
            msg["payload"].clone(),
            1, // Default priority
            None, // No expiration
        );
        edge_node.store_message(message).map_err(|e| TestError::StorageError(e.to_string()))?;
    }
    Ok(())
}

#[then(expr = "{int} messages should be stored locally")]
async fn check_stored_messages(w: &mut StoreAndForwardWorld, count: usize) {
    let edge_node = w.edge_node.lock().await;
    assert_eq!(edge_node.pending_messages.len(), count);
}

#[when("the network becomes available")]
async fn network_becomes_available(w: &mut StoreAndForwardWorld) {
    let mut network = w.network.lock().await;
    network.is_available = true;
}

#[when("the edge node syncs with the broker")]
async fn edge_node_syncs(w: &mut StoreAndForwardWorld) -> Result<(), TestError> {
    let network = w.network.lock().await;
    if !network.is_available {
        return Err(TestError::NetworkError("Network is unavailable".to_string()));
    }
    let mut edge_node = w.edge_node.lock().await;
    let mut broker = w.broker.lock().await;
    let messages = edge_node.sync_messages(None);
    broker.receive_messages(messages).map_err(|e| TestError::BrokerError(e.to_string()))?;
    Ok(())
}

#[then("all pending messages should be sent to the broker")]
async fn messages_sent_to_broker(w: &mut StoreAndForwardWorld) {
    let edge_node = w.edge_node.lock().await;
    let broker = w.broker.lock().await;
    assert_eq!(edge_node.pending_messages.len(), 0);
    assert!(broker.received_messages.len() > 0);
}

#[then("the broker should receive the messages in the correct order")]
async fn check_message_order(w: &mut StoreAndForwardWorld) {
    let broker = w.broker.lock().await;
    let messages = &broker.received_messages;
    for i in 1..messages.len() {
        assert!(messages[i].timestamp >= messages[i-1].timestamp);
    }
}

#[given("the network becomes unstable")]
async fn set_network_unstable(w: &mut StoreAndForwardWorld) {
    // Simulate network instability
    let mut network = w.network.lock().await;
    network.is_available = rand::random();
}

#[when("the edge node attempts to sync with the broker")]
async fn attempt_sync_with_unstable_network(w: &mut StoreAndForwardWorld) -> Result<(), TestError> {
    let mut edge_node = w.edge_node.lock().await;
    let mut broker = w.broker.lock().await;
    let messages = edge_node.sync_messages(Some(rand::thread_rng().gen_range(1..=5)));
    if w.network.lock().await.is_available {
        broker.receive_messages(messages).map_err(|e| TestError::BrokerError(e.to_string()))?;
    }
    Ok(())
}

#[then(expr = "at least {int} message should be sent to the broker")]
async fn check_min_messages_sent(w: &mut StoreAndForwardWorld, min_count: usize) {
    let broker = w.broker.lock().await;
    assert!(broker.received_messages.len() >= min_count);
}

#[then("the remaining messages should still be in local storage")]
async fn check_remaining_messages(w: &mut StoreAndForwardWorld) {
    let edge_node = w.edge_node.lock().await;
    let broker = w.broker.lock().await;
    assert_eq!(edge_node.pending_messages.len() + broker.received_messages.len(), 5);
}

#[when(expr = "the sync interval is set to {int} minutes")]
async fn set_sync_interval(w: &mut StoreAndForwardWorld, interval: u64) {
    let mut edge_node = w.edge_node.lock().await;
    edge_node.sync_interval = interval * 60;
}

#[then(expr = "the edge node should attempt to sync every {int} minutes")]
async fn check_sync_interval(w: &mut StoreAndForwardWorld, expected_interval: u64) {
    let edge_node = w.edge_node.lock().await;
    assert_eq!(edge_node.sync_interval, expected_interval * 60);
}

#[given(expr = "a message with expiration time of {int} hour is stored")]
async fn store_message_with_expiration(w: &mut StoreAndForwardWorld, hours: u64) -> Result<(), TestError> {
    let mut edge_node = w.edge_node.lock().await;
    let expiration = w.clock.lock().await.current_time.duration_since(UNIX_EPOCH).unwrap().as_secs() + hours * 3600;
    let message = Message::new("test".to_string(), "data".to_string(), 1, Some(expiration));
    edge_node.store_message(message).map_err(|e| TestError::StorageError(e.to_string()))?;
    Ok(())
}

#[when(expr = "{int} hours have passed")]
async fn time_passes(w: &mut StoreAndForwardWorld, hours: u64) {
    let duration = Duration::from_secs(hours * 3600);
    w.advance_time(duration).await;
    let mut edge_node = w.edge_node.lock().await;
    let current_time = w.clock.lock().await.current_time.duration_since(UNIX_EPOCH).unwrap().as_secs();
    edge_node.clean_expired_messages(current_time);
}

#[then("the expired message should be removed from local storage")]
async fn check_expired_message_removed(w: &mut StoreAndForwardWorld) {
    let edge_node = w.edge_node.lock().await;
    assert!(edge_node.pending_messages.is_empty());
}

#[given("the local storage is near capacity")]
async fn set_storage_near_capacity(w: &mut StoreAndForwardWorld) {
    let mut edge_node = w.edge_node.lock().await;
    edge_node.storage_capacity = 10;
    for _ in 0..9 {
        edge_node.store_message(Message::new("test".to_string(), "data".to_string(), 1, None)).unwrap();
    }
}

#[when("a producer sends a large message")]
async fn send_large_message(w: &mut StoreAndForwardWorld) -> Result<(), TestError> {
    let mut edge_node = w.edge_node.lock().await;
    let large_message = Message::new("large".to_string(), "very large payload".to_string(), 1, None);
    edge_node.store_message(large_message).map_err(|e| TestError::StorageError(e.to_string()))?;
    Ok(())
}

#[then("the edge node should reject the message")]
async fn check_message_rejected(w: &mut StoreAndForwardWorld) {
    let edge_node = w.edge_node.lock().await;
    assert_eq!(edge_node.pending_messages.len(), 9);
}

#[given("the local storage contains messages with different priorities")]
async fn store_messages_with_priorities(w: &mut StoreAndForwardWorld) -> Result<(), TestError> {
    w.setup_messages_with_priorities(5).await
}

#[when("the edge node syncs with limited bandwidth")]
async fn sync_with_limited_bandwidth(w: &mut StoreAndForwardWorld) -> Result<(), TestError> {
    let mut edge_node = w.edge_node.lock().await;
    let mut broker = w.broker.lock().await;
    let messages = edge_node.sync_messages(Some(3));
    broker.receive_messages(messages).map_err(|e| TestError::BrokerError(e.to_string()))?;
    Ok(())
}

#[then("high priority messages should be synced first")]
async fn check_high_priority_synced(w: &mut StoreAndForwardWorld) {
    let broker = w.broker.lock().await;
    assert_eq!(broker.received_messages.len(), 3);
    for msg in &broker.received_messages {
        assert!(msg.priority >= 3);
    }
}

#[given(expr = "the broker fails during sync after {int} messages")]
async fn simulate_broker_failure(w: &mut StoreAndForwardWorld, successful_messages: usize) -> Result<(), TestError> {
    let mut edge_node = w.edge_node.lock().await;
    let mut broker = w.broker.lock().await;
    let messages = edge_node.sync_messages(Some(successful_messages));
    broker.receive_messages(messages).map_err(|e| TestError::BrokerError(e.to_string()))?;
    broker.is_online = false;
    Ok(())
}

#[when("the broker comes back online")]
async fn broker_comes_online(w: &mut StoreAndForwardWorld) {
    let mut broker = w.broker.lock().await;
    broker.is_online = true;
}

#[when("the edge node resumes sync")]
async fn edge_node_resumes_sync(w: &mut StoreAndForwardWorld) -> Result<(), TestError> {
    let network = w.network.lock().await;
    if !network.is_available {
        return Err(TestError::NetworkError("Network is unavailable".to_string()));
    }
    let mut edge_node = w.edge_node.lock().await;
    let mut broker = w.broker.lock().await;
    let messages = edge_node.sync_messages(None);
    broker.receive_messages(messages).map_err(|e| TestError::BrokerError(e.to_string()))?;
    Ok(())
}

#[then("all 10 messages should eventually be sent to the broker")]
async fn all_messages_sent_to_broker(w: &mut StoreAndForwardWorld) {
    let edge_node = w.edge_node.lock().await;
    let broker = w.broker.lock().await;
    assert_eq!(edge_node.pending_messages.len(), 0);
    assert_eq!(broker.received_messages.len(), 10);
}

// Main test runner
#[tokio::main]
async fn main() {
    StoreAndForwardWorld::run("tests/features/store_and_forward.feature").await;
}