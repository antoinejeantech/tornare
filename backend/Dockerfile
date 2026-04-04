# syntax=docker/dockerfile:1
# Build context: workspace root (so Cargo.lock is available)

FROM rust:1.89-bookworm AS build
WORKDIR /app

# Copy workspace manifest and lockfile first
COPY Cargo.toml Cargo.lock ./
# Copy member manifests so the workspace resolves
COPY bot/Cargo.toml bot/Cargo.toml
COPY backend/Cargo.toml backend/Cargo.toml
# Stub the bot src — we only build tornare
RUN mkdir -p bot/src && echo 'fn main() {}' > bot/src/main.rs
# Real backend source and migrations
COPY backend/src backend/src
COPY backend/migrations backend/migrations
RUN cargo build --locked --release -p tornare

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=build /app/target/release/tornare /usr/local/bin/tornare

CMD ["tornare"]
