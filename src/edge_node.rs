use rusqlite::{params, Connection, Result};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use reqwest::Client;

pub struct EdgeNode {
    node_id: String,
    broker_url: String,
    connection: Arc<Mutex<Connection>>,
    http_client: Client,
}

impl EdgeNode {
    pub fn new(node_id: String, broker_url: String) -> Result<Self> {
        let db_path = format!("edge_node_{}.db", node_id);
        let connection = Connection::open(db_path)?;
        Self::init_db(&connection)?;
        
        Ok(EdgeNode {
            node_id,
            broker_url,
            connection: Arc::new(Mutex::new(connection)),
            http_client: Client::new(),
        })
    }

    fn init_db(conn: &Connection) -> Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                topic TEXT,
                payload TEXT,
                timestamp INTEGER,
                status TEXT
            )",
            [],
        )?;
        Ok(())
    }

    pub fn store_message(&self, topic: &str, payload: Value) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO messages (topic, payload, timestamp, status)
             VALUES (?, ?, strftime('%s', 'now'), 'pending')",
            params![topic, payload.to_string()],
        )?;
        Ok(())
    }

    pub fn get_pending_messages(&self, limit: u32) -> Result<Vec<(i64, String, String, i64)>> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, topic, payload, timestamp
             FROM messages
             WHERE status = 'pending'
             ORDER BY timestamp
             LIMIT ?",
        )?;
        let messages = stmt.query_map([limit], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
            ))
        })?;
        messages.collect()
    }

    pub fn mark_messages_sent(&self, message_ids: &[i64]) -> Result<()> {
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn.prepare("UPDATE messages SET status = 'sent' WHERE id = ?")?;
        for &id in message_ids {
            stmt.execute([id])?;
        }
        Ok(())
    }

    pub async fn sync_with_broker(&self) -> Result<()> {
        let pending_messages = self.get_pending_messages(100)?;
        if pending_messages.is_empty() {
            return Ok(());
        }

        let response = self.http_client
            .post(&format!("{}/edge_sync", self.broker_url))
            .json(&serde_json::json!({
                "node_id": self.node_id,
                "messages": pending_messages
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let processed_ids: Vec<i64> = response.json().await?;
            self.mark_messages_sent(&processed_ids)?;
        }

        Ok(())
    }
}