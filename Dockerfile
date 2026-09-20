# syntax=docker/dockerfile:1

FROM rust:latest AS builder

WORKDIR /app

# Native TLS support for sqlx and a cache-friendly dependency layer.
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && printf 'fn main() {}\\n' > src/main.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
COPY migrations ./migrations
RUN touch src/main.rs \
    && cargo build --release \
    && strip target/release/actix-supabase-api

FROM debian:bookworm-slim AS runner

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --uid 10001 appuser
WORKDIR /app

COPY --from=builder /app/target/release/actix-supabase-api /usr/local/bin/actix-supabase-api

USER appuser
EXPOSE 8080

ENV HOST=0.0.0.0 \
    PORT=8080 \
    RUST_LOG=actix_supabase_api=info,actix_web=info

ENTRYPOINT ["/usr/local/bin/actix-supabase-api"]