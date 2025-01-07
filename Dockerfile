FROM rust:latest as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 -U -s /bin/bash developer

# Install development tools
RUN cargo install cargo-watch cargo-edit sqlx-cli

# Set working directory
WORKDIR /app

# Copy project files
COPY . .

# Build the application
RUN cargo build

# Switch to non-root user
USER developer

# Keep the container running for development
CMD ["sleep", "infinity"]

# Production stage
FROM debian:bullseye-slim as production

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    libtesseract4 \
    tesseract-ocr-eng \
    && rm -rf /var/lib/apt/lists/*

# Create config directory
RUN mkdir -p /etc/document-automation/config

# Copy the binary from builder
COPY --from=builder /usr/src/app/target/debug/document-automation /usr/local/bin/

# Set the entrypoint
ENTRYPOINT ["document-automation"] 