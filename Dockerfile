# Use BuildKit
# syntax=docker/dockerfile:1.4

FROM alpine:3.19 AS base
RUN apk add --no-cache curl

FROM base AS rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

FROM rust AS chef
WORKDIR /app
RUN apk add --no-cache musl-dev cmake
# Install specific version of protoc
RUN apk add --no-cache protobuf-dev=3.21.12-r0
RUN cargo install --version 0.1.62 cargo-chef

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
ARG CARGO_BUILD_JOBS
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-4}
ENV CARGO_BUILD_RUSTC_WRAPPER=sccache
RUN cargo install sccache
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.cache/sccache \
    cargo chef cook --release --recipe-path recipe.json && \
    rm -rf target
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/root/.cargo/git \
    --mount=type=cache,target=/root/.cache/sccache \
    cargo build --release && \
    cp /app/target/release/rapidmq /app/rapidmq && \
    rm -rf target

FROM alpine:3.19 AS runtime
WORKDIR /app
RUN apk add --no-cache libgcc
COPY --from=builder /app/rapidmq /app/rapidmq
CMD ["/app/rapidmq"]