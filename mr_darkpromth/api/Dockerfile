# Stage 1: Build the Rust application
FROM debian:bookworm as builder

# Install build dependencies, including curl for rustup
RUN apt-get update && apt-get install -y curl build-essential pkg-config libssl-dev ca-certificates

# Install Rust via rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Set OpenSSL environment variables to help the linker
ENV OPENSSL_DIR=/usr/include/openssl
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu

WORKDIR /usr/src/app

# --- Create a dependency caching layer --- 
# Copy all Cargo files to define the workspace
COPY Cargo.toml Cargo.lock ./
COPY mr_darkpromth/api/Cargo.toml ./mr_darkpromth/api/
COPY mr_darkpromth/db/Cargo.toml ./mr_darkpromth/db/
COPY mr_darkpromth/services/Cargo.toml ./mr_darkpromth/services/

# Create dummy source files to allow dependency building
RUN mkdir -p mr_darkpromth/api/src && echo "fn main() {}" > mr_darkpromth/api/src/main.rs
RUN mkdir -p mr_darkpromth/db/src && echo "pub fn placeholder() {}" > mr_darkpromth/db/src/lib.rs
RUN mkdir -p mr_darkpromth/services/src && echo "pub fn placeholder() {}" > mr_darkpromth/services/src/lib.rs

# Build only the dependencies
RUN cargo build --release --bin mr_darkpromth_api

# --- Actual Build ---
# Now copy all source code
COPY . .

# Build the application binary
RUN cargo build --release --bin mr_darkpromth_api

# Stage 2: Create the final small image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/app/target/release/mr_darkpromth_api /usr/local/bin/mr_darkpromth_api

# Set environment variables
ENV RUST_LOG=info

EXPOSE 8080

# Run the binary
CMD ["mr_darkpromth_api"]
