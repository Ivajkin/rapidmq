use rapidmq::{RapidMQ, api};
use raft::NodeId;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let node_id = NodeId::from(1); // This should be configured for each node
    let peers = vec![NodeId::from(2), NodeId::from(3)]; // This should be configured for the cluster

    let rapidmq = RapidMQ::new(node_id, peers);
    
    // Run the cluster manager
    let rapidmq_clone = rapidmq.clone();
    tokio::spawn(async move {
        rapidmq_clone.run().await;
    });

    println!("Starting RapidMQ API server on http://127.0.0.1:8080");
    api::start_api(rapidmq).await
}