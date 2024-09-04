from .edge_node import EdgeNode

class Producer:
    def __init__(self, broker_url, edge_node_id=None):
        self.broker_url = broker_url
        self.edge_node = EdgeNode(edge_node_id, broker_url) if edge_node_id else None

    def send(self, topic, message):
        if self.edge_node:
            self.edge_node.store_message(topic, message)
        else:
            # Existing code to send message directly to broker
            pass

    def flush(self):
        if self.edge_node:
            self.edge_node.sync_with_broker()
        # Existing flush logic for direct broker connection