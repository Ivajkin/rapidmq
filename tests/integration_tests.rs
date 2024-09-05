use rapidmq::{RapidMQ, Message};
use raft::NodeId;
use tokio::runtime::Runtime;
use std::sync::Arc;
use std::time::Duration;

fn setup_cluster(node_count: usize) -> Vec<Arc<RapidMQ>> {
    let node_ids: Vec<NodeId> = (1..=node_count).map(NodeId::from).collect();
    node_ids.iter().map(|&id| {
        let peers: Vec<NodeId> = node_ids.iter().filter(|&&n| n != id).cloned().collect();
        Arc::new(RapidMQ::new(id, peers))
    }).collect()
}

async fn wait_for_cluster_sync(nodes: &[Arc<RapidMQ>], duration: Duration) {
    tokio::time::sleep(duration).await;
}

#[test]
fn test_cluster_queue_creation() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let nodes = setup_cluster(3);
        
        nodes[0].create_queue("test_queue");
        
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;
        
        for node in &nodes {
            assert!(node.cluster_manager.get_state().queue_assignments.contains_key("test_queue"));
        }
    });
}

#[test]
fn test_cluster_publish_consume() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let nodes = setup_cluster(3);
        
        nodes[0].create_queue("test_queue");
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;
        
        let message = Message {
            id: "1".to_string(),
            content: "Test cluster message".to_string(),
        };
        
        nodes[0].publish("test_queue", message.clone()).await;
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;
        
        for node in &nodes {
            let consumed = node.consume("test_queue").await.unwrap();
            assert_eq!(consumed.id, message.id);
            assert_eq!(consumed.content, message.content);
        }
    });
}

#[test]
fn test_cluster_node_addition() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut nodes = setup_cluster(3);
        
        let new_node_id = NodeId::from(4);
        let new_node = Arc::new(RapidMQ::new(new_node_id, nodes.iter().map(|n| n.cluster_manager.node.lock().unwrap().id()).collect()));
        
        for node in &nodes {
            node.add_node(new_node_id, format!("127.0.0.1:{}", 50000 + new_node_id.0));
        }
        
        nodes.push(new_node.clone());
        
        wait_for_cluster_sync(&nodes, Duration::from_secs(2)).await;
        
        for node in &nodes {
            assert!(node.cluster_manager.get_state().nodes.contains_key(&new_node_id));
        }

        // Test that the new node can participate in message passing
        new_node.create_queue("new_node_queue");
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;

        let message = Message {
            id: "2".to_string(),
            content: "Message from new node".to_string(),
        };
        new_node.publish("new_node_queue", message.clone()).await;
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;

        let consumed = nodes[0].consume("new_node_queue").await.unwrap();
        assert_eq!(consumed.id, message.id);
        assert_eq!(consumed.content, message.content);
    });
}

#[test]
fn test_cluster_node_removal() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut nodes = setup_cluster(4);
        let node_to_remove = nodes.pop().unwrap();
        let remove_id = node_to_remove.cluster_manager.node.lock().unwrap().id();

        for node in &nodes {
            node.remove_node(remove_id);
        }

        wait_for_cluster_sync(&nodes, Duration::from_secs(2)).await;

        for node in &nodes {
            assert!(!node.cluster_manager.get_state().nodes.contains_key(&remove_id));
        }

        // Test that the cluster still functions after node removal
        nodes[0].create_queue("post_removal_queue");
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;

        let message = Message {
            id: "3".to_string(),
            content: "Post-removal message".to_string(),
        };
        nodes[0].publish("post_removal_queue", message.clone()).await;
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;

        let consumed = nodes[1].consume("post_removal_queue").await.unwrap();
        assert_eq!(consumed.id, message.id);
        assert_eq!(consumed.content, message.content);
    });
}

#[test]
fn test_cluster_load_balancing() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let nodes = setup_cluster(3);
        let queue_count = 10;

        for i in 0..queue_count {
            nodes[0].create_queue(&format!("queue_{}", i));
        }

        wait_for_cluster_sync(&nodes, Duration::from_secs(2)).await;

        let state = nodes[0].cluster_manager.get_state();
        let mut node_loads = vec![0; nodes.len()];

        for (_, &node_id) in state.queue_assignments.iter() {
            let index = nodes.iter().position(|n| n.cluster_manager.node.lock().unwrap().id() == node_id).unwrap();
            node_loads[index] += 1;
        }

        // Check that the load is relatively balanced
        let max_load = *node_loads.iter().max().unwrap();
        let min_load = *node_loads.iter().min().unwrap();
        assert!(max_load - min_load <= 1, "Load is not balanced: {:?}", node_loads);
    });
}

#[test]
fn test_cluster_fault_tolerance() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut nodes = setup_cluster(5);
        
        // Simulate two nodes going down
        let offline_nodes = nodes.split_off(3);
        
        nodes[0].create_queue("fault_tolerant_queue");
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;

        let message = Message {
            id: "4".to_string(),
            content: "Fault tolerance test".to_string(),
        };
        nodes[0].publish("fault_tolerant_queue", message.clone()).await;
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;

        // Check that the remaining nodes can still function
        for node in &nodes {
            let consumed = node.consume("fault_tolerant_queue").await.unwrap();
            assert_eq!(consumed.id, message.id);
            assert_eq!(consumed.content, message.content);
        }

        // Bring the offline nodes back
        nodes.extend(offline_nodes);
        wait_for_cluster_sync(&nodes, Duration::from_secs(2)).await;

        // Check that the previously offline nodes can now participate
        let new_message = Message {
            id: "5".to_string(),
            content: "Post-recovery message".to_string(),
        };
        nodes[3].publish("fault_tolerant_queue", new_message.clone()).await;
        wait_for_cluster_sync(&nodes, Duration::from_secs(1)).await;

        let consumed = nodes[4].consume("fault_tolerant_queue").await.unwrap();
        assert_eq!(consumed.id, new_message.id);
        assert_eq!(consumed.content, new_message.content);
    });
}

#[test]
fn test_publish_consume() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let node_id = NodeId::from(1);
        let peers = vec![NodeId::from(2), NodeId::from(3)];
        let rapidmq = RapidMQ::new(node_id, peers);

        rapidmq.create_queue("test_queue");

        let message = Message {
            id: "1".to_string(),
            content: "Test message".to_string(),
        };

        rapidmq.publish("test_queue", message.clone()).await;

        let consumed = rapidmq.consume("test_queue").await.unwrap();
        assert_eq!(consumed.id, message.id);
        assert_eq!(consumed.content, message.content);
    });
}

#[test]
fn test_multiple_queues() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let node_id = NodeId::from(1);
        let peers = vec![NodeId::from(2), NodeId::from(3)];
        let rapidmq = RapidMQ::new(node_id, peers);

        rapidmq.create_queue("queue1");
        rapidmq.create_queue("queue2");

        let message1 = Message {
            id: "1".to_string(),
            content: "Message 1".to_string(),
        };
        let message2 = Message {
            id: "2".to_string(),
            content: "Message 2".to_string(),
        };

        rapidmq.publish("queue1", message1.clone()).await;
        rapidmq.publish("queue2", message2.clone()).await;

        let consumed1 = rapidmq.consume("queue1").await.unwrap();
        let consumed2 = rapidmq.consume("queue2").await.unwrap();

        assert_eq!(consumed1.content, "Message 1");
        assert_eq!(consumed2.content, "Message 2");
    });
}