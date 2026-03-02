# Stage 1: Plan recipes
FROM lukemathwalker/cargo-chef:latest-rust-1.93 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y \
    protobuf-compiler \
    libpq-dev \
    pkg-config \
    libssl-dev \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build dependencies and binaries
FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*


COPY --from=builder /app/target/release/api-gateway /app/api-gateway
COPY --from=builder /app/target/release/auth-service /app/auth-service
COPY --from=builder /app/target/release/canvas-service /app/canvas-service
COPY --from=builder /app/target/release/chat-service /app/chat-service
COPY --from=builder /app/target/release/voting-service /app/voting-service
COPY --from=builder /app/target/release/group-service /app/group-service
COPY --from=builder /app/target/release/scheduler-service /app/scheduler-service
