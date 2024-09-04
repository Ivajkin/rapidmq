# RapidMQ Deployment Guide

## Table of Contents
1. [System Requirements](#system-requirements)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Deployment Scenarios](#deployment-scenarios)
5. [Monitoring and Management](#monitoring-and-management)
6. [Troubleshooting](#troubleshooting)

## System Requirements

- Operating System: Linux (Ubuntu 20.04 LTS or later recommended), macOS 10.15+, Windows Server 2019+
- CPU: 4+ cores
- RAM: 8GB+ (16GB+ recommended for production)
- Storage: SSD with at least 20GB free space
- Network: Gigabit Ethernet
- Java: OpenJDK 11 or later

## Installation

### Using Docker (Recommended)

1. Pull the RapidMQ Docker image:
   ```
   docker pull rapidmq/rapidmq:latest
   ```

2. Run the container:
   ```
   docker run -d --name rapidmq -p 9092:9092 -p 9093:9093 rapidmq/rapidmq:latest
   ```

### Manual Installation

1. Download the latest RapidMQ release from our website.

2. Extract the archive:
   ```
   tar -xzf rapidmq-<version>.tar.gz
   ```

3. Navigate to the extracted directory:
   ```
   cd rapidmq-<version>
   ```

4. Run the setup script:
   ```
   ./setup.sh
   ```

## Configuration

The main configuration file is `config/rapidmq.yaml`. Key settings include:

- `broker.id`: Unique identifier for this broker
- `listeners`: Network listeners for client connections
- `log.dirs`: Directories for storing log files
- `num.partitions`: Default number of partitions per topic

Example configuration:
