# Build stage
FROM rust:1.74-slim-bullseye as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire workspace
COPY . .

# Build the API Gateway service
RUN cargo build --release --package api_gateway

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/release/api_gateway /app/api_gateway

# Create non-root user
RUN useradd -r -u 1000 -g root lotabots
USER lotabots

# Set environment variables
ENV RUST_LOG=info
ENV RUST_BACKTRACE=1

# Expose the service port
EXPOSE 8080

# Run the service
CMD ["./api_gateway"] 