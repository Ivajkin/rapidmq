pub mod messaging;
pub mod clustering;
pub mod ai;
pub mod quantum;
pub mod storage;
pub mod api;
pub mod monitoring;
pub mod plugins;

// Re-export commonly used items
pub use messaging::Message;
pub use clustering::ClusterManager;
pub use ai::AIModule;
pub use quantum::QuantumModule;