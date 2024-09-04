# Use a more recent Rust image
FROM rust:1.75 as builder

# Set the working directory in the container
WORKDIR /usr/src/rapidmq

# Copy the current directory contents into the container
COPY . .

# Build the application
RUN cargo build --release

# Start a new stage for a smaller final image
FROM debian:bullseye-slim

# Install OpenSSL and CA certificates
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/rapidmq/target/release/rapidmq /usr/local/bin/rapidmq

# Set the startup command to run your binary
CMD ["rapidmq"]