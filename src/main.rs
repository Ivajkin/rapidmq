use rapidmq::{RapidMQ, api};
use raft::NodeId;
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let node_id = NodeId::from(args[1].parse::<u64>().unwrap());
    let peers: Vec<NodeId> = args[2..].iter().map(|s| NodeId::from(s.parse::<u64>().unwrap())).collect();

    let rapidmq = RapidMQ::new(node_id, peers);
    
    // Run the cluster manager
    let rapidmq_clone = rapidmq.clone();
    tokio::spawn(async move {
        rapidmq_clone.run().await;
    });

    println!("Starting RapidMQ API server on http://127.0.0.1:8080");
    api::start_api(rapidmq).await
}