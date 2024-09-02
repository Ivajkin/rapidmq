use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use rocksdb::{DB, Options};
use serde::{Serialize, Deserialize};
use raft::NodeId;

// Add this at the top of the file
pub mod metrics;

// Message struct to represent individual messages
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub content: String,
}

// Queue struct to manage message queues
pub struct Queue {
    messages: VecDeque<Message>,
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
        self.messages.push_back(message.clone());
        self.persist_message(&message);
    }

    pub fn dequeue(&mut self) -> Option<Message> {
        let message = self.messages.pop_front();
        if let Some(ref msg) = message {
            self.remove_persisted_message(&msg.id);
        }
        message
    }

    fn persist_message(&self, message: &Message) {
        let key = format!("{}:{}", self.name, message.id);
        let value = serde_json::to_string(message).unwrap();
        self.db.put(key.as_bytes(), value.as_bytes()).unwrap();
    }

    fn remove_persisted_message(&self, message_id: &str) {
        let key = format!("{}:{}", self.name, message_id);
        self.db.delete(key.as_bytes()).unwrap();
    }

    fn load_messages(name: &str, db: &DB) -> VecDeque<Message> {
        let mut messages = VecDeque::new();
        let prefix = format!("{}:", name);
        let iter = db.iterator(rocksdb::IteratorMode::From(prefix.as_bytes(), rocksdb::Direction::Forward));
        for (key, value) in iter {
            if let (Ok(key), Ok(value)) = (String::from_utf8(key.to_vec()), String::from_utf8(value.to_vec())) {
                if key.starts_with(&prefix) {
                    if let Ok(message) = serde_json::from_str(&value) {
                        messages.push_back(message);
                    }
                } else {
                    break;
                }
            }
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
        if node_id == self.cluster_manager.node.lock().unwrap().id() {
            let mut queues = self.queues.lock().unwrap();
            queues.entry(queue_name.to_string()).or_insert_with(|| Queue::new(queue_name, self.db.clone()));
        }
        metrics::QUEUE_COUNT.inc();
    }

    pub async fn publish(&self, queue_name: &str, message: Message) {
        if let Some(node_id) = self.cluster_manager.get_queue_node(queue_name) {
            if node_id == self.cluster_manager.node.lock().unwrap().id() {
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
            if node_id == self.cluster_manager.node.lock().unwrap().id() {
                let mut queues = self.queues.lock().unwrap();
                queues.get_mut(queue_name).and_then(|queue| queue.dequeue())
            } else {
                // Forward the consume request to the appropriate node
                match self.cluster_manager.consume_remote(node_id, queue_name).await {
                    Ok(message) => message,
                    Err(e) => {
                        eprintln!("Failed to consume message from remote node: {}", e);
                        None
                    }
                }
            }
        } else {
            None
        }
        if let Some(message) = message {
            metrics::MESSAGES_CONSUMED.inc();
            metrics::TOTAL_MESSAGES.dec();
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
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_publish_consume() {
        let mq = RapidMQ::new();
        mq.create_queue("test_queue");

        let message = Message {
            id: "1".to_string(),
            content: "Test message".to_string(),
        };

        mq.publish("test_queue", message.clone());

        let consumed = mq.consume("test_queue").unwrap();
        assert_eq!(consumed.id, message.id);
        assert_eq!(consumed.content, message.content);
    }

    #[test]
    fn test_subscribe() {
        let mq = RapidMQ::new();
        mq.create_queue("main_queue");
        mq.create_queue("subscriber_queue");

        mq.subscribe("main_queue", "subscriber_queue");

        let message = Message {
            id: "1".to_string(),
            content: "Test message".to_string(),
        };

        mq.publish("main_queue", message.clone());

        let consumed_main = mq.consume("main_queue").unwrap();
        let consumed_sub = mq.consume("subscriber_queue").unwrap();

        assert_eq!(consumed_main.id, message.id);
        assert_eq!(consumed_sub.id, message.id);
    }
}

// Add this at the top of the file
pub mod api;
pub mod cluster;

use cluster::ClusterManager;