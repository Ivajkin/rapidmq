# Use BuildKit
# syntax=docker/dockerfile:1.4

# Stage 1: Build the React frontend
FROM node:16 AS frontend-builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm install --legacy-peer-deps
COPY public ./public
COPY src ./src
RUN npm run build

# Stage 2: Build the Rust backend
FROM rust:1.68 as builder
WORKDIR /usr/src/rapidmq
COPY . .
RUN cargo build --release

# Stage 3: Create the final runtime image
FROM debian:buster-slim
RUN apt-get update && apt-get install -y libssl1.1 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rapidmq/target/release/rapidmq /usr/local/bin/rapidmq
COPY --from=frontend-builder /app/build /usr/local/bin/frontend
COPY config/rapidmq.yaml /etc/rapidmq/config.yaml
EXPOSE 8080 50051 9090
CMD ["rapidmq", "--config", "/etc/rapidmq/config.yaml"]