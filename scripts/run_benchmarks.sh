#!/bin/bash
set -e

# Configuration
BENCHMARK_BINARY="target/release/rapidmq_benchmarks"

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Build the benchmark binary
cargo build --release --bin rapidmq_benchmarks

# Run the benchmarks
$BENCHMARK_BINARY

echo "Benchmarks completed successfully!"