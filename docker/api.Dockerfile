# Multi-stage build for pure-reason-api
# Stage 1: Build
FROM rust:1.82-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build release binary
RUN cargo build -p pure-reason-api --release

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/pure-reason-api /app/pure-reason-api

# API key file (mounted at runtime)
RUN mkdir -p /app/config

EXPOSE 8080

ENV RUST_LOG=info
ENV PURE_REASON_BIND=0.0.0.0:8080

CMD ["/app/pure-reason-api", "--bind", "0.0.0.0:8080"]
