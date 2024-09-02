use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use raft::{Config, Node, NodeId, RaftState};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use tonic::{transport::{Server, Channel}, Request, Response, Status};
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

pub mod rapidmq {
    tonic::include_proto!("rapidmq");
}

use rapidmq::{
    rapid_mq_server::{RapidMq, RapidMqServer},
    PublishRequest, PublishResponse, ConsumeRequest, ConsumeResponse, StateUpdateRequest, StateUpdateResponse,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ClusterState {
    pub nodes: HashMap<NodeId, String>,
    pub queue_assignments: HashMap<String, NodeId>,
    pub node_loads: HashMap<NodeId, usize>,
}

pub struct ClusterManager {
    node: Arc<Mutex<Node<ClusterState>>>,
    state: Arc<Mutex<ClusterState>>,
    rpc_clients: Arc<Mutex<HashMap<NodeId, rapidmq::rapid_mq_client::RapidMqClient<tonic::transport::Channel>>>>,
}

impl ClusterManager {
    pub fn new(node_id: NodeId, peers: Vec<NodeId>) -> Self {
        let config = Config::new(node_id);
        let state = ClusterState {
            nodes: peers.into_iter().map(|id| (id, format!("127.0.0.1:{}", 50000 + id.0))).collect(),
            queue_assignments: HashMap::new(),
            node_loads: HashMap::new(),
        };
        let node = Node::new(config, state.clone());

        ClusterManager {
            node: Arc::new(Mutex::new(node)),
            state: Arc::new(Mutex::new(state)),
            rpc_clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_node(&self, node_id: NodeId, address: String) {
        let mut state = self.state.lock().unwrap();
        state.nodes.insert(node_id, address);
        state.node_loads.insert(node_id, 0);
        self.rebalance_queues();
    }

    pub fn remove_node(&self, node_id: NodeId) {
        let mut state = self.state.lock().unwrap();
        state.nodes.remove(&node_id);
        state.node_loads.remove(&node_id);
        self.rebalance_queues();
    }

    pub async fn sync_state(&self) {
        let state = self.state.lock().unwrap().clone();
        for (node_id, _) in state.nodes.iter() {
            if *node_id != self.node.lock().unwrap().id() {
                if let Err(e) = self.send_state_update(*node_id, state.clone()).await {
                    eprintln!("Failed to sync state with node {}: {}", node_id, e);
                }
            }
        }
    }

    async fn send_state_update(&self, node_id: NodeId, state: ClusterState) -> Result<(), Box<dyn std::error::Error>> {
        let mut clients = self.rpc_clients.lock().unwrap();
        let client = clients.entry(node_id).or_insert_with(|| {
            let addr = self.state.lock().unwrap().nodes.get(&node_id).unwrap().clone();
            let channel = Channel::from_shared(addr)
                .unwrap()
                .tls_config(tonic::transport::ClientTlsConfig::new())
                .unwrap()
                .connect_lazy();
            rapidmq::rapid_mq_client::RapidMqClient::new(channel)
        });

        let request = tonic::Request::new(StateUpdateRequest {
            state: serde_json::to_string(&state).unwrap(),
        });

        client.update_state(request).await?;
        Ok(())
    }

    fn rebalance_queues(&self) {
        let mut state = self.state.lock().unwrap();
        let total_queues: usize = state.queue_assignments.len();
        let nodes_count = state.nodes.len();
        let target_load = (total_queues as f64 / nodes_count as f64).ceil() as usize;

        let mut node_loads: HashMap<NodeId, usize> = state.nodes.keys().map(|&id| (id, 0)).collect();
        for &node_id in state.queue_assignments.values() {
            *node_loads.entry(node_id).or_default() += 1;
        }

        let mut queues_to_reassign = Vec::new();
        for (queue, node) in state.queue_assignments.iter() {
            if node_loads[node] > target_load {
                queues_to_reassign.push(queue.clone());
            }
        }

        for queue in queues_to_reassign {
            let old_node = state.queue_assignments[&queue];
            let new_node = self.least_loaded_node(&node_loads);
            state.queue_assignments.insert(queue, new_node);
            *node_loads.entry(old_node).or_default() -= 1;
            *node_loads.entry(new_node).or_default() += 1;
        }

        state.node_loads = node_loads;
    }

    fn least_loaded_node(&self, node_loads: &HashMap<NodeId, usize>) -> NodeId {
        node_loads.iter()
            .min_by_key(|&(_, load)| load)
            .map(|(&node_id, _)| node_id)
            .unwrap_or_else(|| self.state.lock().unwrap().nodes.keys().next().unwrap().clone())
    }

    pub fn assign_queue(&self, queue_name: &str) -> NodeId {
        let mut state = self.state.lock().unwrap();
        let node_id = self.least_loaded_node(&state.node_loads);
        state.queue_assignments.insert(queue_name.to_string(), node_id);
        *state.node_loads.entry(node_id).or_default() += 1;
        node_id
    }

    pub fn get_queue_node(&self, queue_name: &str) -> Option<NodeId> {
        let state = self.state.lock().unwrap();
        state.queue_assignments.get(queue_name).cloned()
    }

    pub fn get_state(&self) -> ClusterState {
        self.state.lock().unwrap().clone()
    }

    pub async fn run(&self) {
        let node = self.node.clone();
        let state = self.state.clone();
        
        // Set up TLS
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
        builder.set_certificate_chain_file("cert.pem").unwrap();
        let acceptor = builder.build();
        
        // Start the RPC server
        let addr = format!("127.0.0.1:{}", 50000 + self.node.lock().unwrap().id().0).parse().unwrap();
        let rapid_mq = RapidMqService { state: state.clone() };
        
        tokio::spawn(async move {
            Server::builder()
                .tls_config(acceptor).unwrap()
                .add_service(RapidMqServer::new(rapid_mq))
                .serve(addr)
                .await
                .unwrap();
        });

        // Run the Raft node
        tokio::spawn(async move {
            loop {
                let mut node = node.lock().unwrap();
                node.tick();
                if let Some(leader) = node.leader() {
                    if leader == node.id() {
                        // This node is the leader, perform leader duties
                        self.perform_leader_duties().await;
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        });
    }

    async fn perform_leader_duties(&self) {
        // Synchronize state across all nodes
        self.sync_state().await;
        // Rebalance queues if necessary
        self.rebalance_queues();
    }

    pub async fn publish_remote(&self, node_id: NodeId, queue_name: &str, message: crate::Message) -> Result<(), Box<dyn std::error::Error>> {
        let mut clients = self.rpc_clients.lock().unwrap();
        let client = clients.entry(node_id).or_insert_with(|| {
            let addr = self.state.lock().unwrap().nodes.get(&node_id).unwrap().clone();
            let channel = Channel::from_shared(addr)
                .unwrap()
                .tls_config(tonic::transport::ClientTlsConfig::new())
                .unwrap()
                .connect_lazy();
            rapidmq::rapid_mq_client::RapidMqClient::new(channel)
        });

        let request = tonic::Request::new(PublishRequest {
            queue_name: queue_name.to_string(),
            message_id: message.id.clone(),
            content: message.content.clone(),
        });

        client.publish_message(request).await?;
        Ok(())
    }

    pub async fn consume_remote(&self, node_id: NodeId, queue_name: &str) -> Result<Option<crate::Message>, Box<dyn std::error::Error>> {
        let mut clients = self.rpc_clients.lock().unwrap();
        let client = clients.entry(node_id).or_insert_with(|| {
            let addr = self.state.lock().unwrap().nodes.get(&node_id).unwrap().clone();
            rapidmq::rapid_mq_client::RapidMqClient::connect(addr)
        });

        let request = tonic::Request::new(ConsumeRequest {
            queue_name: queue_name.to_string(),
        });

        let response = client.consume_message(request).await?;
        let message = response.into_inner();

        if message.message_id.is_empty() {
            Ok(None)
        } else {
            Ok(Some(crate::Message {
                id: message.message_id,
                content: message.content,
            }))
        }
    }
}

pub struct RapidMqService {
    state: Arc<Mutex<ClusterState>>,
}

#[tonic::async_trait]
impl RapidMq for RapidMqService {
    async fn publish_message(
        &self,
        request: Request<PublishRequest>,
    ) -> Result<Response<PublishResponse>, Status> {
        let req = request.into_inner();
        // Implement the logic to publish the message locally
        // This will depend on how you've implemented your local queue management
        Ok(Response::new(PublishResponse { success: true }))
    }

    async fn consume_message(
        &self,
        request: Request<ConsumeRequest>,
    ) -> Result<Response<ConsumeResponse>, Status> {
        let req = request.into_inner();
        // Implement the logic to consume a message locally
        // This will depend on how you've implemented your local queue management
        Ok(Response::new(ConsumeResponse {
            message_id: "".to_string(),
            content: "".to_string(),
        }))
    }

    async fn update_state(
        &self,
        request: Request<StateUpdateRequest>,
    ) -> Result<Response<StateUpdateResponse>, Status> {
        let req = request.into_inner();
        let new_state: ClusterState = serde_json::from_str(&req.state).map_err(|e| Status::internal(e.to_string()))?;
        *self.state.lock().unwrap() = new_state;
        Ok(Response::new(StateUpdateResponse { success: true }))
    }
}