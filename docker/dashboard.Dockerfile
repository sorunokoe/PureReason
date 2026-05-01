# Multi-stage build for pure-reason-dashboard
FROM rust:1.82-slim as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

RUN cargo build -p pure-reason-dashboard --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/pure-reason-dashboard /app/pure-reason-dashboard

EXPOSE 8081

ENV RUST_LOG=info

CMD ["/app/pure-reason-dashboard", "--bind", "0.0.0.0:8081"]
