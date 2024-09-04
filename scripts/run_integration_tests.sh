#!/bin/bash
set -e

# Configuration
DOCKER_COMPOSE_FILE="docker-compose.test.yml"

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Build and start the test environment
docker-compose -f $DOCKER_COMPOSE_FILE up -d

# Run the integration tests
cargo test --test '*' -- --test-threads=1

# Tear down the test environment
docker-compose -f $DOCKER_COMPOSE_FILE down

echo "Integration tests completed successfully!"