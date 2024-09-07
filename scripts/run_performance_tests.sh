#!/bin/bash
set -e

# Configuration
DURATION=300  # 5 minutes
PUBLISH_RATE=1000  # messages per second
CONSUME_RATE=950  # messages per second

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Build the release version
cargo build --release

# Start the RapidMQ server
./target/release/rapidmq &
SERVER_PID=$!

# Wait for the server to start
sleep 5

# Run the performance test
echo "Starting performance test..."
./target/release/rapidmq_perf_test --duration $DURATION --publish-rate $PUBLISH_RATE --consume-rate $CONSUME_RATE

# Stop the server
kill $SERVER_PID

echo "Performance test completed successfully!"