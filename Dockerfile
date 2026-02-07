# Multi-stage build for MR.DarkPromth API
FROM rust:1.83-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency definitions to cache layers
COPY Cargo.toml Cargo.lock ./
COPY mr_darkpromth/core/Cargo.toml ./mr_darkpromth/core/
COPY mr_darkpromth/db/Cargo.toml ./mr_darkpromth/db/
COPY mr_darkpromth/services/Cargo.toml ./mr_darkpromth/services/
COPY mr_darkpromth/services/cerebras_client/Cargo.toml ./mr_darkpromth/services/cerebras_client/
COPY mr_darkpromth/api/Cargo.toml ./mr_darkpromth/api/

# Create dummy source files to build and cache dependencies only
RUN mkdir -p mr_darkpromth/core/src && echo "pub fn lib() {}" > mr_darkpromth/core/src/lib.rs && \
    mkdir -p mr_darkpromth/db/src && echo "pub fn lib() {}" > mr_darkpromth/db/src/lib.rs && \
    mkdir -p mr_darkpromth/services/src && echo "pub fn lib() {}" > mr_darkpromth/services/src/lib.rs && \
    mkdir -p mr_darkpromth/services/cerebras_client/src && echo "pub fn lib() {}" > mr_darkpromth/services/cerebras_client/src/lib.rs && \
    mkdir -p mr_darkpromth/api/src && echo "fn main() {}" > mr_darkpromth/api/src/main.rs

# Build only dependencies
# This will be cached unless Cargo files change
RUN cargo build --release

# Now copy the actual source code
COPY ./mr_darkpromth ./mr_darkpromth
COPY ./migrations ./migrations
COPY ./config ./config

# Build the application with the cached dependencies
# This will be fast because dependencies are cached
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/mr_darkpromth_api /usr/local/bin/mr_darkpromth_api

# Copy migrations and config
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/config ./config

# Create necessary directories
RUN mkdir -p /app/memory /app/logs && chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["mr_darkpromth_api"]
