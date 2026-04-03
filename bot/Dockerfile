# syntax=docker/dockerfile:1

FROM rust:1.89-bookworm AS build
WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /app/target/release/tornare-bot /usr/local/bin/tornare-bot

CMD ["tornare-bot"]
