# Build stage
FROM rust:1.70-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY services/user-management/Cargo.toml ./services/user-management/
COPY common/Cargo.toml ./common/

# Create dummy source files to build dependencies
RUN mkdir -p services/user-management/src && \
    echo "fn main() {}" > services/user-management/src/main.rs && \
    mkdir -p common/src && \
    echo "pub fn dummy() {}" > common/src/lib.rs

# Build dependencies
RUN cargo build --release --package user-management

# Copy actual source code
COPY services/user-management/src ./services/user-management/src
COPY common/src ./common/src

# Build the application
RUN cargo build --release --package user-management

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates libssl1.1 && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/user-management .
COPY services/user-management/config ./config
COPY services/user-management/migrations ./migrations

# Set environment variables
ENV RUST_LOG=info
ENV CONFIG_PATH=/app/config

# Expose port
EXPOSE 8081

# Run the binary
CMD ["./user-management"] 