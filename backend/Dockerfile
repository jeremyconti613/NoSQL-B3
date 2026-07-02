# Build context is the repository root (see docker-compose.yml) so this
# image can COPY the committed data/seed.json snapshot alongside the crate.

# --- Build stage -------------------------------------------------------
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# aws-lc-rs (pulled in transitively via reqwest's rustls TLS stack) builds a
# small amount of C/asm at compile time, hence the compiler toolchain here.
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential cmake \
    && rm -rf /var/lib/apt/lists/*

# Cache dependency compilation separately from application code: as long as
# Cargo.toml/Cargo.lock don't change, this (slow) layer is reused.
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir -p src/bin \
    && echo "fn main() {}" > src/main.rs \
    && echo "" > src/lib.rs \
    && echo "fn main() {}" > src/bin/seed.rs \
    && cargo build --release --bins \
    && rm -rf src

COPY backend/src ./src
# BuildKit can assign COPY'd files a timestamp Cargo's fingerprint cache
# doesn't consider newer than the dummy build above, which would silently
# keep the dummy `fn main() {}` binaries instead of rebuilding — force fresh
# mtimes so the real source is always seen as changed.
RUN find src -exec touch {} + && cargo build --release --bins

# --- Runtime stage -------------------------------------------------------
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/musicgraph-backend ./musicgraph-backend
COPY --from=builder /app/target/release/seed ./seed
COPY data ./data

EXPOSE 8080
CMD ["./musicgraph-backend"]
