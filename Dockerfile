# Use stable Rust for production builds
FROM rust:1.65-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy mr_darkpromth workspace
COPY mr_darkpromth ./mr_darkpromth
COPY migrations ./migrations

# Build directly from workspace without lock file
RUN cd mr_darkpromth && \
    rm -f Cargo.lock && \
    cargo build --release --bin mr_darkpromth_api

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies including curl for health checks
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1000 appuser

WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/mr_darkpromth/target/release/mr_darkpromth_api /usr/local/bin/mr_darkpromth

# Create memory directory
RUN mkdir -p /app/memory && chown appuser:appuser /app/memory

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the application
CMD ["mr_darkpromth"]
