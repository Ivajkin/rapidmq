#!/bin/bash

# Add protoc to PATH
export PATH="/usr/local/bin:$PATH"

# Print protoc version for debugging
echo "protoc version: $(protoc --version)"

# Determine the number of CPU cores
if [[ "$(uname)" == "Darwin" ]]; then
    # macOS
    CORES=$(sysctl -n hw.ncpu)
elif [[ "$(expr substr $(uname -s) 1 5)" == "Linux" ]]; then
    # Linux
    CORES=$(nproc)
else
    # Default to 4 cores if we can't determine
    CORES=4
fi

# Set PROTOC environment variable explicitly
export PROTOC=$(which protoc)

# Build the Docker image in stages
DOCKER_BUILDKIT=1 docker build --target base -t rapidmq:base .
DOCKER_BUILDKIT=1 docker build --target rust -t rapidmq:rust .
DOCKER_BUILDKIT=1 docker build --target chef -t rapidmq:chef .
DOCKER_BUILDKIT=1 docker build --target planner -t rapidmq:planner .
DOCKER_BUILDKIT=1 docker build --build-arg CARGO_BUILD_JOBS=$CORES -t rapidmq:latest .

# Prune Docker system
docker system prune -af

echo "Build completed successfully!"