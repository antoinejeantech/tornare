# syntax=docker/dockerfile:1
# Build context: workspace root (so Cargo.lock is available)

FROM rust:1.89-bookworm AS build
WORKDIR /app

# Copy workspace manifest and lockfile first
COPY Cargo.toml Cargo.lock ./
# Copy member manifests so the workspace resolves
COPY backend/Cargo.toml backend/Cargo.toml
COPY bot/Cargo.toml bot/Cargo.toml
# Stub the backend src — we only build tornare-bot
RUN mkdir -p backend/src && echo '' > backend/src/lib.rs
# Real bot source
COPY bot/src bot/src
RUN cargo build --locked --release -p tornare-bot

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/tornare-bot /usr/local/bin/tornare-bot

CMD ["tornare-bot"]
