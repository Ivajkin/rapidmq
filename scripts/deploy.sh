#!/bin/bash
set -e

# Configuration
DOCKER_IMAGE="rapidmq"
DOCKER_TAG="latest"
REMOTE_HOST="your-remote-host.com"
REMOTE_USER="your-remote-user"
REMOTE_DIR="/path/to/rapidmq"

# Ensure we're in the project root
cd "$(dirname "$0")/.."

# Build the Docker image
docker build -t $DOCKER_IMAGE:$DOCKER_TAG .

# Save the Docker image to a file
docker save $DOCKER_IMAGE:$DOCKER_TAG | gzip > rapidmq_image.tar.gz

# Copy the Docker image to the remote host
scp rapidmq_image.tar.gz $REMOTE_USER@$REMOTE_HOST:$REMOTE_DIR

# SSH into the remote host and deploy
ssh $REMOTE_USER@$REMOTE_HOST << EOF
    cd $REMOTE_DIR
    docker load < rapidmq_image.tar.gz
    docker stop rapidmq || true
    docker rm rapidmq || true
    docker run -d --name rapidmq -p 9092:9092 -p 9093:9093 $DOCKER_IMAGE:$DOCKER_TAG
    rm rapidmq_image.tar.gz
EOF

# Clean up local image file
rm rapidmq_image.tar.gz

echo "Deployment completed successfully!"