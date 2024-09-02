use clap::{Parser, Subcommand};
use std::net::SocketAddr;

#[derive(Parser)]
#[clap(name = "RapidMQ CLI", version = "1.0", author = "Your Name", about = "CLI for managing RapidMQ")]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new queue
    CreateQueue {
        /// The name of the queue
        queue_name: String,
    },
    /// Publish a message to a queue
    PublishMessage {
        /// The name of the queue
        queue_name: String,
        /// The message content
        message: String,
    },
    /// Consume a message from a queue
    ConsumeMessage {
        /// The name of the queue
        queue_name: String,
    },
    /// Add a new node to the cluster
    AddNode {
        /// The ID of the node
        node_id: u64,
        /// The address of the node
        address: SocketAddr,
    },
    /// Remove a node from the cluster
    RemoveNode {
        /// The ID of the node
        node_id: u64,
    },
}