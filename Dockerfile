# syntax=docker/dockerfile:1.7

# ---- build stage ---------------------------------------------------------
FROM rust:1.95-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
  && rm -rf /var/lib/apt/lists/*

# Cache deps separately from source.
# Bump CACHEBUST date to force a full rebuild when deps change.
ARG CACHEBUST=2026-05-02
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
  && cargo build --release \
  && rm -rf src

# sqlx compile-time checks need either a live DB at build time OR
# `cargo sqlx prepare` output checked into the repo with SQLX_OFFLINE=true.
# Using offline mode here by default.
ENV SQLX_OFFLINE=true
COPY . .
RUN cargo build --release --bin broly

# ---- runtime stage -------------------------------------------------------
FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
  && rm -rf /var/lib/apt/lists/* \
  && useradd --system --create-home --shell /usr/sbin/nologin app

COPY --from=builder /app/target/release/broly /usr/local/bin/broly
COPY --from=builder /app/migrations ./migrations

USER app
ENV RUST_LOG=info,broly=debug,serenity=warn

ENTRYPOINT ["/usr/local/bin/broly"]
