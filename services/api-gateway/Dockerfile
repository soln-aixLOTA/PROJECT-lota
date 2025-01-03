# Build stage
FROM rust:1.75-slim-bullseye as builder

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

# Copy the build artifact from the builder stage
COPY --from=builder /usr/src/api-gateway/target/release/api-gateway /usr/local/bin/api-gateway

# Create a non-root user
RUN useradd -m -U -s /bin/false api-gateway

# Set the ownership of the binary
RUN chown api-gateway:api-gateway /usr/local/bin/api-gateway

# Switch to non-root user
USER api-gateway

# Create necessary directories with correct permissions
RUN mkdir -p /home/api-gateway/logs
RUN mkdir -p /home/api-gateway/data

# Set environment variables
ENV RUST_LOG=info
ENV HOST=0.0.0.0
ENV PORT=8080

# Expose the application port
EXPOSE 8080

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/api-gateway"]

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/api/v1/health || exit 1 