# Build stage
# Build stage - using latest stable Rust
FROM rust:latest AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files (no Cargo.lock - let cargo regenerate)
COPY Cargo.toml ./
COPY api ./api
COPY cli ./cli
COPY shared ./shared

# Generate lockfile and build release binaries
RUN cargo generate-lockfile && cargo build --release -p flaglite-api -p flaglite-cli

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false flaglite

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/flaglite-api /usr/local/bin/
COPY --from=builder /app/target/release/flaglite /usr/local/bin/

# Create data directory
RUN mkdir -p /data && chown flaglite:flaglite /data

USER flaglite

# Default environment
ENV DATABASE_URL=sqlite:/data/flaglite.db
ENV RUST_LOG=info

EXPOSE 8080

CMD ["flaglite-api", "serve", "--port", "8080"]
