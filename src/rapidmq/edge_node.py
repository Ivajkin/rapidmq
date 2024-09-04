import sqlite3
import json
from threading import Lock
import requests

class EdgeNode:
    def __init__(self, node_id, broker_url):
        self.node_id = node_id
        self.broker_url = broker_url
        self.db_path = f"edge_node_{node_id}.db"
        self.connection = sqlite3.connect(self.db_path)
        self.lock = Lock()
        self._init_db()

    def _init_db(self):
        with self.connection:
            self.connection.execute("""
                CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    topic TEXT,
                    payload BLOB,
                    timestamp INTEGER,
                    status TEXT
                )
            """)

    def store_message(self, topic, payload):
        with self.lock:
            with self.connection:
                self.connection.execute("""
                    INSERT INTO messages (topic, payload, timestamp, status)
                    VALUES (?, ?, strftime('%s', 'now'), 'pending')
                """, (topic, json.dumps(payload)))

    def get_pending_messages(self, limit=100):
        with self.lock:
            with self.connection:
                cursor = self.connection.execute("""
                    SELECT id, topic, payload, timestamp
                    FROM messages
                    WHERE status = 'pending'
                    ORDER BY timestamp
                    LIMIT ?
                """, (limit,))
                return cursor.fetchall()

    def mark_messages_sent(self, message_ids):
        with self.lock:
            with self.connection:
                self.connection.executemany("""
                    UPDATE messages
                    SET status = 'sent'
                    WHERE id = ?
                """, [(id,) for id in message_ids])

    def sync_with_broker(self):
        pending_messages = self.get_pending_messages()
        if not pending_messages:
            return

        try:
            response = requests.post(
                f"{self.broker_url}/edge_sync",
                json={
                    "node_id": self.node_id,
                    "messages": pending_messages
                }
            )
            if response.status_code == 200:
                processed_ids = response.json()["processed_ids"]
                self.mark_messages_sent(processed_ids)
        except requests.RequestException:
            # Handle connection errors, possibly with exponential backoff
            pass