# syntax=docker/dockerfile:1

FROM rust:1.89-bookworm AS build
WORKDIR /app

COPY Cargo.toml Cargo.toml
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

COPY --from=build /app/target/release/tornare /usr/local/bin/tornare

CMD ["tornare"]
