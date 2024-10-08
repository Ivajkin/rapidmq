use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rapidmq::{RapidMQ, Message};
use raft::NodeId;
use crate::proto::RapidMQMessage;
use prost::Message as ProstMessage;

fn benchmark_publish(c: &mut Criterion) {
    let node_id = NodeId::from(1);
    let peers = vec![NodeId::from(2), NodeId::from(3)];
    let rapidmq = RapidMQ::new(node_id, peers);
    rapidmq.create_queue("test_queue");

    c.bench_function("publish message", |b| {
        b.iter(|| {
            let message = Message {
                id: "1".to_string(),
                content: "Test message".to_string(),
            };
            let proto_message: RapidMQMessage = message.into();
            let encoded = proto_message.encode_to_vec();
            black_box(rapidmq.publish("test_queue", encoded));
        })
    });
}

fn benchmark_consume(c: &mut Criterion) {
    let node_id = NodeId::from(1);
    let peers = vec![NodeId::from(2), NodeId::from(3)];
    let rapidmq = RapidMQ::new(node_id, peers);
    rapidmq.create_queue("test_queue");

    // Pre-populate the queue
    for i in 0..1000 {
        let message = Message {
            id: i.to_string(),
            content: format!("Test message {}", i),
        };
        rapidmq.publish("test_queue", message);
    }

    c.bench_function("consume message", |b| {
        b.iter(|| {
            black_box(rapidmq.consume("test_queue"));
        })
    });
}

criterion_group!(benches, benchmark_publish, benchmark_consume);
criterion_main!(benches);