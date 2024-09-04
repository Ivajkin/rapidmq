use crate::edge_node::EdgeNode;
use serde_json::Value;

pub struct Producer {
    broker_url: String,
    edge_node: Option<EdgeNode>,
}

impl Producer {
    pub fn new(broker_url: String, edge_node_id: Option<String>) -> Self {
        let edge_node = edge_node_id.map(|id| EdgeNode::new(id, broker_url.clone()).unwrap());
        Producer {
            broker_url,
            edge_node,
        }
    }

    pub fn send(&self, topic: &str, message: Value) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(edge_node) = &self.edge_node {
            edge_node.store_message(topic, message)?;
        } else {
            // Existing code to send message directly to broker
            // This would involve making an HTTP request or using a Rust MQTT client
        }
        Ok(())
    }

    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(edge_node) = &self.edge_node {
            edge_node.sync_with_broker().await?;
        }
        // Existing flush logic for direct broker connection
        Ok(())
    }
}