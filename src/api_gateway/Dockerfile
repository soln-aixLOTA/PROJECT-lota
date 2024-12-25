# Builder stage
FROM rust:1.74-slim-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /usr/src/api-gateway
COPY . .

# Build with release profile
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/api-gateway/target/release/api-gateway /usr/local/bin/api-gateway

# Create a non-root user
RUN useradd -m -U -s /bin/false api-gateway

# Switch to non-root user
USER api-gateway

# Expose ports
EXPOSE 8080 9090

# Set the entrypoint
ENTRYPOINT ["api-gateway"] 