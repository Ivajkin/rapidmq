use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use raft::prelude::*;
use rocksdb::{DB, Options};
use serde::{Serialize, Deserialize};
use raft::NodeId;
use prost::Message as ProstMessage;

// Add this at the top of the file
pub mod metrics;

// Include the generated protobuf code
pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/rapidmq.rs"));
}

use proto::RapidMQMessage;

// Message struct to represent individual messages
#[derive(Clone, Debug)]
pub struct Message {
    pub id: String,
    pub content: String,
}

impl From<Message> for RapidMQMessage {
    fn from(msg: Message) -> Self {
        RapidMQMessage {
            id: msg.id,
            content: msg.content,
        }
    }
}

impl From<RapidMQMessage> for Message {
    fn from(msg: RapidMQMessage) -> Self {
        Message {
            id: msg.id,
            content: msg.content,
        }
    }
}

// Queue struct to manage message queues
pub struct Queue {
    messages: VecDeque<Vec<u8>>,
    db: Arc<DB>,
    name: String,
}

impl Queue {
    pub fn new(name: &str, db: Arc<DB>) -> Self {
        let messages = Queue::load_messages(name, &db);
        Queue {
            messages,
            db,
            name: name.to_string(),
        }
    }

    pub fn enqueue(&mut self, message: Message) {
        let proto_message: RapidMQMessage = message.into();
        let encoded = proto_message.encode_to_vec();
        self.messages.push_back(encoded);
        self.persist_message(&encoded);
    }

    pub fn dequeue(&mut self) -> Option<Message> {
        self.messages.pop_front().and_then(|encoded| {
            RapidMQMessage::decode(&encoded[..])
                .ok()
                .map(|proto_message| proto_message.into())
        })
    }

    fn persist_message(&self, encoded: &[u8]) {
        let key = format!("{}:{}", self.name, uuid::Uuid::new_v4());
        self.db.put(key.as_bytes(), encoded).unwrap();
    }

    fn load_messages(name: &str, db: &DB) -> VecDeque<Vec<u8>> {
        let mut messages = VecDeque::new();
        let prefix = format!("{}:", name);
        let iter = db.iterator(rocksdb::IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward));
        for (_, value) in iter {
            messages.push_back(value.to_vec());
        }
        messages
    }
}

// RapidMQ struct to manage the overall messaging system
#[derive(Clone)]
pub struct RapidMQ {
    queues: Arc<Mutex<HashMap<String, Queue>>>,
    subscribers: Arc<Mutex<HashMap<String, Vec<String>>>>,
    db: Arc<DB>,
    cluster_manager: Arc<ClusterManager>,
}

impl RapidMQ {
    pub fn new(node_id: NodeId, peers: Vec<NodeId>) -> Self {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = Arc::new(DB::open(&opts, format!("rapidmq_storage_{}", node_id)).unwrap());

        metrics::register_metrics();

        let cluster_manager = Arc::new(ClusterManager::new(node_id, peers));

        RapidMQ {
            queues: Arc::new(Mutex::new(HashMap::new())),
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            db,
            cluster_manager,
        }
    }

    pub fn create_queue(&self, queue_name: &str) {
        let node_id = self.cluster_manager.assign_queue(queue_name);
        if node_id == self.cluster_manager.node.lock().id() {
            let mut queues = self.queues.lock().unwrap();
            queues.entry(queue_name.to_string()).or_insert_with(|| Queue::new(queue_name, self.db.clone()));
        }
        metrics::QUEUE_COUNT.inc();
    }

    pub async fn publish(&self, queue_name: &str, message: Message) {
        if let Some(node_id) = self.cluster_manager.get_queue_node(queue_name) {
            if node_id == self.cluster_manager.node.lock().id() {
                let mut queues = self.queues.lock().unwrap();
                if let Some(queue) = queues.get_mut(queue_name) {
                    queue.enqueue(message.clone());
                }

                let subscribers = self.subscribers.lock().unwrap();
                if let Some(subs) = subscribers.get(queue_name) {
                    for subscriber in subs {
                        if let Some(sub_queue) = queues.get_mut(subscriber) {
                            sub_queue.enqueue(message.clone());
                        }
                    }
                }
            } else {
                // Forward the message to the appropriate node
                if let Err(e) = self.cluster_manager.publish_remote(node_id, queue_name, message).await {
                    eprintln!("Failed to publish message to remote node: {}", e);
                }
            }
        }
        metrics::MESSAGES_PUBLISHED.inc();
        metrics::TOTAL_MESSAGES.inc();
    }

    pub async fn consume(&self, queue_name: &str) -> Option<Message> {
        if let Some(node_id) = self.cluster_manager.get_queue_node(queue_name) {
            if node_id == self.cluster_manager.node.lock().id() {
                let mut queues = self.queues.lock().unwrap();
                let message = queues.get_mut(queue_name).and_then(|queue| queue.dequeue());
                if message.is_some() {
                    metrics::MESSAGES_CONSUMED.inc();
                    metrics::TOTAL_MESSAGES.dec();
                }
                message
            } else {
                // Forward the consume request to the appropriate node
                match self.cluster_manager.consume_remote(node_id, queue_name).await {
                    Ok(message) => {
                        if message.is_some() {
                            metrics::MESSAGES_CONSUMED.inc();
                            metrics::TOTAL_MESSAGES.dec();
                        }
                        message
                    }
                    Err(e) => {
                        eprintln!("Failed to consume message from remote node: {}", e);
                        None
                    }
                }
            }
        } else {
            None
        }
    }

    pub fn subscribe(&self, queue_name: &str, subscriber_queue: &str) {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers
            .entry(queue_name.to_string())
            .or_insert_with(Vec::new)
            .push(subscriber_queue.to_string());
    }

    pub async fn run(&self) {
        self.cluster_manager.run().await;
    }

    pub fn add_node(&self, node_id: NodeId, address: String) {
        self.cluster_manager.add_node(node_id, address);
    }

    pub fn remove_node(&self, node_id: NodeId) {
        self.cluster_manager.remove_node(node_id);
    }

    pub async fn adaptive_publish(&self, queue_name: &str, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        let priority = self.cluster_manager.ai_module.predict_message_priority(&message.content).await?;
        let node_id = self.cluster_manager.assign_queue(queue_name).await;
        
        if priority > 0.8 {
            // High priority message, use quantum-optimized routing
            let optimized_route = self.cluster_manager.quantum_module.optimize_routing(vec![node_id.0]);
            let target_node = NodeId::from(optimized_route[0]);
            self.cluster_manager.publish_remote(target_node, queue_name, message).await?;
        } else {
            // Normal priority, use standard routing
            self.publish(queue_name, message).await?;
        }
        
        Ok(())
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::runtime::Runtime;

    fn setup() -> (RapidMQ, NodeId) {
        let node_id = NodeId::from(1);
        let peers = vec![NodeId::from(2), NodeId::from(3)];
        (RapidMQ::new(node_id, peers), node_id)
    }

    #[test]
    fn test_create_queue() {
        let (mq, _) = setup();
        mq.create_queue("test_queue");
        assert!(mq.queues.lock().unwrap().contains_key("test_queue"));
        assert_eq!(metrics::QUEUE_COUNT.get(), 1);
    }

    #[test]
    fn test_publish_consume() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (mq, _) = setup();
            mq.create_queue("test_queue");

            let message = Message {
                id: "1".to_string(),
                content: "Test message".to_string(),
            };

            mq.publish("test_queue", message.clone()).await;
            assert_eq!(metrics::MESSAGES_PUBLISHED.get(), 1);
            assert_eq!(metrics::TOTAL_MESSAGES.get(), 1);

            let consumed = mq.consume("test_queue").await.unwrap();
            assert_eq!(consumed.id, message.id);
            assert_eq!(consumed.content, message.content);
            assert_eq!(metrics::MESSAGES_CONSUMED.get(), 1);
            assert_eq!(metrics::TOTAL_MESSAGES.get(), 0);
        });
    }

    #[test]
    fn test_subscribe() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let (mq, _) = setup();
            mq.create_queue("main_queue");
            mq.create_queue("subscriber_queue");

            mq.subscribe("main_queue", "subscriber_queue");

            let message = Message {
                id: "1".to_string(),
                content: "Test message".to_string(),
            };

            mq.publish("main_queue", message.clone()).await;

            let consumed_main = mq.consume("main_queue").await.unwrap();
            let consumed_sub = mq.consume("subscriber_queue").await.unwrap();

            assert_eq!(consumed_main.id, message.id);
            assert_eq!(consumed_sub.id, message.id);
            assert_eq!(metrics::MESSAGES_PUBLISHED.get(), 1);
            assert_eq!(metrics::MESSAGES_CONSUMED.get(), 2);
        });
    }

    #[test]
    fn test_add_remove_node() {
        let (mq, _) = setup();
        let new_node_id = NodeId::from(4);
        mq.add_node(new_node_id, "127.0.0.1:50004".to_string());
        
        let state = mq.cluster_manager.get_state();
        assert!(state.nodes.contains_key(&new_node_id));

        mq.remove_node(new_node_id);
        let state = mq.cluster_manager.get_state();
        assert!(!state.nodes.contains_key(&new_node_id));
    }
}

// Add this at the top of the file
pub mod api;
pub mod cluster;

use cluster::ClusterManager;

// Add new modules
pub mod ai_module;
pub mod quantum_module;

use ai_module::AIModule;
use quantum_module::QuantumModule;

//! RapidMQ: A high-performance, AI-enhanced message queue system
//!
//! This library provides a robust, scalable message queue implementation
//! with advanced features like AI-powered message prioritization and
//! quantum-inspired routing optimization.
//!
//! # Quick Start
//!
//! ```rust
//! use rapidmq::RapidMQClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = RapidMQClient::new("config/rapidmq.yaml")?;
//!     
//!     client.publish("my-topic", "Hello, RapidMQ!").await?;
//!     
//!     let mut subscription = client.subscribe("my-topic").await?;
//!     while let Some(message) = subscription.next_message().await {
//!         println!("Received: {:?}", message);
//!     }
//!     
//!     Ok(())
//! }
//! ```

// ... rest of the file ...