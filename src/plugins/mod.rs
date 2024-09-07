use async_trait::async_trait;
use crate::modules::messaging::Message;

#[async_trait]
pub trait Plugin: Send + Sync {
    async fn on_message(&self, message: &mut Message) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct PluginManager {
    plugins: Vec<Box<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        PluginManager { plugins: Vec::new() }
    }

    pub fn load_plugin<P: Plugin + 'static>(&mut self, plugin: P) {
        self.plugins.push(Box::new(plugin));
    }

    pub async fn process_message(&self, message: &mut Message) -> Result<(), Box<dyn std::error::Error>> {
        for plugin in &self.plugins {
            plugin.on_message(message).await?;
        }
        Ok(())
    }
}