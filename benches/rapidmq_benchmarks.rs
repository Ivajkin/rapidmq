use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rapidmq::{RapidMQ, Message};
use raft::NodeId;

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
            black_box(rapidmq.publish("test_queue", message));
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

fn benchmark_ai_prioritization(c: &mut Criterion) {
    // Implement benchmark for AI-based message prioritization
}

fn benchmark_quantum_routing(c: &mut Criterion) {
    // Implement benchmark for quantum-inspired routing optimization
}

criterion_group!(benches, benchmark_publish, benchmark_consume, benchmark_ai_prioritization, benchmark_quantum_routing);
criterion_main!(benches);