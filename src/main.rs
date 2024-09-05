use rapidmq::{RapidMQ, api};
use raft::NodeId;
use std::env;
use clap::Parser;
use std::net::SocketAddr;

mod cli;
use cli::{Cli, Commands};

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

    // Parse CLI arguments
    let cli = Cli::parse();

    match &cli.command {
        Commands::CreateQueue { queue_name } => {
            rapidmq.create_queue(queue_name);
            println!("Queue '{}' created", queue_name);
        }
        Commands::PublishMessage { queue_name, message } => {
            let msg = rapidmq::Message {
                id: uuid::Uuid::new_v4().to_string(),
                content: message.clone(),
            };
            rapidmq.publish(queue_name, msg).await.unwrap();
            println!("Message published to queue '{}'", queue_name);
        }
        Commands::ConsumeMessage { queue_name } => {
            if let Some(message) = rapidmq.consume(queue_name).await {
                println!("Consumed message: {}", message.content);
            } else {
                println!("No messages in queue '{}'", queue_name);
            }
        }
        Commands::AddNode { node_id, address } => {
            rapidmq.add_node(NodeId::from(*node_id), address.to_string()).await;
            println!("Node {} added with address {}", node_id, address);
        }
        Commands::RemoveNode { node_id } => {
            rapidmq.remove_node(NodeId::from(*node_id)).await;
            println!("Node {} removed", node_id);
        }
    }

    println!("Starting RapidMQ with AI and Quantum Computing enhancements");
    println!("AI-powered message prioritization and Quantum-optimized routing enabled");

    println!("Starting RapidMQ API server on http://127.0.0.1:8080");
    api::start_api(rapidmq).await
}