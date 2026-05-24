# syntax=docker/dockerfile:1.7

# --- builder ---
FROM rust:1.95-bookworm AS builder
WORKDIR /app

COPY rust-toolchain.toml Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY src ./src
COPY migrations ./migrations

ENV SQLX_OFFLINE=true
RUN cargo build --release --bin dearday

# --- runtime ---
FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/dearday /usr/local/bin/dearday
COPY prod/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

EXPOSE 3000
ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
