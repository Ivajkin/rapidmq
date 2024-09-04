class MessageBroker:
    # ... existing code ...

    def handle_edge_node_sync(self, edge_node_id, messages):
        for msg_id, topic, payload, timestamp in messages:
            self.publish(topic, payload, timestamp)
        return [msg[0] for msg in messages]  # Return list of processed message IDs

    # ... existing code ...