# Use stable Rust for production builds
FROM rust:1.77-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml ./

# Create dummy src directory for cargo update
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Generate Cargo.lock if not exists
RUN if [ ! -f Cargo.lock ]; then cargo update; fi

# Remove dummy src
RUN rm -rf src

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations

# Build the application
RUN touch src/main.rs && cargo build --release

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
COPY --from=builder /app/target/release/mr_darkpromth /usr/local/bin/mr_darkpromth

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
